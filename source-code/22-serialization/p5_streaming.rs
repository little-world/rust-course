// Pattern 5: Streaming Serialization
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{self, BufRead, BufReader, BufWriter, Cursor, Write};

#[derive(Serialize, Deserialize, Debug)]
struct Record {
    id: u64,
    name: String,
    value: f64,
}

// Stream JSON array to writer
fn stream_json_array<W: Write>(mut writer: W, records: &[Record]) -> io::Result<()> {
    writer.write_all(b"[")?;

    for (i, record) in records.iter().enumerate() {
        if i > 0 {
            writer.write_all(b",")?;
        }

        let json =
            serde_json::to_string(record).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        writer.write_all(json.as_bytes())?;
    }

    writer.write_all(b"]")?;
    writer.flush()?;

    Ok(())
}

fn streaming_array_demo() -> io::Result<()> {
    println!("=== Streaming Array Demo ===\n");

    let records = vec![
        Record {
            id: 1,
            name: "Alice".to_string(),
            value: 100.0,
        },
        Record {
            id: 2,
            name: "Bob".to_string(),
            value: 200.0,
        },
        Record {
            id: 3,
            name: "Carol".to_string(),
            value: 300.0,
        },
    ];

    let mut output = Vec::new();
    stream_json_array(&mut output, &records)?;

    println!("Streamed JSON array:");
    println!("{}\n", String::from_utf8_lossy(&output));

    Ok(())
}

// JSON Lines format (newline-delimited JSON)
#[derive(Serialize, Deserialize, Debug)]
struct LogEntry {
    timestamp: u64,
    level: String,
    message: String,
}

fn json_lines_write_demo() -> io::Result<()> {
    println!("=== JSON Lines Write Demo ===\n");

    let entries = vec![
        LogEntry {
            timestamp: 1000,
            level: "INFO".to_string(),
            message: "Server started".to_string(),
        },
        LogEntry {
            timestamp: 1001,
            level: "DEBUG".to_string(),
            message: "Processing request".to_string(),
        },
        LogEntry {
            timestamp: 1002,
            level: "ERROR".to_string(),
            message: "Connection failed".to_string(),
        },
    ];

    let mut output = Vec::new();

    for entry in &entries {
        let json =
            serde_json::to_string(entry).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        writeln!(output, "{}", json)?;
    }

    println!("JSON Lines format:");
    println!("{}", String::from_utf8_lossy(&output));

    Ok(())
}

fn json_lines_read_demo() -> io::Result<()> {
    println!("=== JSON Lines Read Demo ===\n");

    // Simulated JSON Lines input
    let json_lines = r#"{"timestamp":1000,"level":"INFO","message":"Server started"}
{"timestamp":1001,"level":"DEBUG","message":"Processing request"}
{"timestamp":1002,"level":"ERROR","message":"Connection failed"}
{"timestamp":1003,"level":"WARN","message":"High memory usage"}"#;

    let reader = BufReader::new(Cursor::new(json_lines));

    println!("Processing JSON Lines (one at a time):");
    for (line_num, line) in reader.lines().enumerate() {
        let line = line?;
        match serde_json::from_str::<LogEntry>(&line) {
            Ok(entry) => {
                println!("  Line {}: [{:5}] {}", line_num, entry.level, entry.message);
            }
            Err(e) => {
                eprintln!("  Line {}: Parse error: {}", line_num, e);
            }
        }
    }

    println!();
    Ok(())
}

// Streaming deserializer for multiple JSON objects
fn streaming_deserializer_demo() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Streaming Deserializer Demo ===\n");

    #[derive(Deserialize, Debug)]
    struct Item {
        id: u64,
        name: String,
    }

    // Multiple JSON objects (whitespace-separated)
    let json = r#"
        {"id": 1, "name": "Item 1"}
        {"id": 2, "name": "Item 2"}
        {"id": 3, "name": "Item 3"}
    "#;

    let cursor = Cursor::new(json);
    let deserializer = serde_json::Deserializer::from_reader(cursor);

    println!("Streaming multiple JSON objects:");
    for item in deserializer.into_iter::<Item>() {
        match item {
            Ok(item) => println!("  {:?}", item),
            Err(e) => eprintln!("  Error: {}", e),
        }
    }

    println!();
    Ok(())
}

// Large dataset streaming writer
struct DataStreamWriter<W: Write> {
    writer: W,
    count: usize,
}

impl<W: Write> DataStreamWriter<W> {
    fn new(mut writer: W) -> io::Result<Self> {
        writer.write_all(b"[")?;
        Ok(DataStreamWriter { writer, count: 0 })
    }

    fn write_record<T: Serialize>(&mut self, record: &T) -> io::Result<()> {
        if self.count > 0 {
            self.writer.write_all(b",")?;
        }

        let json =
            serde_json::to_string(record).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        self.writer.write_all(json.as_bytes())?;
        self.count += 1;

        // Flush periodically to avoid memory buildup
        if self.count % 100 == 0 {
            self.writer.flush()?;
        }

        Ok(())
    }

    fn finish(mut self) -> io::Result<usize> {
        self.writer.write_all(b"]")?;
        self.writer.flush()?;
        Ok(self.count)
    }
}

#[derive(Serialize, Deserialize)]
struct DataPoint {
    x: f64,
    y: f64,
    timestamp: u64,
}

fn large_dataset_demo() -> io::Result<()> {
    println!("=== Large Dataset Streaming Demo ===\n");

    let mut output = Vec::new();
    let mut writer = DataStreamWriter::new(&mut output)?;

    // Stream 1000 points (in production, could be millions)
    println!("Streaming 1000 data points...");
    for i in 0..1000 {
        let point = DataPoint {
            x: i as f64,
            y: (i as f64 * 0.1).sin(),
            timestamp: i,
        };
        writer.write_record(&point)?;
    }

    let count = writer.finish()?;
    println!("Wrote {} records", count);
    println!("Total size: {} bytes", output.len());

    // Parse first few to verify
    let parsed: Vec<DataPoint> = serde_json::from_slice(&output)?;
    println!("Verified: parsed {} records back", parsed.len());

    println!();
    Ok(())
}

// Binary streaming with length-prefixed messages
struct BinaryStreamWriter<W: Write> {
    writer: W,
}

impl<W: Write> BinaryStreamWriter<W> {
    fn new(writer: W) -> Self {
        BinaryStreamWriter { writer }
    }

    fn write_record<T: Serialize>(&mut self, record: &T) -> io::Result<()> {
        let bytes =
            bincode::serialize(record).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        // Write length prefix (4 bytes, big-endian)
        let len = bytes.len() as u32;
        self.writer.write_all(&len.to_be_bytes())?;

        // Write data
        self.writer.write_all(&bytes)?;

        Ok(())
    }

    fn flush(&mut self) -> io::Result<()> {
        self.writer.flush()
    }
}

struct BinaryStreamReader<R: io::Read> {
    reader: R,
}

impl<R: io::Read> BinaryStreamReader<R> {
    fn new(reader: R) -> Self {
        BinaryStreamReader { reader }
    }

    fn read_record<T: for<'de> Deserialize<'de>>(&mut self) -> io::Result<Option<T>> {
        // Read length prefix
        let mut len_bytes = [0u8; 4];
        match self.reader.read_exact(&mut len_bytes) {
            Ok(()) => {}
            Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => return Ok(None),
            Err(e) => return Err(e),
        }

        let len = u32::from_be_bytes(len_bytes) as usize;

        // Read data
        let mut data = vec![0u8; len];
        self.reader.read_exact(&mut data)?;

        // Deserialize
        let record =
            bincode::deserialize(&data).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        Ok(Some(record))
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct Message {
    id: u64,
    content: String,
}

fn binary_streaming_demo() -> io::Result<()> {
    println!("=== Binary Streaming Demo ===\n");

    let mut buffer = Vec::new();

    // Write messages
    {
        let mut writer = BinaryStreamWriter::new(&mut buffer);

        for i in 0..5 {
            let msg = Message {
                id: i,
                content: format!("Message {}", i),
            };
            writer.write_record(&msg)?;
        }
        writer.flush()?;
    }

    println!("Wrote 5 messages ({} bytes total)", buffer.len());

    // Read messages back
    {
        let mut reader = BinaryStreamReader::new(Cursor::new(&buffer));

        println!("\nReading messages:");
        while let Some(msg) = reader.read_record::<Message>()? {
            println!("  {:?}", msg);
        }
    }

    println!();
    Ok(())
}

// File-based streaming example
fn file_streaming_demo() -> io::Result<()> {
    println!("=== File Streaming Demo ===\n");

    let temp_path = "/tmp/streaming_demo.jsonl";

    // Write to file
    {
        let file = File::create(temp_path)?;
        let mut writer = BufWriter::new(file);

        println!("Writing 100 log entries to file...");
        for i in 0..100 {
            let entry = LogEntry {
                timestamp: 1000 + i,
                level: if i % 10 == 0 { "ERROR" } else { "INFO" }.to_string(),
                message: format!("Log message {}", i),
            };

            let json = serde_json::to_string(&entry)
                .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
            writeln!(writer, "{}", json)?;
        }
        writer.flush()?;
    }

    // Read from file and process
    {
        let file = File::open(temp_path)?;
        let reader = BufReader::new(file);

        let mut error_count = 0;
        let mut total_count = 0;

        println!("Reading and processing file...");
        for line in reader.lines() {
            let line = line?;
            let entry: LogEntry = serde_json::from_str(&line)
                .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

            total_count += 1;
            if entry.level == "ERROR" {
                error_count += 1;
            }
        }

        println!("Processed {} entries, found {} errors", total_count, error_count);
    }

    // Cleanup
    std::fs::remove_file(temp_path)?;

    println!();
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Streaming Serialization Demo ===\n");

    streaming_array_demo()?;
    json_lines_write_demo()?;
    json_lines_read_demo()?;
    streaming_deserializer_demo()?;
    large_dataset_demo()?;
    binary_streaming_demo()?;
    file_streaming_demo()?;

    println!("Streaming serialization demo completed");
    Ok(())
}
