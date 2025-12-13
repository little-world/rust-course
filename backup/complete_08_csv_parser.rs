use rayon::prelude::*;
use std::collections::HashMap;
use std::fs::File;
use std::hash::Hash;
use std::io::{self, BufRead, BufReader, Read, Seek, SeekFrom};
use std::path::Path;
use std::sync::Arc;

// =============================================================================
// Milestone 1: CSV record parsing
// =============================================================================

#[derive(Debug, Clone, PartialEq)]
pub struct CsvRecord {
    fields: Vec<String>,
}

#[derive(Debug, PartialEq)]
pub enum ParseError {
    UnterminatedQuote,
    InvalidEscape,
    EmptyLine,
}

impl CsvRecord {
    pub fn parse_csv_line(line: &str, delimiter: char) -> Result<Self, ParseError> {
        let trimmed_line = line.trim_end_matches(|c| c == '\n' || c == '\r');
        if trimmed_line.trim().is_empty() {
            return Err(ParseError::EmptyLine);
        }

        let mut fields = Vec::new();
        let mut current = String::new();
        let mut chars = trimmed_line.chars().peekable();
        let mut in_quotes = false;
        let mut field_started = false;

        while let Some(ch) = chars.next() {
            if in_quotes {
                match ch {
                    '"' => {
                        if matches!(chars.peek(), Some('"')) {
                            chars.next();
                            current.push('"');
                        } else {
                            in_quotes = false;
                            field_started = true;
                        }
                    }
                    _ => {
                        current.push(ch);
                    }
                }
            } else {
                match ch {
                    c if c == delimiter => {
                        fields.push(current.clone());
                        current.clear();
                        field_started = false;
                    }
                    '"' => {
                        if !field_started {
                            in_quotes = true;
                        } else {
                            return Err(ParseError::InvalidEscape);
                        }
                    }
                    _ => {
                        current.push(ch);
                        field_started = true;
                    }
                }
            }
        }

        if in_quotes {
            return Err(ParseError::UnterminatedQuote);
        }

        fields.push(current);
        Ok(CsvRecord { fields })
    }

    pub fn get_field(&self, index: usize) -> Option<&str> {
        self.fields.get(index).map(|field| field.as_str())
    }

    pub fn field_count(&self) -> usize {
        self.fields.len()
    }

    fn map_field<F>(&mut self, index: usize, mapper: F)
    where
        F: FnOnce(&str) -> String,
    {
        if let Some(field) = self.fields.get_mut(index) {
            let new_value = mapper(field.as_str());
            *field = new_value;
        }
    }
}

// =============================================================================
// Milestone 2: Streaming iterator over CSV files
// =============================================================================

pub struct CsvFileIterator {
    reader: BufReader<File>,
    delimiter: char,
    line_number: usize,
}

#[derive(Debug)]
pub enum Error {
    Io(std::io::Error),
    Parse(ParseError, usize),
}

impl CsvFileIterator {
    pub fn new(path: &Path, delimiter: char) -> Result<Self, std::io::Error> {
        let file = File::open(path)?;
        Ok(Self {
            reader: BufReader::new(file),
            delimiter,
            line_number: 0,
        })
    }
}

impl Iterator for CsvFileIterator {
    type Item = Result<CsvRecord, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut line = String::new();

        loop {
            line.clear();
            match self.reader.read_line(&mut line) {
                Ok(0) => return None,
                Ok(_) => {
                    self.line_number += 1;
                    match CsvRecord::parse_csv_line(
                        line.trim_end_matches(|c| c == '\n' || c == '\r'),
                        self.delimiter,
                    ) {
                        Ok(record) => return Some(Ok(record)),
                        Err(ParseError::EmptyLine) => continue,
                        Err(err) => return Some(Err(Error::Parse(err, self.line_number))),
                    }
                }
                Err(err) => return Some(Err(Error::Io(err))),
            }
        }
    }
}

// =============================================================================
// Milestone 3: Type-safe column extraction
// =============================================================================

pub trait FromCsvField: Sized {
    fn from_csv_field(field: &str) -> Result<Self, ConversionError>;
}

#[derive(Debug, PartialEq)]
pub enum ConversionError {
    ParseInt(std::num::ParseIntError),
    ParseFloat(std::num::ParseFloatError),
    InvalidValue(String),
    MissingField,
}

impl FromCsvField for String {
    fn from_csv_field(field: &str) -> Result<Self, ConversionError> {
        Ok(field.to_string())
    }
}

impl FromCsvField for i64 {
    fn from_csv_field(field: &str) -> Result<Self, ConversionError> {
        field
            .trim()
            .parse::<i64>()
            .map_err(ConversionError::ParseInt)
    }
}

impl FromCsvField for f64 {
    fn from_csv_field(field: &str) -> Result<Self, ConversionError> {
        field
            .trim()
            .parse::<f64>()
            .map_err(ConversionError::ParseFloat)
    }
}

impl FromCsvField for bool {
    fn from_csv_field(field: &str) -> Result<Self, ConversionError> {
        match field.trim().to_lowercase().as_str() {
            "true" | "yes" | "1" => Ok(true),
            "false" | "no" | "0" => Ok(false),
            _ => Err(ConversionError::InvalidValue(field.to_string())),
        }
    }
}

impl CsvRecord {
    pub fn get_typed<T: FromCsvField>(&self, index: usize) -> Result<T, ConversionError> {
        let field = self.get_field(index).ok_or(ConversionError::MissingField)?;
        T::from_csv_field(field)
    }
}

// =============================================================================
// Milestone 4: Filtering and transforming CSV streams
// =============================================================================

pub trait CsvFilterExt: Iterator<Item = Result<CsvRecord, Error>> + Sized {
    fn filter_by_column<F>(self, column: usize, predicate: F) -> FilterByColumn<Self, F>
    where
        F: FnMut(&str) -> bool;

    fn filter_valid(self) -> FilterValid<Self>;
}

impl<I> CsvFilterExt for I
where
    I: Iterator<Item = Result<CsvRecord, Error>> + Sized,
{
    fn filter_by_column<F>(self, column: usize, predicate: F) -> FilterByColumn<Self, F>
    where
        F: FnMut(&str) -> bool,
    {
        FilterByColumn {
            iter: self,
            column,
            predicate,
        }
    }

    fn filter_valid(self) -> FilterValid<Self> {
        FilterValid { iter: self }
    }
}

pub struct FilterByColumn<I, F> {
    iter: I,
    column: usize,
    predicate: F,
}

impl<I, F> Iterator for FilterByColumn<I, F>
where
    I: Iterator<Item = Result<CsvRecord, Error>>,
    F: FnMut(&str) -> bool,
{
    type Item = Result<CsvRecord, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(record) = self.iter.next() {
            match record {
                Ok(record) => {
                    if let Some(value) = record.get_field(self.column) {
                        if (self.predicate)(value) {
                            return Some(Ok(record));
                        }
                    }
                }
                Err(err) => return Some(Err(err)),
            }
        }
        None
    }
}

pub struct FilterValid<I> {
    iter: I,
}

impl<I> Iterator for FilterValid<I>
where
    I: Iterator<Item = Result<CsvRecord, Error>>,
{
    type Item = CsvRecord;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(record) = self.iter.next() {
            if let Ok(record) = record {
                return Some(record);
            }
        }
        None
    }
}

pub trait CsvTransformExt: Iterator<Item = CsvRecord> + Sized {
    fn map_column<F>(self, column: usize, f: F) -> MapColumn<Self, F>
    where
        F: FnMut(&str) -> String;
}

impl<I> CsvTransformExt for I
where
    I: Iterator<Item = CsvRecord> + Sized,
{
    fn map_column<F>(self, column: usize, mapper: F) -> MapColumn<Self, F>
    where
        F: FnMut(&str) -> String,
    {
        MapColumn {
            iter: self,
            column,
            mapper,
        }
    }
}

pub struct MapColumn<I, F> {
    iter: I,
    column: usize,
    mapper: F,
}

impl<I, F> Iterator for MapColumn<I, F>
where
    I: Iterator<Item = CsvRecord>,
    F: FnMut(&str) -> String,
{
    type Item = CsvRecord;

    fn next(&mut self) -> Option<Self::Item> {
        let mut record = self.iter.next()?;
        record.map_field(self.column, |value| (self.mapper)(value));
        Some(record)
    }
}

// =============================================================================
// Milestone 5: Streaming aggregations
// =============================================================================

#[derive(Debug, Clone)]
pub struct CsvAggregator {
    pub count: usize,
    pub sum: f64,
    pub min: f64,
    pub max: f64,
}

impl CsvAggregator {
    pub fn new() -> Self {
        Self {
            count: 0,
            sum: 0.0,
            min: f64::INFINITY,
            max: f64::NEG_INFINITY,
        }
    }

    pub fn update(&mut self, value: f64) {
        if self.count == 0 {
            self.min = value;
            self.max = value;
        } else {
            if value < self.min {
                self.min = value;
            }
            if value > self.max {
                self.max = value;
            }
        }
        self.count += 1;
        self.sum += value;
    }

    pub fn mean(&self) -> Option<f64> {
        if self.count == 0 {
            None
        } else {
            Some(self.sum / self.count as f64)
        }
    }

    pub fn from_column<I>(records: I, column: usize) -> Self
    where
        I: Iterator<Item = CsvRecord>,
    {
        records.fold(CsvAggregator::new(), |mut agg, record| {
            if let Ok(value) = record.get_typed::<f64>(column) {
                agg.update(value);
            }
            agg
        })
    }

    pub fn merge(&mut self, other: CsvAggregator) {
        if other.count == 0 {
            return;
        }
        if self.count == 0 {
            *self = other;
            return;
        }
        self.count += other.count;
        self.sum += other.sum;
        if other.min < self.min {
            self.min = other.min;
        }
        if other.max > self.max {
            self.max = other.max;
        }
    }
}

pub struct GroupedAggregator<K> {
    groups: HashMap<K, CsvAggregator>,
}

impl<K: Eq + Hash> GroupedAggregator<K> {
    pub fn new() -> Self {
        Self {
            groups: HashMap::new(),
        }
    }

    pub fn update(&mut self, key: K, value: f64) {
        self.groups
            .entry(key)
            .or_insert_with(CsvAggregator::new)
            .update(value);
    }

    pub fn get(&self, key: &K) -> Option<&CsvAggregator> {
        self.groups.get(key)
    }

    pub fn from_records<I, KF, VF>(records: I, key_fn: KF, value_fn: VF) -> Self
    where
        I: Iterator<Item = CsvRecord>,
        KF: Fn(&CsvRecord) -> K,
        VF: Fn(&CsvRecord) -> Option<f64>,
    {
        let mut agg = GroupedAggregator::new();
        for record in records {
            let key = key_fn(&record);
            if let Some(value) = value_fn(&record) {
                agg.update(key, value);
            }
        }
        agg
    }
}

impl<K: Eq + Hash + Clone> GroupedAggregator<K> {
    pub fn merge(&mut self, other: GroupedAggregator<K>) {
        for (key, stats) in other.groups {
            self.groups
                .entry(key)
                .or_insert_with(CsvAggregator::new)
                .merge(stats);
        }
    }
}

// =============================================================================
// Milestone 6: Parallel CSV processing
// =============================================================================

fn find_chunk_boundaries(file: &mut File, num_chunks: usize) -> io::Result<Vec<u64>> {
    let file_size = file.metadata()?.len();
    let num_chunks = num_chunks.max(1);
    if file_size == 0 {
        return Ok(vec![0, 0]);
    }

    let chunk_size = (file_size / num_chunks as u64).max(1);
    let mut boundaries = vec![0];
    let mut next = chunk_size;

    while next < file_size {
        file.seek(SeekFrom::Start(next))?;
        let mut buf = [0u8; 1];
        let mut pos = next;
        loop {
            match file.read(&mut buf) {
                Ok(0) => {
                    pos = file_size;
                    break;
                }
                Ok(1) => {
                    pos += 1;
                    if buf[0] == b'\n' {
                        break;
                    }
                }
                Ok(_) => unreachable!(),
                Err(err) => return Err(err),
            }
            if pos >= file_size {
                pos = file_size;
                break;
            }
        }
        if pos >= file_size {
            break;
        }
        boundaries.push(pos);
        next = pos + chunk_size;
    }

    if *boundaries.last().unwrap() != file_size {
        boundaries.push(file_size);
    }

    Ok(boundaries)
}

pub fn parallel_process_csv<F, R>(
    path: &Path,
    delimiter: char,
    num_workers: usize,
    process_chunk: F,
) -> io::Result<Vec<R>>
where
    F: Fn(Vec<CsvRecord>) -> R + Send + Sync + 'static,
    R: Send,
{
    let num_workers = num_workers.max(1);
    let path_buf = Arc::new(path.to_path_buf());
    let mut file = File::open(&*path_buf)?;
    let boundaries = find_chunk_boundaries(&mut file, num_workers)?;
    let chunk_ranges: Vec<(u64, u64)> = boundaries
        .windows(2)
        .map(|window| (window[0], window[1]))
        .collect();
    let process_fn = Arc::new(process_chunk);

    chunk_ranges
        .into_par_iter()
        .map({
            let path_buf = Arc::clone(&path_buf);
            let processor = Arc::clone(&process_fn);
            move |(start, end)| -> io::Result<R> {
                let mut chunk_file = File::open(&*path_buf)?;
                chunk_file.seek(SeekFrom::Start(start))?;
                let mut reader = BufReader::new(chunk_file);
                let mut consumed = start;
                let mut records = Vec::new();
                let mut line = String::new();

                while consumed < end {
                    line.clear();
                    let bytes_read = reader.read_line(&mut line)?;
                    if bytes_read == 0 {
                        break;
                    }
                    consumed += bytes_read as u64;
                    let trimmed = line.trim_end_matches(|c| c == '\n' || c == '\r');
                    if trimmed.trim().is_empty() {
                        continue;
                    }
                    match CsvRecord::parse_csv_line(trimmed, delimiter) {
                        Ok(record) => records.push(record),
                        Err(ParseError::EmptyLine) => continue,
                        Err(err) => {
                            return Err(io::Error::new(
                                io::ErrorKind::InvalidData,
                                format!("Failed to parse chunk at byte {}: {:?}", consumed, err),
                            ))
                        }
                    }
                }

                Ok((*processor)(records))
            }
        })
        .collect()
}

pub fn parallel_aggregate_column(
    path: &Path,
    delimiter: char,
    column: usize,
    num_workers: usize,
) -> io::Result<CsvAggregator> {
    let aggregates = parallel_process_csv(path, delimiter, num_workers, move |records| {
        records
            .into_iter()
            .fold(CsvAggregator::new(), |mut agg, record| {
                if let Ok(value) = record.get_typed::<f64>(column) {
                    agg.update(value);
                }
                agg
            })
    })?;

    let mut final_agg = CsvAggregator::new();
    for agg in aggregates {
        final_agg.merge(agg);
    }
    Ok(final_agg)
}

pub fn parallel_group_by<K>(
    path: &Path,
    delimiter: char,
    key_column: usize,
    value_column: usize,
    num_workers: usize,
) -> io::Result<GroupedAggregator<K>>
where
    K: Eq + Hash + Send + Clone + FromCsvField + 'static,
{
    let grouped_results = parallel_process_csv(path, delimiter, num_workers, move |records| {
        records
            .into_iter()
            .fold(GroupedAggregator::new(), |mut agg, record| {
                if let (Ok(key), Ok(value)) = (
                    record.get_typed::<K>(key_column),
                    record.get_typed::<f64>(value_column),
                ) {
                    agg.update(key, value);
                }
                agg
            })
    })?;

    let mut final_grouped = GroupedAggregator::new();
    for grouped in grouped_results {
        final_grouped.merge(grouped);
    }
    Ok(final_grouped)
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn create_test_csv(content: &str) -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(content.as_bytes()).unwrap();
        file
    }

    fn create_large_test_csv(rows: usize) -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "category,value").unwrap();
        for i in 0..rows {
            let category = match i % 3 {
                0 => "A",
                1 => "B",
                _ => "C",
            };
            writeln!(file, "{},{}", category, i).unwrap();
        }
        file
    }

    #[test]
    fn test_simple_csv_parsing() {
        let record = CsvRecord::parse_csv_line("foo,bar,baz", ',').unwrap();
        assert_eq!(record.field_count(), 3);
        assert_eq!(record.get_field(0), Some("foo"));
        assert_eq!(record.get_field(1), Some("bar"));
        assert_eq!(record.get_field(2), Some("baz"));
    }

    #[test]
    fn test_quoted_fields() {
        let record = CsvRecord::parse_csv_line(r#"foo,"bar,baz",qux"#, ',').unwrap();
        assert_eq!(record.field_count(), 3);
        assert_eq!(record.get_field(1), Some("bar,baz"));
    }

    #[test]
    fn test_escaped_quotes() {
        let record = CsvRecord::parse_csv_line(r#""foo ""bar"" baz""#, ',').unwrap();
        assert_eq!(record.get_field(0), Some(r#"foo "bar" baz"#));
    }

    #[test]
    fn test_empty_fields() {
        let record = CsvRecord::parse_csv_line("foo,,bar", ',').unwrap();
        assert_eq!(record.field_count(), 3);
        assert_eq!(record.get_field(1), Some(""));
    }

    #[test]
    fn test_custom_delimiter() {
        let record = CsvRecord::parse_csv_line("foo|bar|baz", '|').unwrap();
        assert_eq!(record.field_count(), 3);
    }

    #[test]
    fn test_iterate_simple_csv() {
        let file = create_test_csv("a,b,c\n1,2,3\n4,5,6");
        let mut iter = CsvFileIterator::new(file.path(), ',').unwrap();

        let record1 = iter.next().unwrap().unwrap();
        assert_eq!(record1.get_field(0), Some("a"));

        let record2 = iter.next().unwrap().unwrap();
        assert_eq!(record2.get_field(0), Some("1"));

        let record3 = iter.next().unwrap().unwrap();
        assert_eq!(record3.get_field(0), Some("4"));

        assert!(iter.next().is_none());
    }

    #[test]
    fn test_skip_empty_lines() {
        let file = create_test_csv("a,b\n\n1,2\n");
        let records: Vec<_> = CsvFileIterator::new(file.path(), ',')
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

        assert_eq!(records.len(), 2);
    }

    #[test]
    fn test_error_reporting_with_line_numbers() {
        let file = create_test_csv("a,b\n\"unterminated\n3,4");
        let mut iter = CsvFileIterator::new(file.path(), ',').unwrap();

        iter.next();
        let err = iter.next().unwrap().unwrap_err();

        match err {
            Error::Parse(ParseError::UnterminatedQuote, line) => assert_eq!(line, 2),
            _ => panic!("Expected parse error"),
        }
    }

    #[test]
    fn test_extract_integers() {
        let record = CsvRecord::parse_csv_line("100,200,300", ',').unwrap();
        assert_eq!(record.get_typed::<i64>(0).unwrap(), 100);
        assert_eq!(record.get_typed::<i64>(1).unwrap(), 200);
    }

    #[test]
    fn test_extract_floats() {
        let record = CsvRecord::parse_csv_line("3.14,2.71,1.41", ',').unwrap();
        assert_eq!(record.get_typed::<f64>(0).unwrap(), 3.14);
    }

    #[test]
    fn test_extract_booleans() {
        let record = CsvRecord::parse_csv_line("true,false,yes,no,1,0", ',').unwrap();
        assert_eq!(record.get_typed::<bool>(0).unwrap(), true);
        assert_eq!(record.get_typed::<bool>(1).unwrap(), false);
        assert_eq!(record.get_typed::<bool>(2).unwrap(), true);
        assert_eq!(record.get_typed::<bool>(3).unwrap(), false);
        assert_eq!(record.get_typed::<bool>(4).unwrap(), true);
        assert_eq!(record.get_typed::<bool>(5).unwrap(), false);
    }

    #[test]
    fn test_conversion_errors() {
        let record = CsvRecord::parse_csv_line("not_a_number,42", ',').unwrap();
        assert!(record.get_typed::<i64>(0).is_err());
        assert!(record.get_typed::<i64>(1).is_ok());
    }

    #[test]
    fn test_missing_field_error() {
        let record = CsvRecord::parse_csv_line("a,b", ',').unwrap();
        assert!(matches!(
            record.get_typed::<String>(5),
            Err(ConversionError::MissingField)
        ));
    }

    #[test]
    fn test_filter_by_column() {
        let csv = "status,amount\ncompleted,100\npending,200\ncompleted,300";
        let file = create_test_csv(csv);

        let completed: Vec<_> = CsvFileIterator::new(file.path(), ',')
            .unwrap()
            .skip(1)
            .filter_by_column(0, |status| status == "completed")
            .filter_valid()
            .collect();

        assert_eq!(completed.len(), 2);
        assert_eq!(completed[0].get_field(1), Some("100"));
        assert_eq!(completed[1].get_field(1), Some("300"));
    }

    #[test]
    fn test_map_column_transformation() {
        let csv = "name,age\nalice,30\nbob,25";
        let file = create_test_csv(csv);

        let uppercase: Vec<_> = CsvFileIterator::new(file.path(), ',')
            .unwrap()
            .filter_valid()
            .map_column(0, |name| name.to_uppercase())
            .collect();

        assert_eq!(uppercase[1].get_field(0), Some("ALICE"));
        assert_eq!(uppercase[2].get_field(0), Some("BOB"));
    }

    #[test]
    fn test_chained_operations() {
        let csv = "status,amount\ncompleted,100\npending,200\ncompleted,50\nfailed,75";
        let file = create_test_csv(csv);

        let result: Vec<_> = CsvFileIterator::new(file.path(), ',')
            .unwrap()
            .skip(1)
            .filter_by_column(0, |s| s == "completed")
            .filter_valid()
            .collect();

        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_basic_aggregation() {
        let csv = "value\n10\n20\n30\n40\n50";
        let file = create_test_csv(csv);

        let agg = CsvAggregator::from_column(
            CsvFileIterator::new(file.path(), ',')
                .unwrap()
                .skip(1)
                .filter_map(Result::ok),
            0,
        );

        assert_eq!(agg.count, 5);
        assert_eq!(agg.sum, 150.0);
        assert_eq!(agg.min, 10.0);
        assert_eq!(agg.max, 50.0);
        assert_eq!(agg.mean(), Some(30.0));
    }

    #[test]
    fn test_grouped_aggregation() {
        let csv = "category,amount\nA,100\nB,200\nA,150\nB,250\nA,50";
        let file = create_test_csv(csv);

        let grouped = GroupedAggregator::from_records(
            CsvFileIterator::new(file.path(), ',')
                .unwrap()
                .skip(1)
                .filter_map(Result::ok),
            |rec| rec.get_field(0).unwrap().to_string(),
            |rec| rec.get_typed::<f64>(1).ok(),
        );

        let stats_a = grouped.get(&"A".to_string()).unwrap();
        assert_eq!(stats_a.count, 3);
        assert_eq!(stats_a.sum, 300.0);

        let stats_b = grouped.get(&"B".to_string()).unwrap();
        assert_eq!(stats_b.count, 2);
        assert_eq!(stats_b.sum, 450.0);
    }

    #[test]
    fn test_empty_aggregation() {
        let agg = CsvAggregator::new();
        assert_eq!(agg.mean(), None);
    }

    #[test]
    fn test_parallel_vs_sequential_correctness() {
        let file = create_large_test_csv(1000);

        let seq_agg = CsvAggregator::from_column(
            CsvFileIterator::new(file.path(), ',')
                .unwrap()
                .skip(1)
                .filter_map(Result::ok),
            1,
        );

        let par_agg = parallel_aggregate_column(file.path(), ',', 1, 4).unwrap();

        assert_eq!(seq_agg.count, par_agg.count);
        assert_eq!(seq_agg.sum, par_agg.sum);
        assert_eq!(seq_agg.min, par_agg.min);
        assert_eq!(seq_agg.max, par_agg.max);
    }

    #[test]
    fn test_parallel_grouped_aggregation() {
        let file = create_large_test_csv(900);

        let grouped = parallel_group_by::<String>(file.path(), ',', 0, 1, 4).unwrap();

        assert_eq!(grouped.get(&"A".to_string()).unwrap().count, 300);
        assert_eq!(grouped.get(&"B".to_string()).unwrap().count, 300);
        assert_eq!(grouped.get(&"C".to_string()).unwrap().count, 300);
    }

    #[test]
    #[ignore]
    fn benchmark_parallel_speedup() {
        use std::time::Instant;

        let file = create_large_test_csv(100_000);

        let start = Instant::now();
        let _ = CsvAggregator::from_column(
            CsvFileIterator::new(file.path(), ',')
                .unwrap()
                .skip(1)
                .filter_map(Result::ok),
            1,
        );
        let seq_time = start.elapsed();

        let start = Instant::now();
        let _ = parallel_aggregate_column(file.path(), ',', 1, 4).unwrap();
        let par_time = start.elapsed();

        assert!(par_time < seq_time || par_time.as_secs_f64() == 0.0);
    }
}

fn main() {}