# Zero-Copy Protocol Parser

## Problem Statement

Build a high-performance binary message parser that:
- Parses structured binary protocols without heap allocation
- Uses reference binding modes to destructure nested data efficiently
- Implements `Cow<'a, str>` for optional string normalization
- Provides flexible APIs using `AsRef` and `Borrow` traits correctly
- Follows Rust's `as_`/`to_`/`into_` naming conventions
- Demonstrates borrow splitting for parsing nested structures
- Achieves zero-copy parsing where the parsed view borrows directly from input bytes

---

## Why Zero-Copy Parsing Matters

**Performance Impact**: Memory allocation is expensive:

| Operation | Time (approx) |
|-----------|---------------|
| L1 cache access | 1 ns |
| Heap allocation | 25-100 ns |
| Memory copy (1KB) | 250 ns |
| System call | 1000+ ns |

**Real-World Impact**:
- **High-frequency trading**: Every microsecond matters; zero-copy parsing of FIX/ITCH protocols
- **Network proxies**: Parse millions of requests/second without allocation pressure
- **Game servers**: Parse player input with minimal GC pauses
- **Embedded systems**: Parse sensor data with limited RAM

**The Allocation Problem**:
```rust
// BAD: Allocates for every field
struct ParsedMessage {
    name: String,        // heap allocation
    payload: Vec<u8>,    // heap allocation
    headers: Vec<String>, // multiple allocations
}

// GOOD: Borrows from input buffer
struct ParsedMessage<'a> {
    name: &'a str,       // points into input
    payload: &'a [u8],   // points into input
    headers: Vec<&'a str>, // only Vec allocates, strings borrow
}
```

---

## Protocol Format: Simple Binary Message Protocol (SBMP)

We'll parse a simple binary protocol designed to demonstrate reference patterns:

```
Message Layout:
┌─────────────────────────────────────────────────────────────────┐
│ Header (12 bytes fixed)                                         │
├─────────┬─────────┬─────────┬───────────────────────────────────┤
│ Magic   │ Version │ MsgType │ Payload Length                    │
│ 4 bytes │ 1 byte  │ 1 byte  │ 2 bytes (big-endian)              │
├─────────┴─────────┴─────────┴───────────────────────────────────┤
│ Flags (1 byte) │ Reserved (3 bytes)                             │
├─────────────────────────────────────────────────────────────────┤
│ Field Count (2 bytes, big-endian)                               │
├─────────────────────────────────────────────────────────────────┤
│ Fields (variable length, repeated Field Count times)            │
│ ┌─────────────────────────────────────────────────────────────┐ │
│ │ Field Header (4 bytes)                                      │ │
│ │ ┌──────────┬──────────┬────────────────────────────────────┐│ │
│ │ │ Field ID │ Type     │ Length                             ││ │
│ │ │ 1 byte   │ 1 byte   │ 2 bytes (big-endian)               ││ │
│ │ └──────────┴──────────┴────────────────────────────────────┘│ │
│ │ Field Data (Length bytes)                                   │ │
│ └─────────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────┘

Field Types:
  0x01 = UTF-8 String
  0x02 = Binary blob
  0x03 = Unsigned integer (1/2/4/8 bytes based on length)
  0x04 = Nested message

Message Types:
  0x01 = Request
  0x02 = Response
  0x03 = Event
  0x04 = Heartbeat

Flags (bit field):
  bit 0: Compressed
  bit 1: Encrypted
  bit 2: Requires ACK
  bits 3-7: Reserved
```

**Example Message (hex dump)**:
```
53 42 4D 50    // Magic: "SBMP"
01             // Version: 1
01             // MsgType: Request
00 1A          // Payload length: 26 bytes
05             // Flags: Compressed + Requires ACK
00 00 00       // Reserved
00 02          // Field count: 2

// Field 1: String field
01             // Field ID: 1
01             // Type: String
00 05          // Length: 5
68 65 6C 6C 6F // Data: "hello"

// Field 2: Integer field
02             // Field ID: 2
03             // Type: Integer
00 04          // Length: 4
00 00 01 00    // Data: 256 (big-endian)
```

---

## Key Concepts Explained

### 1. Reference Binding Modes

When pattern matching, Rust infers whether bindings should move or borrow based on the matched value's type:

```rust
struct Header {
    magic: [u8; 4],
    version: u8,
    msg_type: u8,
}

// Matching on &Header: bindings become references automatically
fn inspect_header(header: &Header) {
    let Header { magic, version, msg_type } = header;
    // magic: &[u8; 4], version: &u8, msg_type: &u8
    // All are references because we matched on &Header
}

// Matching on owned Header: bindings move
fn consume_header(header: Header) {
    let Header { magic, version, msg_type } = header;
    // magic: [u8; 4], version: u8, msg_type: u8
    // All are owned because we matched on Header
}

// Explicit ref keyword overrides
fn mixed_binding(header: Header) {
    let Header { ref magic, version, msg_type } = header;
    // magic: &[u8; 4] (borrowed via ref)
    // version: u8, msg_type: u8 (moved/copied)
}
```

**Why it matters for parsing**: When parsing from a byte buffer, we want to borrow slices rather than copy. Binding mode inference lets us write natural destructuring that automatically borrows.

### 2. Cow (Clone-on-Write)

`Cow<'a, B>` represents data that might be borrowed or owned:

```rust
use std::borrow::Cow;

enum Cow<'a, B: ToOwned + ?Sized> {
    Borrowed(&'a B),      // Just a reference, no allocation
    Owned(<B as ToOwned>::Owned), // Owned copy
}

// Common instantiations:
// Cow<'a, str>  - either &'a str or String
// Cow<'a, [u8]> - either &'a [u8] or Vec<u8>
```

**When to use Cow**:
```rust
// String that MIGHT need normalization
fn normalize_string(s: &str) -> Cow<'_, str> {
    if s.contains('\0') {
        // Must allocate to remove null bytes
        Cow::Owned(s.replace('\0', ""))
    } else {
        // No allocation needed
        Cow::Borrowed(s)
    }
}

// Most strings pass through without allocation
let clean = normalize_string("hello");      // Cow::Borrowed
let fixed = normalize_string("hel\0lo");    // Cow::Owned
```

**Why it matters for parsing**: Many parsed strings need no transformation. Cow lets us defer allocation until actually necessary.

### 3. AsRef vs Borrow

Both traits provide reference conversion, but with different semantics:

```rust
// AsRef: "can be viewed as"
// No semantic guarantees - just type conversion
pub trait AsRef<T: ?Sized> {
    fn as_ref(&self) -> &T;
}

// Borrow: "can be borrowed as, with equivalent semantics"
// MUST have same Hash, Eq, Ord behavior
pub trait Borrow<Borrowed: ?Sized> {
    fn borrow(&self) -> &Borrowed;
}
```

**The critical difference**:
```rust
use std::collections::HashMap;
use std::borrow::Borrow;

// HashMap::get uses Borrow, not AsRef
impl<K, V> HashMap<K, V> {
    pub fn get<Q>(&self, k: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    { /* ... */ }
}

// This works because String: Borrow<str>
// AND String and &str hash the same way
let mut map: HashMap<String, i32> = HashMap::new();
map.insert("key".to_string(), 42);
let value = map.get("key");  // &str lookup on String keys - works!

// If we used AsRef, this would be UNSOUND:
// String::as_ref() -> &[u8] has DIFFERENT hash than String!
```

**Why it matters for parsing**: When building lookup tables from parsed data, using `Borrow` correctly enables zero-allocation lookups.

### 4. Conversion Naming Conventions

Rust has strict conventions that encode ownership semantics:

| Prefix | Receiver | Returns | Allocates | Example |
|--------|----------|---------|-----------|---------|
| `as_` | `&self` | `&T` | Never | `as_bytes()`, `as_str()` |
| `to_` | `&self` | `T` | Usually | `to_string()`, `to_vec()` |
| `into_` | `self` | `T` | Rarely | `into_bytes()`, `into_inner()` |

```rust
struct ParsedField<'a> {
    data: &'a [u8],
}

impl<'a> ParsedField<'a> {
    // as_: Returns reference, O(1), no allocation
    fn as_bytes(&self) -> &[u8] {
        self.data
    }

    // to_: Returns owned, may allocate
    fn to_vec(&self) -> Vec<u8> {
        self.data.to_vec()  // Allocates!
    }

    // into_: Consumes self, transfers ownership
    fn into_owned(self) -> Vec<u8> {
        self.data.to_vec()
    }
}
```

**Why it matters**: Users can predict allocation behavior from method names alone.

### 5. Borrow Splitting

Rust allows multiple mutable borrows if they're provably disjoint:

```rust
struct Message<'a> {
    header: &'a [u8],
    payload: &'a [u8],
}

impl<'a> Message<'a> {
    // Returns mutable references to DIFFERENT parts
    fn split_mut(&mut self) -> (&mut &'a [u8], &mut &'a [u8]) {
        (&mut self.header, &mut self.payload)
    }
}

// For slices, use split_at / split_at_mut
fn parse_at_boundary(data: &[u8], mid: usize) -> (&[u8], &[u8]) {
    data.split_at(mid)  // Two non-overlapping slices
}
```

**Why it matters for parsing**: We can hand out references to different parts of a buffer simultaneously.

---

## Milestone 1: Basic Zero-Copy Header Parsing

**Goal**: Parse fixed-size headers using slice patterns and binding modes.

**Concepts**: Reference binding, slice patterns, explicit type annotations.

### Starter Code

```rust
/// Magic bytes identifying SBMP protocol
const MAGIC: [u8; 4] = *b"SBMP";

/// Minimum valid message size (header only)
const MIN_MESSAGE_SIZE: usize = 14;

/// Message types in the protocol
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum MessageType {
    Request = 0x01,
    Response = 0x02,
    Event = 0x03,
    Heartbeat = 0x04,
}

/// Flags packed into a single byte
#[derive(Debug, Clone, Copy, Default)]
pub struct Flags {
    pub compressed: bool,
    pub encrypted: bool,
    pub requires_ack: bool,
}

/// Parsed message header - borrows from input buffer
#[derive(Debug)]
pub struct Header<'a> {
    /// Reference to the magic bytes in the original buffer
    pub magic: &'a [u8; 4],
    pub version: u8,
    pub msg_type: MessageType,
    pub payload_length: u16,
    pub flags: Flags,
    /// Reference to reserved bytes (for forward compatibility)
    pub reserved: &'a [u8; 3],
}

/// Parse errors with context
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseError {
    BufferTooShort { expected: usize, actual: usize },
    InvalidMagic { found: [u8; 4] },
    InvalidMessageType { value: u8 },
    InvalidUtf8 { field_id: u8 },
    UnexpectedEndOfData { field_id: u8, expected: usize, remaining: usize },
}

// ============================================================
// TODO: Implement the following
// ============================================================

impl Flags {
    /// Parse flags from a single byte
    ///
    /// Bit layout:
    /// - bit 0: compressed
    /// - bit 1: encrypted
    /// - bit 2: requires_ack
    /// - bits 3-7: reserved (ignore)
    pub fn from_byte(byte: u8) -> Self {
        todo!("Extract flag bits from byte")
    }

    /// Convert flags back to byte representation
    pub fn to_byte(&self) -> u8 {
        todo!("Pack flags into a byte")
    }
}

impl MessageType {
    /// Parse message type from byte, returning error for invalid values
    pub fn from_byte(byte: u8) -> Result<Self, ParseError> {
        todo!("Match byte to MessageType variant")
    }
}

impl<'a> Header<'a> {
    /// Parse header from byte slice, borrowing data where possible
    ///
    /// This demonstrates:
    /// - Slice pattern matching with exact sizes
    /// - Reference binding (header borrows from input)
    /// - Zero-copy: magic and reserved point into original buffer
    pub fn parse(data: &'a [u8]) -> Result<Self, ParseError> {
        // Check minimum size
        if data.len() < MIN_MESSAGE_SIZE {
            return Err(ParseError::BufferTooShort {
                expected: MIN_MESSAGE_SIZE,
                actual: data.len(),
            });
        }

        todo!("Parse header fields from data slice")

        // Hints:
        // - Use data[0..4].try_into().unwrap() to get &[u8; 4]
        // - Use u16::from_be_bytes([data[i], data[i+1]]) for multi-byte integers
        // - The header struct should BORROW magic and reserved from data
    }

    /// Returns the total message size (header + payload)
    pub fn total_size(&self) -> usize {
        MIN_MESSAGE_SIZE + self.payload_length as usize
    }
}
```

### Tests for Milestone 1

```rust
#[cfg(test)]
mod tests {
    use super::*;

    fn sample_header() -> Vec<u8> {
        vec![
            0x53, 0x42, 0x4D, 0x50, // Magic: "SBMP"
            0x01,                   // Version: 1
            0x01,                   // MsgType: Request
            0x00, 0x1A,             // Payload length: 26
            0x05,                   // Flags: compressed + requires_ack
            0x00, 0x00, 0x00,       // Reserved
            0x00, 0x02,             // Field count: 2
        ]
    }

    #[test]
    fn test_flags_from_byte() {
        let flags = Flags::from_byte(0b00000101);
        assert!(flags.compressed);
        assert!(!flags.encrypted);
        assert!(flags.requires_ack);

        let flags = Flags::from_byte(0b00000010);
        assert!(!flags.compressed);
        assert!(flags.encrypted);
        assert!(!flags.requires_ack);
    }

    #[test]
    fn test_flags_roundtrip() {
        let original = Flags {
            compressed: true,
            encrypted: false,
            requires_ack: true,
        };
        let byte = original.to_byte();
        let parsed = Flags::from_byte(byte);
        assert_eq!(original.compressed, parsed.compressed);
        assert_eq!(original.encrypted, parsed.encrypted);
        assert_eq!(original.requires_ack, parsed.requires_ack);
    }

    #[test]
    fn test_message_type_parsing() {
        assert_eq!(MessageType::from_byte(0x01).unwrap(), MessageType::Request);
        assert_eq!(MessageType::from_byte(0x02).unwrap(), MessageType::Response);
        assert_eq!(MessageType::from_byte(0x03).unwrap(), MessageType::Event);
        assert_eq!(MessageType::from_byte(0x04).unwrap(), MessageType::Heartbeat);

        assert!(matches!(
            MessageType::from_byte(0x00),
            Err(ParseError::InvalidMessageType { value: 0x00 })
        ));
        assert!(matches!(
            MessageType::from_byte(0xFF),
            Err(ParseError::InvalidMessageType { value: 0xFF })
        ));
    }

    #[test]
    fn test_header_parse_success() {
        let data = sample_header();
        let header = Header::parse(&data).unwrap();

        assert_eq!(header.magic, b"SBMP");
        assert_eq!(header.version, 1);
        assert_eq!(header.msg_type, MessageType::Request);
        assert_eq!(header.payload_length, 26);
        assert!(header.flags.compressed);
        assert!(!header.flags.encrypted);
        assert!(header.flags.requires_ack);
    }

    #[test]
    fn test_header_borrows_from_input() {
        let data = sample_header();
        let header = Header::parse(&data).unwrap();

        // Verify that magic points into the original data
        let magic_ptr = header.magic.as_ptr();
        let data_ptr = data.as_ptr();
        assert_eq!(magic_ptr, data_ptr, "magic should borrow from input");

        // Verify reserved also borrows
        let reserved_ptr = header.reserved.as_ptr();
        let expected_reserved_ptr = unsafe { data_ptr.add(9) };
        assert_eq!(reserved_ptr, expected_reserved_ptr);
    }

    #[test]
    fn test_header_parse_too_short() {
        let short_data = vec![0x53, 0x42, 0x4D, 0x50]; // Only magic
        let result = Header::parse(&short_data);

        assert!(matches!(
            result,
            Err(ParseError::BufferTooShort { expected: 14, actual: 4 })
        ));
    }

    #[test]
    fn test_header_parse_invalid_magic() {
        let mut data = sample_header();
        data[0] = 0x00; // Corrupt magic

        let result = Header::parse(&data);
        assert!(matches!(
            result,
            Err(ParseError::InvalidMagic { .. })
        ));
    }
}
```

### Solution for Milestone 1

```rust
impl Flags {
    pub fn from_byte(byte: u8) -> Self {
        Flags {
            compressed: (byte & 0b0000_0001) != 0,
            encrypted: (byte & 0b0000_0010) != 0,
            requires_ack: (byte & 0b0000_0100) != 0,
        }
    }

    pub fn to_byte(&self) -> u8 {
        let mut byte = 0u8;
        if self.compressed { byte |= 0b0000_0001; }
        if self.encrypted { byte |= 0b0000_0010; }
        if self.requires_ack { byte |= 0b0000_0100; }
        byte
    }
}

impl MessageType {
    pub fn from_byte(byte: u8) -> Result<Self, ParseError> {
        match byte {
            0x01 => Ok(MessageType::Request),
            0x02 => Ok(MessageType::Response),
            0x03 => Ok(MessageType::Event),
            0x04 => Ok(MessageType::Heartbeat),
            _ => Err(ParseError::InvalidMessageType { value: byte }),
        }
    }
}

impl<'a> Header<'a> {
    pub fn parse(data: &'a [u8]) -> Result<Self, ParseError> {
        if data.len() < MIN_MESSAGE_SIZE {
            return Err(ParseError::BufferTooShort {
                expected: MIN_MESSAGE_SIZE,
                actual: data.len(),
            });
        }

        // Extract magic as a reference to array in the original buffer
        // This is zero-copy: we're creating a reference, not copying
        let magic: &[u8; 4] = data[0..4].try_into().unwrap();

        if magic != &MAGIC {
            return Err(ParseError::InvalidMagic { found: *magic });
        }

        let version = data[4];
        let msg_type = MessageType::from_byte(data[5])?;
        let payload_length = u16::from_be_bytes([data[6], data[7]]);
        let flags = Flags::from_byte(data[8]);

        // Reserved bytes also borrowed from input
        let reserved: &[u8; 3] = data[9..12].try_into().unwrap();

        Ok(Header {
            magic,
            version,
            msg_type,
            payload_length,
            flags,
            reserved,
        })
    }
}
```

---

## Milestone 2: Field Parsing with Cow for String Normalization

**Goal**: Parse variable-length fields, using `Cow` for strings that might need normalization.

**Concepts**: Cow<'a, str>, clone-on-write semantics, conditional allocation.

### Starter Code

```rust
use std::borrow::Cow;

/// Field types in the protocol
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum FieldType {
    String = 0x01,
    Binary = 0x02,
    Integer = 0x03,
    Nested = 0x04,
}

/// A parsed field value - uses Cow for strings to enable zero-copy
/// when no normalization needed, or owned String when normalization required
#[derive(Debug, Clone)]
pub enum FieldValue<'a> {
    /// String field - Cow allows zero-copy OR owned based on content
    String(Cow<'a, str>),
    /// Binary data - always borrowed from input
    Binary(&'a [u8]),
    /// Integer - extracted and converted
    Integer(u64),
    /// Nested message - borrowed slice for recursive parsing
    Nested(&'a [u8]),
}

/// A parsed field with metadata
#[derive(Debug, Clone)]
pub struct Field<'a> {
    pub id: u8,
    pub field_type: FieldType,
    pub value: FieldValue<'a>,
}

/// String normalization options
#[derive(Debug, Clone, Default)]
pub struct NormalizationOptions {
    /// Remove null bytes from strings
    pub strip_nulls: bool,
    /// Trim leading/trailing whitespace
    pub trim_whitespace: bool,
    /// Convert to lowercase
    pub lowercase: bool,
}

// ============================================================
// TODO: Implement the following
// ============================================================

impl FieldType {
    pub fn from_byte(byte: u8) -> Result<Self, ParseError> {
        todo!("Match byte to FieldType variant")
    }
}

impl<'a> FieldValue<'a> {
    /// Parse a string field with optional normalization
    ///
    /// This demonstrates Cow semantics:
    /// - If no normalization needed: Cow::Borrowed (zero-copy)
    /// - If normalization needed: Cow::Owned (allocates)
    ///
    /// The beauty: caller doesn't need to know which case occurred!
    pub fn parse_string(
        data: &'a [u8],
        options: &NormalizationOptions
    ) -> Result<Self, ParseError> {
        todo!("Parse UTF-8 string with conditional normalization")

        // Hints:
        // 1. First convert bytes to &str using std::str::from_utf8
        // 2. Check if ANY normalization is needed
        // 3. If no normalization: return Cow::Borrowed(s)
        // 4. If normalization needed: build owned String and return Cow::Owned
    }

    /// Check if a string needs any normalization
    fn needs_normalization(s: &str, options: &NormalizationOptions) -> bool {
        todo!("Check if string needs any transformation")
    }

    /// Apply all normalizations to create an owned string
    fn normalize_string(s: &str, options: &NormalizationOptions) -> String {
        todo!("Apply all requested normalizations")
    }

    /// Parse binary field (always zero-copy)
    pub fn parse_binary(data: &'a [u8]) -> Self {
        todo!("Return borrowed binary data")
    }

    /// Parse integer field (1, 2, 4, or 8 bytes, big-endian)
    pub fn parse_integer(data: &'a [u8]) -> Result<Self, ParseError> {
        todo!("Parse big-endian integer of various sizes")
    }
}

impl<'a> Field<'a> {
    /// Parse a single field from the buffer
    ///
    /// Returns the parsed field AND the number of bytes consumed
    /// This allows the caller to advance through multiple fields
    pub fn parse(
        data: &'a [u8],
        options: &NormalizationOptions,
    ) -> Result<(Self, usize), ParseError> {
        todo!("Parse field header and value")

        // Field layout:
        // [0]: field_id
        // [1]: field_type
        // [2..4]: length (big-endian u16)
        // [4..4+length]: data
    }
}

/// Parse all fields from a payload
pub fn parse_fields<'a>(
    payload: &'a [u8],
    field_count: u16,
    options: &NormalizationOptions,
) -> Result<Vec<Field<'a>>, ParseError> {
    todo!("Parse multiple fields, accumulating results")
}
```

### Tests for Milestone 2

```rust
#[cfg(test)]
mod field_tests {
    use super::*;

    #[test]
    fn test_string_no_normalization_is_borrowed() {
        let data = b"hello world";
        let options = NormalizationOptions::default();

        let value = FieldValue::parse_string(data, &options).unwrap();

        match value {
            FieldValue::String(cow) => {
                // Should be Borrowed since no normalization needed
                assert!(matches!(cow, Cow::Borrowed(_)));
                assert_eq!(cow.as_ref(), "hello world");
            }
            _ => panic!("Expected String variant"),
        }
    }

    #[test]
    fn test_string_with_nulls_is_owned() {
        let data = b"hel\x00lo";
        let options = NormalizationOptions {
            strip_nulls: true,
            ..Default::default()
        };

        let value = FieldValue::parse_string(data, &options).unwrap();

        match value {
            FieldValue::String(cow) => {
                // Should be Owned since we had to remove null
                assert!(matches!(cow, Cow::Owned(_)));
                assert_eq!(cow.as_ref(), "hello");
            }
            _ => panic!("Expected String variant"),
        }
    }

    #[test]
    fn test_string_trim_whitespace() {
        let data = b"  hello  ";
        let options = NormalizationOptions {
            trim_whitespace: true,
            ..Default::default()
        };

        let value = FieldValue::parse_string(data, &options).unwrap();

        match value {
            FieldValue::String(cow) => {
                assert!(matches!(cow, Cow::Owned(_)));
                assert_eq!(cow.as_ref(), "hello");
            }
            _ => panic!("Expected String variant"),
        }
    }

    #[test]
    fn test_string_lowercase() {
        let data = b"Hello World";
        let options = NormalizationOptions {
            lowercase: true,
            ..Default::default()
        };

        let value = FieldValue::parse_string(data, &options).unwrap();

        match value {
            FieldValue::String(cow) => {
                assert!(matches!(cow, Cow::Owned(_)));
                assert_eq!(cow.as_ref(), "hello world");
            }
            _ => panic!("Expected String variant"),
        }
    }

    #[test]
    fn test_string_already_normalized() {
        // String that would pass all normalizations unchanged
        let data = b"hello";
        let options = NormalizationOptions {
            strip_nulls: true,
            trim_whitespace: true,
            lowercase: true,
        };

        let value = FieldValue::parse_string(data, &options).unwrap();

        match value {
            FieldValue::String(cow) => {
                // Should still be Borrowed - no actual changes needed
                assert!(matches!(cow, Cow::Borrowed(_)));
                assert_eq!(cow.as_ref(), "hello");
            }
            _ => panic!("Expected String variant"),
        }
    }

    #[test]
    fn test_binary_is_always_borrowed() {
        let data = vec![0x01, 0x02, 0x03, 0x04];
        let value = FieldValue::parse_binary(&data);

        match value {
            FieldValue::Binary(slice) => {
                assert_eq!(slice.as_ptr(), data.as_ptr());
                assert_eq!(slice, &[0x01, 0x02, 0x03, 0x04]);
            }
            _ => panic!("Expected Binary variant"),
        }
    }

    #[test]
    fn test_integer_parsing() {
        // 1-byte integer
        assert!(matches!(
            FieldValue::parse_integer(&[0x42]),
            Ok(FieldValue::Integer(0x42))
        ));

        // 2-byte integer (big-endian)
        assert!(matches!(
            FieldValue::parse_integer(&[0x01, 0x00]),
            Ok(FieldValue::Integer(256))
        ));

        // 4-byte integer
        assert!(matches!(
            FieldValue::parse_integer(&[0x00, 0x01, 0x00, 0x00]),
            Ok(FieldValue::Integer(65536))
        ));

        // 8-byte integer
        assert!(matches!(
            FieldValue::parse_integer(&[0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00]),
            Ok(FieldValue::Integer(256))
        ));
    }

    #[test]
    fn test_field_parse() {
        // Field: ID=1, Type=String, Length=5, Data="hello"
        let data = vec![
            0x01,       // Field ID
            0x01,       // Type: String
            0x00, 0x05, // Length: 5
            0x68, 0x65, 0x6C, 0x6C, 0x6F, // "hello"
        ];

        let options = NormalizationOptions::default();
        let (field, consumed) = Field::parse(&data, &options).unwrap();

        assert_eq!(field.id, 1);
        assert_eq!(field.field_type, FieldType::String);
        assert_eq!(consumed, 9); // 4 header + 5 data

        match field.value {
            FieldValue::String(cow) => {
                assert_eq!(cow.as_ref(), "hello");
            }
            _ => panic!("Expected String"),
        }
    }

    #[test]
    fn test_parse_multiple_fields() {
        let payload = vec![
            // Field 1: String "hi"
            0x01, 0x01, 0x00, 0x02, 0x68, 0x69,
            // Field 2: Integer 42
            0x02, 0x03, 0x00, 0x01, 0x2A,
        ];

        let options = NormalizationOptions::default();
        let fields = parse_fields(&payload, 2, &options).unwrap();

        assert_eq!(fields.len(), 2);
        assert_eq!(fields[0].id, 1);
        assert_eq!(fields[1].id, 2);

        match &fields[1].value {
            FieldValue::Integer(n) => assert_eq!(*n, 42),
            _ => panic!("Expected Integer"),
        }
    }
}
```

### Solution for Milestone 2

```rust
impl FieldType {
    pub fn from_byte(byte: u8) -> Result<Self, ParseError> {
        match byte {
            0x01 => Ok(FieldType::String),
            0x02 => Ok(FieldType::Binary),
            0x03 => Ok(FieldType::Integer),
            0x04 => Ok(FieldType::Nested),
            _ => Err(ParseError::InvalidMessageType { value: byte }),
        }
    }
}

impl<'a> FieldValue<'a> {
    pub fn parse_string(
        data: &'a [u8],
        options: &NormalizationOptions,
    ) -> Result<Self, ParseError> {
        // Convert bytes to str - this validates UTF-8
        let s = std::str::from_utf8(data)
            .map_err(|_| ParseError::InvalidUtf8 { field_id: 0 })?;

        // Check if we need to allocate
        if Self::needs_normalization(s, options) {
            Ok(FieldValue::String(Cow::Owned(Self::normalize_string(s, options))))
        } else {
            // Zero-copy path: just borrow the original bytes as str
            Ok(FieldValue::String(Cow::Borrowed(s)))
        }
    }

    fn needs_normalization(s: &str, options: &NormalizationOptions) -> bool {
        if options.strip_nulls && s.contains('\0') {
            return true;
        }
        if options.trim_whitespace && (s.starts_with(char::is_whitespace) || s.ends_with(char::is_whitespace)) {
            return true;
        }
        if options.lowercase && s.chars().any(|c| c.is_uppercase()) {
            return true;
        }
        false
    }

    fn normalize_string(s: &str, options: &NormalizationOptions) -> String {
        let mut result = s.to_string();

        if options.strip_nulls {
            result = result.replace('\0', "");
        }
        if options.trim_whitespace {
            result = result.trim().to_string();
        }
        if options.lowercase {
            result = result.to_lowercase();
        }

        result
    }

    pub fn parse_binary(data: &'a [u8]) -> Self {
        FieldValue::Binary(data)
    }

    pub fn parse_integer(data: &'a [u8]) -> Result<Self, ParseError> {
        let value = match data.len() {
            1 => data[0] as u64,
            2 => u16::from_be_bytes([data[0], data[1]]) as u64,
            4 => u32::from_be_bytes([data[0], data[1], data[2], data[3]]) as u64,
            8 => u64::from_be_bytes([
                data[0], data[1], data[2], data[3],
                data[4], data[5], data[6], data[7],
            ]),
            _ => return Err(ParseError::InvalidMessageType { value: data.len() as u8 }),
        };
        Ok(FieldValue::Integer(value))
    }
}

impl<'a> Field<'a> {
    pub fn parse(
        data: &'a [u8],
        options: &NormalizationOptions,
    ) -> Result<(Self, usize), ParseError> {
        if data.len() < 4 {
            return Err(ParseError::BufferTooShort {
                expected: 4,
                actual: data.len(),
            });
        }

        let field_id = data[0];
        let field_type = FieldType::from_byte(data[1])?;
        let length = u16::from_be_bytes([data[2], data[3]]) as usize;

        if data.len() < 4 + length {
            return Err(ParseError::UnexpectedEndOfData {
                field_id,
                expected: length,
                remaining: data.len() - 4,
            });
        }

        let field_data = &data[4..4 + length];

        let value = match field_type {
            FieldType::String => FieldValue::parse_string(field_data, options)?,
            FieldType::Binary => FieldValue::parse_binary(field_data),
            FieldType::Integer => FieldValue::parse_integer(field_data)?,
            FieldType::Nested => FieldValue::Nested(field_data),
        };

        Ok((
            Field {
                id: field_id,
                field_type,
                value,
            },
            4 + length,
        ))
    }
}

pub fn parse_fields<'a>(
    payload: &'a [u8],
    field_count: u16,
    options: &NormalizationOptions,
) -> Result<Vec<Field<'a>>, ParseError> {
    let mut fields = Vec::with_capacity(field_count as usize);
    let mut offset = 0;

    for _ in 0..field_count {
        let (field, consumed) = Field::parse(&payload[offset..], options)?;
        fields.push(field);
        offset += consumed;
    }

    Ok(fields)
}
```

---

## Milestone 3: Lookup Table with Borrow Trait

**Goal**: Build a field lookup table that supports heterogeneous key types.

**Concepts**: Borrow trait, HashMap with borrowed lookups, key equivalence.

### Starter Code

```rust
use std::borrow::Borrow;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

/// A field identifier that can be looked up by name or ID
///
/// This demonstrates the Borrow trait: we store FieldKey in the HashMap,
/// but can look up using either &str (for name) or u8 (for ID)
#[derive(Debug, Clone)]
pub struct FieldKey {
    pub id: u8,
    pub name: String,
}

// For HashMap to work correctly with Borrow, we need:
// 1. FieldKey, &str, and u8 to have compatible Hash implementations
// 2. FieldKey, &str, and u8 to have compatible Eq implementations

impl PartialEq for FieldKey {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id  // Primary key is ID
    }
}

impl Eq for FieldKey {}

impl Hash for FieldKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);  // Hash by ID only
    }
}

// ============================================================
// TODO: Implement Borrow<u8> for FieldKey
// This allows looking up by ID: map.get(&42u8)
// ============================================================

impl Borrow<u8> for FieldKey {
    fn borrow(&self) -> &u8 {
        todo!("Return reference to the ID")
    }
}

/// A wrapper around u8 for name-based lookups
///
/// Since we can't implement Borrow<str> for FieldKey (would need different Hash),
/// we use a separate approach for name lookups
#[derive(Debug)]
pub struct FieldRegistry<'a> {
    /// Primary storage: ID -> Field
    by_id: HashMap<FieldKey, Field<'a>>,
    /// Secondary index: name -> ID (for name-based lookup)
    name_to_id: HashMap<String, u8>,
}

impl<'a> FieldRegistry<'a> {
    pub fn new() -> Self {
        FieldRegistry {
            by_id: HashMap::new(),
            name_to_id: HashMap::new(),
        }
    }

    /// Register a field with a name
    pub fn insert(&mut self, name: impl Into<String>, field: Field<'a>) {
        let name = name.into();
        let id = field.id;

        self.name_to_id.insert(name.clone(), id);
        self.by_id.insert(FieldKey { id, name }, field);
    }

    /// Look up by ID (uses Borrow<u8>)
    pub fn get_by_id(&self, id: u8) -> Option<&Field<'a>> {
        todo!("Use Borrow trait to look up by ID")
        // Hint: HashMap::get accepts anything that K: Borrow<Q>
    }

    /// Look up by name (uses secondary index)
    pub fn get_by_name(&self, name: &str) -> Option<&Field<'a>> {
        todo!("Look up ID by name, then get field by ID")
    }

    /// Iterate over all fields
    pub fn iter(&self) -> impl Iterator<Item = (&FieldKey, &Field<'a>)> {
        self.by_id.iter()
    }
}

// ============================================================
// Bonus: Generic lookup function demonstrating Borrow bounds
// ============================================================

/// A generic function that can look up in any map where keys implement Borrow<Q>
///
/// This pattern is used throughout the standard library
pub fn lookup_flexible<'a, 'b, K, V, Q>(
    map: &'a HashMap<K, V>,
    key: &'b Q,
) -> Option<&'a V>
where
    K: Borrow<Q> + Hash + Eq,
    Q: Hash + Eq + ?Sized,
{
    todo!("Delegate to HashMap::get")
}
```

### Tests for Milestone 3

```rust
#[cfg(test)]
mod registry_tests {
    use super::*;

    fn sample_field(id: u8) -> Field<'static> {
        Field {
            id,
            field_type: FieldType::Integer,
            value: FieldValue::Integer(id as u64 * 100),
        }
    }

    #[test]
    fn test_borrow_u8_for_field_key() {
        let key = FieldKey {
            id: 42,
            name: "test".to_string(),
        };

        let borrowed: &u8 = key.borrow();
        assert_eq!(*borrowed, 42);
    }

    #[test]
    fn test_registry_lookup_by_id() {
        let mut registry = FieldRegistry::new();
        registry.insert("user_id", sample_field(1));
        registry.insert("timestamp", sample_field(2));
        registry.insert("payload", sample_field(3));

        // Look up by ID
        let field = registry.get_by_id(2).unwrap();
        assert_eq!(field.id, 2);

        // Non-existent ID
        assert!(registry.get_by_id(99).is_none());
    }

    #[test]
    fn test_registry_lookup_by_name() {
        let mut registry = FieldRegistry::new();
        registry.insert("user_id", sample_field(1));
        registry.insert("timestamp", sample_field(2));

        // Look up by name
        let field = registry.get_by_name("user_id").unwrap();
        assert_eq!(field.id, 1);

        // Non-existent name
        assert!(registry.get_by_name("nonexistent").is_none());
    }

    #[test]
    fn test_hashmap_borrow_lookup() {
        // Demonstrate that HashMap lookup works with Borrow
        let mut map: HashMap<FieldKey, i32> = HashMap::new();
        map.insert(FieldKey { id: 1, name: "one".to_string() }, 100);
        map.insert(FieldKey { id: 2, name: "two".to_string() }, 200);

        // Look up using &u8 instead of &FieldKey
        // This works because FieldKey: Borrow<u8>
        assert_eq!(map.get(&1u8), Some(&100));
        assert_eq!(map.get(&2u8), Some(&200));
        assert_eq!(map.get(&3u8), None);
    }

    #[test]
    fn test_flexible_lookup() {
        let mut map: HashMap<String, i32> = HashMap::new();
        map.insert("hello".to_string(), 1);
        map.insert("world".to_string(), 2);

        // String: Borrow<str>, so we can look up with &str
        assert_eq!(lookup_flexible(&map, "hello"), Some(&1));
        assert_eq!(lookup_flexible(&map, "world"), Some(&2));
        assert_eq!(lookup_flexible(&map, "missing"), None);
    }
}
```

### Solution for Milestone 3

```rust
impl Borrow<u8> for FieldKey {
    fn borrow(&self) -> &u8 {
        &self.id
    }
}

impl<'a> FieldRegistry<'a> {
    pub fn get_by_id(&self, id: u8) -> Option<&Field<'a>> {
        // HashMap::get uses Borrow<Q> where K: Borrow<Q>
        // Since FieldKey: Borrow<u8>, we can pass &u8
        self.by_id.get(&id)
    }

    pub fn get_by_name(&self, name: &str) -> Option<&Field<'a>> {
        // First look up ID by name
        let id = self.name_to_id.get(name)?;
        // Then look up field by ID
        self.get_by_id(*id)
    }
}

pub fn lookup_flexible<'a, 'b, K, V, Q>(
    map: &'a HashMap<K, V>,
    key: &'b Q,
) -> Option<&'a V>
where
    K: Borrow<Q> + Hash + Eq,
    Q: Hash + Eq + ?Sized,
{
    map.get(key)
}
```

---

## Milestone 4: Complete Message Parser with Naming Conventions

**Goal**: Create a complete `Message` type with a consistent API following Rust naming conventions.

**Concepts**: as_/to_/into_ naming, API design, comprehensive parsing.

### Starter Code

```rust
/// A fully parsed message with zero-copy field access
#[derive(Debug)]
pub struct Message<'a> {
    header: Header<'a>,
    fields: Vec<Field<'a>>,
    /// Keep reference to original buffer for raw access
    raw: &'a [u8],
}

impl<'a> Message<'a> {
    // ============================================================
    // Parsing
    // ============================================================

    /// Parse a complete message from bytes
    pub fn parse(data: &'a [u8]) -> Result<Self, ParseError> {
        Self::parse_with_options(data, &NormalizationOptions::default())
    }

    /// Parse with custom normalization options
    pub fn parse_with_options(
        data: &'a [u8],
        options: &NormalizationOptions,
    ) -> Result<Self, ParseError> {
        todo!("Parse header, then parse fields from payload")
    }

    // ============================================================
    // as_* methods: Return references, O(1), no allocation
    // ============================================================

    /// Get the raw bytes of the entire message
    pub fn as_bytes(&self) -> &[u8] {
        todo!()
    }

    /// Get the header
    pub fn as_header(&self) -> &Header<'a> {
        todo!()
    }

    /// Get a field by ID (returns reference)
    pub fn as_field(&self, id: u8) -> Option<&Field<'a>> {
        todo!()
    }

    /// Get all fields as a slice
    pub fn as_fields(&self) -> &[Field<'a>] {
        todo!()
    }

    /// Get just the payload bytes (after header)
    pub fn as_payload(&self) -> &[u8] {
        todo!()
    }

    // ============================================================
    // to_* methods: Return owned data, may allocate
    // ============================================================

    /// Create an owned copy of all string fields
    pub fn to_strings(&self) -> Vec<String> {
        todo!("Collect all String fields into owned Strings")
    }

    /// Serialize message back to bytes
    pub fn to_bytes(&self) -> Vec<u8> {
        todo!("Serialize the message (may differ from original if normalized)")
    }

    /// Create a debug representation
    pub fn to_debug_string(&self) -> String {
        todo!("Format message for debugging")
    }

    // ============================================================
    // into_* methods: Consume self, transfer ownership
    // ============================================================

    /// Consume message, return owned fields
    pub fn into_fields(self) -> Vec<Field<'a>> {
        todo!()
    }

    /// Consume message, return just the header
    pub fn into_header(self) -> Header<'a> {
        todo!()
    }

    /// Decompose into parts
    pub fn into_parts(self) -> (Header<'a>, Vec<Field<'a>>) {
        todo!()
    }

    // ============================================================
    // Convenience methods
    // ============================================================

    /// Get a string field by ID
    pub fn get_string(&self, id: u8) -> Option<&str> {
        todo!("Find field by ID, extract string value")
    }

    /// Get an integer field by ID
    pub fn get_integer(&self, id: u8) -> Option<u64> {
        todo!("Find field by ID, extract integer value")
    }

    /// Get a binary field by ID
    pub fn get_binary(&self, id: u8) -> Option<&[u8]> {
        todo!("Find field by ID, extract binary value")
    }

    /// Check if message has a specific field
    pub fn has_field(&self, id: u8) -> bool {
        todo!()
    }

    /// Get field count
    pub fn field_count(&self) -> usize {
        todo!()
    }

    /// Check message type
    pub fn message_type(&self) -> MessageType {
        todo!()
    }

    /// Check if message is compressed
    pub fn is_compressed(&self) -> bool {
        todo!()
    }
}

// ============================================================
// Implement standard traits
// ============================================================

impl<'a> AsRef<[u8]> for Message<'a> {
    fn as_ref(&self) -> &[u8] {
        self.as_bytes()
    }
}
```

### Tests for Milestone 4

```rust
#[cfg(test)]
mod message_tests {
    use super::*;

    fn sample_message() -> Vec<u8> {
        vec![
            // Header (14 bytes)
            0x53, 0x42, 0x4D, 0x50, // Magic
            0x01,                   // Version
            0x01,                   // Type: Request
            0x00, 0x11,             // Payload: 17 bytes
            0x01,                   // Flags: compressed
            0x00, 0x00, 0x00,       // Reserved
            0x00, 0x02,             // Field count: 2

            // Field 1: String "hello" (ID=1)
            0x01, 0x01, 0x00, 0x05,
            0x68, 0x65, 0x6C, 0x6C, 0x6F,

            // Field 2: Integer 256 (ID=2)
            0x02, 0x03, 0x00, 0x02,
            0x01, 0x00,
        ]
    }

    #[test]
    fn test_message_parse() {
        let data = sample_message();
        let msg = Message::parse(&data).unwrap();

        assert_eq!(msg.message_type(), MessageType::Request);
        assert_eq!(msg.field_count(), 2);
        assert!(msg.is_compressed());
    }

    #[test]
    fn test_as_methods_no_allocation() {
        let data = sample_message();
        let msg = Message::parse(&data).unwrap();

        // as_bytes returns reference to original
        assert_eq!(msg.as_bytes().as_ptr(), data.as_ptr());

        // as_fields returns slice reference
        let fields = msg.as_fields();
        assert_eq!(fields.len(), 2);
    }

    #[test]
    fn test_get_string() {
        let data = sample_message();
        let msg = Message::parse(&data).unwrap();

        assert_eq!(msg.get_string(1), Some("hello"));
        assert_eq!(msg.get_string(2), None); // Field 2 is integer
        assert_eq!(msg.get_string(99), None); // Non-existent
    }

    #[test]
    fn test_get_integer() {
        let data = sample_message();
        let msg = Message::parse(&data).unwrap();

        assert_eq!(msg.get_integer(2), Some(256));
        assert_eq!(msg.get_integer(1), None); // Field 1 is string
    }

    #[test]
    fn test_to_strings_allocates() {
        let data = sample_message();
        let msg = Message::parse(&data).unwrap();

        let strings = msg.to_strings();
        assert_eq!(strings, vec!["hello".to_string()]);
    }

    #[test]
    fn test_into_parts_consumes() {
        let data = sample_message();
        let msg = Message::parse(&data).unwrap();

        let (header, fields) = msg.into_parts();
        assert_eq!(header.version, 1);
        assert_eq!(fields.len(), 2);

        // msg is consumed, can't use it anymore
    }

    #[test]
    fn test_as_ref_trait() {
        let data = sample_message();
        let msg = Message::parse(&data).unwrap();

        // Message implements AsRef<[u8]>
        fn takes_as_ref(data: impl AsRef<[u8]>) -> usize {
            data.as_ref().len()
        }

        assert_eq!(takes_as_ref(&msg), data.len());
    }

    #[test]
    fn test_has_field() {
        let data = sample_message();
        let msg = Message::parse(&data).unwrap();

        assert!(msg.has_field(1));
        assert!(msg.has_field(2));
        assert!(!msg.has_field(3));
    }
}
```

### Solution for Milestone 4

```rust
impl<'a> Message<'a> {
    pub fn parse_with_options(
        data: &'a [u8],
        options: &NormalizationOptions,
    ) -> Result<Self, ParseError> {
        let header = Header::parse(data)?;

        // Payload starts after the 14-byte header
        let payload_start = MIN_MESSAGE_SIZE;
        let field_count = u16::from_be_bytes([data[12], data[13]]);

        let payload = &data[payload_start..];
        let fields = parse_fields(payload, field_count, options)?;

        Ok(Message {
            header,
            fields,
            raw: data,
        })
    }

    // as_* methods

    pub fn as_bytes(&self) -> &[u8] {
        self.raw
    }

    pub fn as_header(&self) -> &Header<'a> {
        &self.header
    }

    pub fn as_field(&self, id: u8) -> Option<&Field<'a>> {
        self.fields.iter().find(|f| f.id == id)
    }

    pub fn as_fields(&self) -> &[Field<'a>] {
        &self.fields
    }

    pub fn as_payload(&self) -> &[u8] {
        &self.raw[MIN_MESSAGE_SIZE..]
    }

    // to_* methods

    pub fn to_strings(&self) -> Vec<String> {
        self.fields
            .iter()
            .filter_map(|f| match &f.value {
                FieldValue::String(cow) => Some(cow.to_string()),
                _ => None,
            })
            .collect()
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        // For simplicity, return a copy of raw bytes
        // A full implementation would re-serialize
        self.raw.to_vec()
    }

    pub fn to_debug_string(&self) -> String {
        format!(
            "Message {{ type: {:?}, version: {}, fields: {}, compressed: {} }}",
            self.header.msg_type,
            self.header.version,
            self.fields.len(),
            self.header.flags.compressed
        )
    }

    // into_* methods

    pub fn into_fields(self) -> Vec<Field<'a>> {
        self.fields
    }

    pub fn into_header(self) -> Header<'a> {
        self.header
    }

    pub fn into_parts(self) -> (Header<'a>, Vec<Field<'a>>) {
        (self.header, self.fields)
    }

    // Convenience methods

    pub fn get_string(&self, id: u8) -> Option<&str> {
        self.as_field(id).and_then(|f| match &f.value {
            FieldValue::String(cow) => Some(cow.as_ref()),
            _ => None,
        })
    }

    pub fn get_integer(&self, id: u8) -> Option<u64> {
        self.as_field(id).and_then(|f| match &f.value {
            FieldValue::Integer(n) => Some(*n),
            _ => None,
        })
    }

    pub fn get_binary(&self, id: u8) -> Option<&[u8]> {
        self.as_field(id).and_then(|f| match &f.value {
            FieldValue::Binary(data) => Some(*data),
            _ => None,
        })
    }

    pub fn has_field(&self, id: u8) -> bool {
        self.fields.iter().any(|f| f.id == id)
    }

    pub fn field_count(&self) -> usize {
        self.fields.len()
    }

    pub fn message_type(&self) -> MessageType {
        self.header.msg_type
    }

    pub fn is_compressed(&self) -> bool {
        self.header.flags.compressed
    }
}
```

---

## Milestone 5: Nested Message Parsing with Borrow Splitting

**Goal**: Parse nested messages and demonstrate borrow splitting for simultaneous access.

**Concepts**: Borrow splitting, recursive parsing, lifetime propagation.

### Starter Code

```rust
/// A message that may contain nested messages as field values
#[derive(Debug)]
pub struct NestedMessage<'a> {
    pub header: Header<'a>,
    pub fields: Vec<ParsedField<'a>>,
    raw: &'a [u8],
}

/// A field that has been fully parsed, including nested messages
#[derive(Debug)]
pub enum ParsedField<'a> {
    String { id: u8, value: Cow<'a, str> },
    Binary { id: u8, value: &'a [u8] },
    Integer { id: u8, value: u64 },
    /// Nested message has been recursively parsed
    Nested { id: u8, message: Box<NestedMessage<'a>> },
}

impl<'a> NestedMessage<'a> {
    /// Parse a message, recursively parsing any nested message fields
    pub fn parse(data: &'a [u8]) -> Result<Self, ParseError> {
        Self::parse_with_options(data, &NormalizationOptions::default(), 0)
    }

    /// Parse with depth limit to prevent stack overflow on malicious input
    fn parse_with_options(
        data: &'a [u8],
        options: &NormalizationOptions,
        depth: usize,
    ) -> Result<Self, ParseError> {
        const MAX_DEPTH: usize = 16;

        if depth > MAX_DEPTH {
            return Err(ParseError::BufferTooShort {
                expected: 0,
                actual: 0
            }); // Would use a proper NestingTooDeep error
        }

        todo!("Parse header and fields, recursively parsing nested messages")
    }

    /// Get a reference to a top-level field AND a nested field simultaneously
    ///
    /// This demonstrates borrow splitting: we can have multiple immutable
    /// references to different parts of the structure
    pub fn get_field_pair(&self, id1: u8, id2: u8) -> (Option<&ParsedField<'a>>, Option<&ParsedField<'a>>) {
        todo!("Return references to two different fields")
    }

    /// Iterate over all nested messages (at any depth)
    pub fn nested_messages(&self) -> Vec<&NestedMessage<'a>> {
        todo!("Collect all nested messages recursively")
    }

    /// Find a field at a path like [1, 3, 2] meaning:
    /// field 1 -> nested message -> field 3 -> nested message -> field 2
    pub fn get_at_path(&self, path: &[u8]) -> Option<&ParsedField<'a>> {
        todo!("Navigate through nested structure")
    }
}

/// Builder for constructing nested messages (demonstrates mutable borrow splitting)
#[derive(Debug, Default)]
pub struct MessageBuilder {
    fields: Vec<BuilderField>,
}

#[derive(Debug)]
enum BuilderField {
    String { id: u8, value: String },
    Binary { id: u8, value: Vec<u8> },
    Integer { id: u8, value: u64 },
    Nested { id: u8, builder: MessageBuilder },
}

impl MessageBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_string(&mut self, id: u8, value: impl Into<String>) -> &mut Self {
        self.fields.push(BuilderField::String {
            id,
            value: value.into()
        });
        self
    }

    pub fn add_integer(&mut self, id: u8, value: u64) -> &mut Self {
        self.fields.push(BuilderField::Integer { id, value });
        self
    }

    /// Add a nested message, returning a mutable reference to configure it
    ///
    /// This demonstrates mutable borrow of a sub-structure
    pub fn add_nested(&mut self, id: u8) -> &mut MessageBuilder {
        self.fields.push(BuilderField::Nested {
            id,
            builder: MessageBuilder::new()
        });

        // Return mutable reference to the nested builder
        match self.fields.last_mut().unwrap() {
            BuilderField::Nested { builder, .. } => builder,
            _ => unreachable!(),
        }
    }

    /// Build the message into bytes
    pub fn build(&self) -> Vec<u8> {
        todo!("Serialize the builder into SBMP format")
    }
}
```

### Tests for Milestone 5

```rust
#[cfg(test)]
mod nested_tests {
    use super::*;

    fn message_with_nested() -> Vec<u8> {
        vec![
            // Outer header
            0x53, 0x42, 0x4D, 0x50,
            0x01, 0x01,
            0x00, 0x1F,  // Payload: 31 bytes
            0x00, 0x00, 0x00, 0x00,
            0x00, 0x02,  // 2 fields

            // Field 1: String "outer"
            0x01, 0x01, 0x00, 0x05,
            0x6F, 0x75, 0x74, 0x65, 0x72,

            // Field 2: Nested message
            0x02, 0x04, 0x00, 0x12,  // Type 0x04 = Nested, 18 bytes
            // Nested message content:
            0x53, 0x42, 0x4D, 0x50,
            0x01, 0x02,  // Response
            0x00, 0x04,  // Payload: 4 bytes
            0x00, 0x00, 0x00, 0x00,
            0x00, 0x01,  // 1 field
            // Nested field: Integer 42
            0x01, 0x03, 0x00, 0x01, 0x2A,
        ]
    }

    #[test]
    fn test_nested_parse() {
        let data = message_with_nested();
        let msg = NestedMessage::parse(&data).unwrap();

        assert_eq!(msg.fields.len(), 2);

        // First field is string
        match &msg.fields[0] {
            ParsedField::String { id, value } => {
                assert_eq!(*id, 1);
                assert_eq!(value.as_ref(), "outer");
            }
            _ => panic!("Expected string"),
        }

        // Second field is nested
        match &msg.fields[1] {
            ParsedField::Nested { id, message } => {
                assert_eq!(*id, 2);
                assert_eq!(message.header.msg_type, MessageType::Response);
                assert_eq!(message.fields.len(), 1);
            }
            _ => panic!("Expected nested"),
        }
    }

    #[test]
    fn test_borrow_splitting() {
        let data = message_with_nested();
        let msg = NestedMessage::parse(&data).unwrap();

        // Can get references to two fields simultaneously
        let (f1, f2) = msg.get_field_pair(1, 2);

        assert!(f1.is_some());
        assert!(f2.is_some());

        // Both references are valid at the same time
        match (f1.unwrap(), f2.unwrap()) {
            (ParsedField::String { value, .. }, ParsedField::Nested { message, .. }) => {
                assert_eq!(value.as_ref(), "outer");
                assert_eq!(message.fields.len(), 1);
            }
            _ => panic!("Unexpected field types"),
        }
    }

    #[test]
    fn test_nested_messages_collection() {
        let data = message_with_nested();
        let msg = NestedMessage::parse(&data).unwrap();

        let nested = msg.nested_messages();
        assert_eq!(nested.len(), 1);
        assert_eq!(nested[0].header.msg_type, MessageType::Response);
    }

    #[test]
    fn test_get_at_path() {
        let data = message_with_nested();
        let msg = NestedMessage::parse(&data).unwrap();

        // Path [2, 1] = field 2 (nested) -> field 1 (integer)
        let field = msg.get_at_path(&[2, 1]).unwrap();

        match field {
            ParsedField::Integer { value, .. } => {
                assert_eq!(*value, 42);
            }
            _ => panic!("Expected integer at path"),
        }
    }

    #[test]
    fn test_builder() {
        let mut builder = MessageBuilder::new();
        builder
            .add_string(1, "hello")
            .add_integer(2, 100);

        builder.add_nested(3)
            .add_string(1, "nested")
            .add_integer(2, 200);

        let bytes = builder.build();

        // Verify we can parse what we built
        let msg = NestedMessage::parse(&bytes).unwrap();
        assert_eq!(msg.fields.len(), 3);
    }
}
```

### Solution for Milestone 5

```rust
impl<'a> NestedMessage<'a> {
    fn parse_with_options(
        data: &'a [u8],
        options: &NormalizationOptions,
        depth: usize,
    ) -> Result<Self, ParseError> {
        const MAX_DEPTH: usize = 16;

        if depth > MAX_DEPTH {
            return Err(ParseError::BufferTooShort {
                expected: 0,
                actual: 0
            });
        }

        let header = Header::parse(data)?;
        let field_count = u16::from_be_bytes([data[12], data[13]]);
        let payload = &data[MIN_MESSAGE_SIZE..];

        let mut fields = Vec::with_capacity(field_count as usize);
        let mut offset = 0;

        for _ in 0..field_count {
            if payload.len() < offset + 4 {
                break;
            }

            let field_id = payload[offset];
            let field_type = FieldType::from_byte(payload[offset + 1])?;
            let length = u16::from_be_bytes([
                payload[offset + 2],
                payload[offset + 3]
            ]) as usize;

            let field_data = &payload[offset + 4..offset + 4 + length];

            let parsed = match field_type {
                FieldType::String => {
                    let s = std::str::from_utf8(field_data)
                        .map_err(|_| ParseError::InvalidUtf8 { field_id })?;

                    let value = if FieldValue::needs_normalization(s, options) {
                        Cow::Owned(FieldValue::normalize_string(s, options))
                    } else {
                        Cow::Borrowed(s)
                    };

                    ParsedField::String { id: field_id, value }
                }
                FieldType::Binary => {
                    ParsedField::Binary { id: field_id, value: field_data }
                }
                FieldType::Integer => {
                    let value = match length {
                        1 => field_data[0] as u64,
                        2 => u16::from_be_bytes([field_data[0], field_data[1]]) as u64,
                        4 => u32::from_be_bytes([
                            field_data[0], field_data[1],
                            field_data[2], field_data[3]
                        ]) as u64,
                        8 => u64::from_be_bytes([
                            field_data[0], field_data[1], field_data[2], field_data[3],
                            field_data[4], field_data[5], field_data[6], field_data[7],
                        ]),
                        _ => 0,
                    };
                    ParsedField::Integer { id: field_id, value }
                }
                FieldType::Nested => {
                    let nested = Self::parse_with_options(field_data, options, depth + 1)?;
                    ParsedField::Nested {
                        id: field_id,
                        message: Box::new(nested)
                    }
                }
            };

            fields.push(parsed);
            offset += 4 + length;
        }

        Ok(NestedMessage {
            header,
            fields,
            raw: data,
        })
    }

    pub fn get_field_pair(
        &self,
        id1: u8,
        id2: u8
    ) -> (Option<&ParsedField<'a>>, Option<&ParsedField<'a>>) {
        let f1 = self.fields.iter().find(|f| match f {
            ParsedField::String { id, .. } |
            ParsedField::Binary { id, .. } |
            ParsedField::Integer { id, .. } |
            ParsedField::Nested { id, .. } => *id == id1,
        });

        let f2 = self.fields.iter().find(|f| match f {
            ParsedField::String { id, .. } |
            ParsedField::Binary { id, .. } |
            ParsedField::Integer { id, .. } |
            ParsedField::Nested { id, .. } => *id == id2,
        });

        (f1, f2)
    }

    pub fn nested_messages(&self) -> Vec<&NestedMessage<'a>> {
        let mut result = Vec::new();

        for field in &self.fields {
            if let ParsedField::Nested { message, .. } = field {
                result.push(message.as_ref());
                result.extend(message.nested_messages());
            }
        }

        result
    }

    pub fn get_at_path(&self, path: &[u8]) -> Option<&ParsedField<'a>> {
        if path.is_empty() {
            return None;
        }

        let field = self.fields.iter().find(|f| match f {
            ParsedField::String { id, .. } |
            ParsedField::Binary { id, .. } |
            ParsedField::Integer { id, .. } |
            ParsedField::Nested { id, .. } => *id == path[0],
        })?;

        if path.len() == 1 {
            return Some(field);
        }

        // Need to descend into nested message
        match field {
            ParsedField::Nested { message, .. } => {
                message.get_at_path(&path[1..])
            }
            _ => None,
        }
    }
}

impl MessageBuilder {
    pub fn build(&self) -> Vec<u8> {
        let mut payload = Vec::new();

        for field in &self.fields {
            match field {
                BuilderField::String { id, value } => {
                    payload.push(*id);
                    payload.push(0x01); // String type
                    let len = value.len() as u16;
                    payload.extend_from_slice(&len.to_be_bytes());
                    payload.extend_from_slice(value.as_bytes());
                }
                BuilderField::Binary { id, value } => {
                    payload.push(*id);
                    payload.push(0x02); // Binary type
                    let len = value.len() as u16;
                    payload.extend_from_slice(&len.to_be_bytes());
                    payload.extend_from_slice(value);
                }
                BuilderField::Integer { id, value } => {
                    payload.push(*id);
                    payload.push(0x03); // Integer type
                    let bytes = value.to_be_bytes();
                    // Find minimal representation
                    let (data, len) = if *value <= 0xFF {
                        (&bytes[7..8], 1u16)
                    } else if *value <= 0xFFFF {
                        (&bytes[6..8], 2u16)
                    } else if *value <= 0xFFFFFFFF {
                        (&bytes[4..8], 4u16)
                    } else {
                        (&bytes[..], 8u16)
                    };
                    payload.extend_from_slice(&len.to_be_bytes());
                    payload.extend_from_slice(data);
                }
                BuilderField::Nested { id, builder } => {
                    let nested_bytes = builder.build();
                    payload.push(*id);
                    payload.push(0x04); // Nested type
                    let len = nested_bytes.len() as u16;
                    payload.extend_from_slice(&len.to_be_bytes());
                    payload.extend_from_slice(&nested_bytes);
                }
            }
        }

        // Build header
        let mut result = Vec::with_capacity(14 + payload.len());
        result.extend_from_slice(b"SBMP");
        result.push(0x01); // Version
        result.push(0x01); // Type: Request
        result.extend_from_slice(&(payload.len() as u16).to_be_bytes());
        result.push(0x00); // Flags
        result.extend_from_slice(&[0x00, 0x00, 0x00]); // Reserved
        result.extend_from_slice(&(self.fields.len() as u16).to_be_bytes());
        result.extend_from_slice(&payload);

        result
    }
}
```

---

## Summary

This project demonstrated key reference and iterator patterns through building a zero-copy protocol parser:

| Milestone | Pattern | Key Takeaway |
|-----------|---------|--------------|
| 1 | Reference Binding | Binding modes propagate from matched type |
| 2 | Cow | Defer allocation until actually needed |
| 3 | Borrow vs AsRef | Borrow requires hash equivalence |
| 4 | Naming Conventions | as_/to_/into_ communicate allocation intent |
| 5 | Borrow Splitting | Multiple borrows of disjoint data |

**Performance Impact**: A zero-copy parser can process millions of messages per second because it avoids heap allocation on the hot path. The `Cow` pattern means most strings pass through without allocation, while still handling edge cases that require normalization.

**Type System Benefits**: Rust's lifetime system ensures parsed references remain valid. The compiler verifies at compile time that we never use a `Message<'a>` after the underlying buffer is freed.
