// Pattern 2: Codecs for Framing
use bytes::{Buf, BufMut, BytesMut};
use std::io;
use tokio_util::codec::{Decoder, Encoder};

// Custom codec for length-prefixed messages
struct LengthPrefixedCodec;

impl Decoder for LengthPrefixedCodec {
    type Item = Vec<u8>;
    type Error = io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        // Need at least 4 bytes for the length prefix
        if src.len() < 4 {
            return Ok(None); // Need more data
        }

        // Read the length prefix (big-endian u32)
        let mut length_bytes = [0u8; 4];
        length_bytes.copy_from_slice(&src[..4]);
        let length = u32::from_be_bytes(length_bytes) as usize;

        // Check if we have the complete message
        if src.len() < 4 + length {
            return Ok(None); // Need more data
        }

        // We have a complete message—extract it
        src.advance(4);  // Skip the length prefix
        let data = src.split_to(length).to_vec();
        Ok(Some(data))
    }
}

impl Encoder<Vec<u8>> for LengthPrefixedCodec {
    type Error = io::Error;

    fn encode(&mut self, item: Vec<u8>, dst: &mut BytesMut) -> Result<(), Self::Error> {
        // Write length prefix
        let length = item.len() as u32;
        dst.put_u32(length);

        // Write message data
        dst.put_slice(&item);
        Ok(())
    }
}

// Simple line codec implementation (for demonstration)
struct SimpleLineCodec;

impl Decoder for SimpleLineCodec {
    type Item = String;
    type Error = io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        // Find newline
        if let Some(pos) = src.iter().position(|&b| b == b'\n') {
            // Extract the line (without newline)
            let line = src.split_to(pos);
            src.advance(1); // Skip the newline

            // Convert to string
            let s = String::from_utf8(line.to_vec())
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
            Ok(Some(s))
        } else {
            Ok(None) // No complete line yet
        }
    }
}

impl Encoder<String> for SimpleLineCodec {
    type Error = io::Error;

    fn encode(&mut self, item: String, dst: &mut BytesMut) -> Result<(), Self::Error> {
        dst.put_slice(item.as_bytes());
        dst.put_u8(b'\n');
        Ok(())
    }
}

// Demonstrate codec usage without network
fn demo_length_prefixed_codec() {
    println!("=== Length-Prefixed Codec Demo ===");

    let mut codec = LengthPrefixedCodec;
    let mut buffer = BytesMut::new();

    // Encode some messages
    let messages = vec![
        b"Hello, World!".to_vec(),
        b"Short".to_vec(),
        b"A longer message with more content".to_vec(),
    ];

    for msg in &messages {
        codec.encode(msg.clone(), &mut buffer).unwrap();
        println!("Encoded: {:?} ({} bytes + 4 byte prefix)", String::from_utf8_lossy(msg), msg.len());
    }

    println!("\nBuffer size after encoding: {} bytes", buffer.len());

    // Decode the messages
    println!("\nDecoding messages:");
    while let Some(decoded) = codec.decode(&mut buffer).unwrap() {
        println!("Decoded: {:?} ({} bytes)", String::from_utf8_lossy(&decoded), decoded.len());
    }

    println!("Remaining buffer: {} bytes", buffer.len());
}

fn demo_line_codec() {
    println!("\n=== Line Codec Demo ===");

    let mut codec = SimpleLineCodec;
    let mut buffer = BytesMut::new();

    // Encode some lines
    let lines = vec![
        "First line".to_string(),
        "Second line".to_string(),
        "Third line with more text".to_string(),
    ];

    for line in &lines {
        codec.encode(line.clone(), &mut buffer).unwrap();
        println!("Encoded: {:?}", line);
    }

    println!("\nBuffer content: {:?}", String::from_utf8_lossy(&buffer));

    // Decode the lines
    println!("\nDecoding lines:");
    while let Some(decoded) = codec.decode(&mut buffer).unwrap() {
        println!("Decoded: {:?}", decoded);
    }
}

fn demo_partial_decode() {
    println!("\n=== Partial Decode Demo ===");

    let mut codec = LengthPrefixedCodec;
    let mut buffer = BytesMut::new();

    // Encode a message
    let msg = b"Complete message".to_vec();
    codec.encode(msg.clone(), &mut buffer).unwrap();

    // Simulate partial data arrival
    let full_data = buffer.split();
    buffer.clear();

    println!("Simulating partial data arrival...");

    // First, only 2 bytes arrive (partial length prefix)
    buffer.extend_from_slice(&full_data[..2]);
    let result = codec.decode(&mut buffer).unwrap();
    println!("After 2 bytes: {:?} (waiting for more)", result);

    // Next, 2 more bytes (complete length prefix, but no data)
    buffer.extend_from_slice(&full_data[2..4]);
    let result = codec.decode(&mut buffer).unwrap();
    println!("After 4 bytes: {:?} (have length, waiting for data)", result);

    // Finally, the rest of the message
    buffer.extend_from_slice(&full_data[4..]);
    let result = codec.decode(&mut buffer).unwrap();
    println!("After all bytes: {:?}", result.map(|d| String::from_utf8_lossy(&d).to_string()));
}

#[tokio::main]
async fn main() {
    println!("=== Codec Demo ===\n");

    demo_length_prefixed_codec();
    demo_line_codec();
    demo_partial_decode();

    println!("\nCodec demo completed");
}
