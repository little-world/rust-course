//! Pattern 4a: Streaming Algorithms
//! Example: CSV Parsing with Streaming
//!
//! Run with: cargo run --example p4a_csv_parsing

use std::io::BufRead;

/// Parse CSV data lazily from a reader.
/// Each line is processed as it's read, no full file load.
fn parse_csv_stream(reader: impl BufRead) -> impl Iterator<Item = Vec<String>> {
    reader.lines().filter_map(Result::ok).map(|line| {
        line.split(',')
            .map(|s| s.trim().to_string())
            .collect()
    })
}

/// Parse CSV with header extraction.
fn parse_csv_with_header(
    reader: impl BufRead,
) -> (Option<Vec<String>>, impl Iterator<Item = Vec<String>>) {
    let mut lines = reader.lines().filter_map(Result::ok);

    let header = lines.next().map(|line| {
        line.split(',')
            .map(|s| s.trim().to_string())
            .collect()
    });

    let rows = lines.map(|line| {
        line.split(',')
            .map(|s| s.trim().to_string())
            .collect()
    });

    (header, rows)
}

/// Convert CSV rows to named records using header.
fn csv_to_records(
    reader: impl BufRead,
) -> impl Iterator<Item = std::collections::HashMap<String, String>> {
    let mut lines = reader.lines().filter_map(Result::ok);

    let header: Vec<String> = lines
        .next()
        .map(|line| {
            line.split(',')
                .map(|s| s.trim().to_string())
                .collect()
        })
        .unwrap_or_default();

    lines.map(move |line| {
        let values: Vec<String> = line
            .split(',')
            .map(|s| s.trim().to_string())
            .collect();

        header
            .iter()
            .cloned()
            .zip(values)
            .collect()
    })
}

fn main() {
    println!("=== CSV Parsing with Streaming ===\n");

    // Usage: parse CSV data lazily from a reader
    let csv_data = "a,b,c\n1,2,3\n4,5,6";
    let reader = std::io::Cursor::new(csv_data);
    println!("CSV data:\n{}\n", csv_data);

    println!("Parsed rows:");
    for row in parse_csv_stream(reader) {
        println!("  {:?}", row);
    }

    println!("\n=== CSV with Header ===");
    let csv_with_header = "name,age,city\nAlice,30,NYC\nBob,25,LA\nCarol,35,Chicago";
    println!("CSV data:\n{}\n", csv_with_header);

    let reader2 = std::io::Cursor::new(csv_with_header);
    let (header, rows) = parse_csv_with_header(reader2);

    println!("Header: {:?}", header);
    println!("Data rows:");
    for row in rows {
        println!("  {:?}", row);
    }

    println!("\n=== CSV to Named Records ===");
    let reader3 = std::io::Cursor::new(csv_with_header);
    println!("Records:");
    for record in csv_to_records(reader3) {
        println!("  {:?}", record);
    }

    println!("\n=== Processing Large CSV (Simulated) ===");
    // Generate a "large" CSV in memory
    let mut large_csv = String::from("id,value\n");
    for i in 1..=100 {
        large_csv.push_str(&format!("{},{}\n", i, i * 10));
    }

    let reader4 = std::io::Cursor::new(large_csv);
    let sum: i32 = parse_csv_stream(reader4)
        .skip(1) // Skip header
        .filter_map(|row| row.get(1).and_then(|v| v.parse::<i32>().ok()))
        .sum();

    println!("Sum of 'value' column (1-100 * 10): {}", sum);
    // Sum of 10+20+30+...+1000 = 10 * (1+2+...+100) = 10 * 5050 = 50500

    println!("\n=== Filtering CSV Rows ===");
    let reader5 = std::io::Cursor::new("name,age,city\nAlice,30,NYC\nBob,25,LA\nCarol,35,Chicago");

    let adults: Vec<_> = csv_to_records(reader5)
        .filter(|r| {
            r.get("age")
                .and_then(|a| a.parse::<i32>().ok())
                .map(|age| age >= 30)
                .unwrap_or(false)
        })
        .collect();

    println!("People age >= 30:");
    for person in adults {
        println!("  {:?}", person);
    }

    println!("\n=== Memory Efficiency ===");
    println!("Only one line is in memory at a time!");
    println!("Can process GB-sized CSV files with constant memory.");

    println!("\n=== Key Points ===");
    println!("1. BufRead::lines() streams lazily");
    println!("2. filter_map(Result::ok) handles I/O errors");
    println!("3. Each row parsed and processed independently");
    println!("4. Memory proportional to line length, not file size");
}
