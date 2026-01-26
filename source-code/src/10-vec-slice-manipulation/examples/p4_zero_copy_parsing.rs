//! Pattern 4: Zero-Copy Slicing
//! Example: Zero-Copy Parsing
//!
//! Run with: cargo run --example p4_zero_copy_parsing

fn main() {
    println!("=== Zero-Copy Parsing ===\n");

    // Return slices instead of cloning
    println!("=== CSV Field Extraction (Zero-Copy) ===\n");

    fn find_field<'a>(record: &'a [u8], field_index: usize) -> &'a [u8] {
        let mut start = 0;
        let mut current_field = 0;

        for (i, &byte) in record.iter().enumerate() {
            if byte == b',' {
                if current_field == field_index {
                    return &record[start..i];
                }
                current_field += 1;
                start = i + 1;
            }
        }

        if current_field == field_index {
            &record[start..]
        } else {
            &[]
        }
    }

    let record = b"Alice,30,Engineer,New York";
    println!("Record: {}", String::from_utf8_lossy(record));

    for i in 0..5 {
        let field = find_field(record, i);
        println!("  Field {}: '{}'", i, String::from_utf8_lossy(field));
    }

    // Split without allocation
    println!("\n=== Split Without Allocation ===\n");

    fn parse_csv_line(line: &str) -> Vec<&str> {
        line.split(',')
            .map(|s| s.trim())
            .collect()
    }

    let line = "  field1 ,  field2  , field3,field4  ";
    println!("Line: '{}'", line);

    let fields = parse_csv_line(line);
    println!("Parsed fields: {:?}", fields);

    // Multiple slices from one allocation
    println!("\n=== Frame Parsing (Multiple Slices) ===\n");

    #[derive(Debug)]
    enum ParseError {
        TooShort,
    }

    struct Frame<'a> {
        header: &'a [u8],
        payload: &'a [u8],
        checksum: &'a [u8],
    }

    impl<'a> Frame<'a> {
        fn parse(data: &'a [u8]) -> Result<Self, ParseError> {
            if data.len() < 10 {
                return Err(ParseError::TooShort);
            }

            Ok(Frame {
                header: &data[0..4],
                payload: &data[4..data.len() - 4],
                checksum: &data[data.len() - 4..],
            })
        }
    }

    let packet = b"HDR_Hello, World!CSUM";
    println!("Packet ({} bytes): {:?}", packet.len(), packet);

    match Frame::parse(packet) {
        Ok(frame) => {
            println!("Parsed frame:");
            println!("  Header:   {:?}", frame.header);
            println!("  Payload:  {:?} ('{}')",
                frame.payload, String::from_utf8_lossy(frame.payload));
            println!("  Checksum: {:?}", frame.checksum);
        }
        Err(e) => println!("Parse error: {:?}", e),
    }

    // split_at for header/body separation
    println!("\n=== Split At for Header/Body ===\n");

    const HEADER_SIZE: usize = 8;

    fn parse_message(data: &[u8]) -> (&[u8], &[u8]) {
        if data.len() < HEADER_SIZE {
            return (&[], data);
        }
        data.split_at(HEADER_SIZE)
    }

    let message = b"HEADER01This is the message body";
    let (header, body) = parse_message(message);

    println!("Message: {:?}", message);
    println!("Header:  {:?} ('{}')", header, String::from_utf8_lossy(header));
    println!("Body:    {:?} ('{}')", body, String::from_utf8_lossy(body));

    // split_first and split_last
    println!("\n=== Split First/Last for Protocol Parsing ===\n");

    #[derive(Debug)]
    struct Packet<'a> {
        version: u8,
        payload: &'a [u8],
        checksum: u8,
    }

    #[derive(Debug)]
    enum PacketError {
        Empty,
        NoChecksum,
    }

    fn parse_packet(data: &[u8]) -> Result<Packet, PacketError> {
        let (&version, rest) = data.split_first()
            .ok_or(PacketError::Empty)?;

        // split_last returns (&last_element, &[rest])
        let (&checksum, payload) = rest.split_last()
            .ok_or(PacketError::NoChecksum)?;

        Ok(Packet { version, payload, checksum })
    }

    let data = b"\x01Hello\xFF";
    match parse_packet(data) {
        Ok(packet) => {
            println!("Parsed packet:");
            println!("  Version:  {}", packet.version);
            println!("  Payload:  '{}'", String::from_utf8_lossy(packet.payload));
            println!("  Checksum: 0x{:02X}", packet.checksum);
        }
        Err(e) => println!("Error: {:?}", e),
    }

    // Iterating without collecting
    println!("\n=== Process Without Collecting ===\n");

    fn sum_valid_numbers(data: &str) -> i32 {
        data.split(',')
            .filter_map(|s| s.trim().parse::<i32>().ok())
            .sum()
    }

    let data = "1, 2, abc, 3, , 4, xyz, 5";
    println!("Data: '{}'", data);
    println!("Sum of valid numbers: {}", sum_valid_numbers(data));

    println!("\n=== Key Points ===");
    println!("1. Return slices (&[T], &str) instead of owned types");
    println!("2. Use split, split_at, split_first/last for parsing");
    println!("3. Lifetime parameter ties slice to source data");
    println!("4. Zero allocation = 10-100x faster for parsing");
    println!("5. Process iterators directly without collect()");
}
