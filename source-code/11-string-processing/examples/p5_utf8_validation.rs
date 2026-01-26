//! Pattern 5: UTF-8 Validation and Repair
//! Lossy and Strict UTF-8 Validation
//!
//! Run with: cargo run --example p5_utf8_validation

fn main() {
    println!("=== UTF-8 Validation and Repair ===\n");

    // Valid UTF-8
    println!("=== Valid UTF-8 ===\n");

    let valid = "Hello, 世界!".as_bytes();
    let validator = Utf8Validator::new(valid);

    match validator.validate() {
        Ok(s) => println!("Valid UTF-8: '{}'", s),
        Err(e) => println!("Invalid at position {}", e.valid_up_to),
    }

    // Invalid UTF-8
    println!("\n=== Invalid UTF-8 ===\n");

    let invalid = &[0x48, 0x65, 0x6C, 0x6C, 0x6F, 0xFF, 0xFE];  // "Hello" + invalid bytes
    let validator = Utf8Validator::new(invalid);

    match validator.validate() {
        Ok(s) => println!("Valid UTF-8: '{}'", s),
        Err(e) => println!("Invalid UTF-8 at position {}, error_len: {:?}", e.valid_up_to, e.error_len),
    }

    // Lossy conversion
    println!("\n=== Lossy Conversion ===\n");

    let lossy = validate_utf8_lossy(invalid);
    println!("Lossy conversion of invalid bytes: '{}'", lossy);

    // Standard library validation
    println!("\n=== Standard Library Validation ===\n");

    match std::str::from_utf8(valid) {
        Ok(s) => println!("std::str::from_utf8 (valid): Ok('{}')", s),
        Err(e) => println!("std::str::from_utf8 (valid): Err({:?})", e),
    }

    match std::str::from_utf8(invalid) {
        Ok(s) => println!("std::str::from_utf8 (invalid): Ok('{}')", s),
        Err(e) => println!("std::str::from_utf8 (invalid): Err(valid_up_to: {})", e.valid_up_to()),
    }

    // UTF-8 structure demonstration
    println!("\n=== UTF-8 Structure ===\n");

    let examples = [
        ("ASCII", "A"),
        ("2-byte", "é"),
        ("3-byte", "世"),
        ("4-byte", "𝄞"),
    ];

    for (desc, s) in examples {
        let bytes: Vec<u8> = s.bytes().collect();
        println!("{} '{}': {} bytes -> {:02X?}", desc, s, bytes.len(), bytes);
    }

    println!("\n=== Key Points ===");
    println!("1. UTF-8 is variable-length: 1-4 bytes per character");
    println!("2. from_utf8() for strict validation, from_utf8_lossy() for recovery");
    println!("3. Overlong encodings are security risks - detect and reject");
    println!("4. Always validate external data before treating as UTF-8");
}

fn validate_utf8_lossy(data: &[u8]) -> String {
    String::from_utf8_lossy(data).into_owned()
}

fn validate_utf8_strict(data: &[u8]) -> Result<&str, std::str::Utf8Error> {
    std::str::from_utf8(data)
}

#[derive(Debug)]
struct Utf8Error {
    valid_up_to: usize,
    error_len: Option<usize>,
}

struct Utf8Validator<'a> {
    data: &'a [u8],
}

impl<'a> Utf8Validator<'a> {
    fn new(data: &'a [u8]) -> Self {
        Utf8Validator { data }
    }

    fn validate(&self) -> Result<&'a str, Utf8Error> {
        let mut pos = 0;

        while pos < self.data.len() {
            match self.decode_char(pos) {
                Ok((_, next_pos)) => pos = next_pos,
                Err(error_pos) => {
                    return Err(Utf8Error {
                        valid_up_to: error_pos,
                        error_len: self.error_length(error_pos),
                    });
                }
            }
        }

        unsafe { Ok(std::str::from_utf8_unchecked(self.data)) }
    }

    fn decode_char(&self, pos: usize) -> Result<(char, usize), usize> {
        if pos >= self.data.len() {
            return Err(pos);
        }

        let first = self.data[pos];

        // 1-byte sequence (ASCII)
        if first < 0x80 {
            return Ok((first as char, pos + 1));
        }

        // 2-byte sequence
        if first & 0xE0 == 0xC0 {
            if pos + 1 >= self.data.len() {
                return Err(pos);
            }
            let second = self.data[pos + 1];
            if second & 0xC0 != 0x80 {
                return Err(pos);
            }
            let ch = ((first as u32 & 0x1F) << 6)
                | (second as u32 & 0x3F);
            if ch < 0x80 {
                return Err(pos);  // Overlong encoding
            }
            return Ok((char::from_u32(ch).ok_or(pos)?, pos + 2));
        }

        // 3-byte sequence
        if first & 0xF0 == 0xE0 {
            if pos + 2 >= self.data.len() {
                return Err(pos);
            }
            let second = self.data[pos + 1];
            let third = self.data[pos + 2];
            if second & 0xC0 != 0x80 || third & 0xC0 != 0x80 {
                return Err(pos);
            }
            let ch = ((first as u32 & 0x0F) << 12)
                | ((second as u32 & 0x3F) << 6)
                | (third as u32 & 0x3F);
            if ch < 0x800 {
                return Err(pos);  // Overlong encoding
            }
            return Ok((char::from_u32(ch).ok_or(pos)?, pos + 3));
        }

        // 4-byte sequence
        if first & 0xF8 == 0xF0 {
            if pos + 3 >= self.data.len() {
                return Err(pos);
            }
            let bytes = &self.data[pos..pos + 4];
            if bytes[1] & 0xC0 != 0x80
                || bytes[2] & 0xC0 != 0x80
                || bytes[3] & 0xC0 != 0x80
            {
                return Err(pos);
            }
            let ch = ((first as u32 & 0x07) << 18)
                | ((bytes[1] as u32 & 0x3F) << 12)
                | ((bytes[2] as u32 & 0x3F) << 6)
                | (bytes[3] as u32 & 0x3F);
            if ch < 0x10000 || ch > 0x10FFFF {
                return Err(pos);  // Overlong or out of range
            }
            return Ok((char::from_u32(ch).ok_or(pos)?, pos + 4));
        }

        Err(pos)
    }

    fn error_length(&self, pos: usize) -> Option<usize> {
        if pos >= self.data.len() {
            return None;
        }

        let first = self.data[pos];
        if first < 0x80 {
            Some(1)
        } else if first & 0xE0 == 0xC0 {
            Some(2)
        } else if first & 0xF0 == 0xE0 {
            Some(3)
        } else if first & 0xF8 == 0xF0 {
            Some(4)
        } else {
            Some(1)
        }
    }
}
