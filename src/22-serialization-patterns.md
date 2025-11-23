# Serialization Patterns

[Serde Patterns](#pattern-1-serde-patterns)

- Problem: Manual serialization tedious for every type/format; forget to update when struct changes; JSON, TOML, MessagePack each need custom code
- Solution: Derive Serialize/Deserialize; field attributes (rename, skip, default); custom serializers; serde data model separates structure from format
- Why It Matters: Zero-cost abstraction—fast as hand-written; switch formats (JSON→MessagePack) in one line; compile-time type safety catches mismatches
- Use Cases: REST APIs (JSON), config files (TOML/YAML), Rust-to-Rust RPC (bincode), cross-language messaging (MessagePack), database storage

[Zero-Copy Deserialization](#pattern-2-zero-copy-deserialization)

- Problem: Deserializing allocates (String, Vec)—wasteful when input buffer lives long enough; parsing JSON allocates even for borrowed data
- Solution: Use &str and &[u8] in structs; #[serde(borrow)] attribute; serde_json::from_slice with lifetime-aware types; zero-copy avoids heap allocation
- Why It Matters: 10x faster for large inputs (no allocation); constant memory usage; critical for high-throughput parsers (log processing, network protocols)
- Use Cases: Parsing logs, HTTP request/response parsing, streaming data, embedded systems (limited RAM), zero-allocation parsers

[Schema Evolution](#pattern-3-schema-evolution)

- Problem: API changes break clients; adding fields breaks deserialization; renaming fields incompatible; version migrations painful; backward compatibility hard
- Solution: #[serde(default)] for new fields; #[serde(rename)] preserves wire format; #[serde(alias)] accepts old names; versioning with tagged unions
- Why It Matters: Enables gradual rollout—old clients work with new servers; adding fields doesn't break compatibility; refactoring safe (rename internally, keep API)
- Use Cases: Versioned APIs (v1/v2 coexist), database migrations, config file evolution, backward-compatible protocols, gradual service updates

[Binary vs Text Formats](#pattern-4-binary-vs-text-formats)

- Problem: JSON human-readable but large/slow; bincode compact but Rust-only; need cross-language binary format; size vs compatibility tradeoff
- Solution: JSON for APIs/humans (readable, debuggable); bincode for Rust↔Rust (smallest, fastest); MessagePack/CBOR for cross-language binary; TOML for configs
- Why It Matters: JSON 2-5x larger than binary; bincode 10x faster than JSON parse; MessagePack: 60% JSON size, language-agnostic; format choice impacts latency/bandwidth
- Use Cases: JSON (REST APIs, config), bincode (IPC, caching), MessagePack (microservices), CBOR (IoT, embedded), TOML (simple configs), YAML (complex configs)

[Streaming Serialization](#pattern-5-streaming-serialization)

- Problem: Serializing GB data exhausts memory; can't load entire dataset; need to process incrementally; parsing large JSON arrays allocates all elements
- Solution: Stream API with iterators; serialize incrementally; serde_json::Deserializer::from_reader with streaming_iterator; write as you go, not all-at-once
- Why It Matters: O(1) memory vs O(N) for full load; process files larger than RAM; enables backpressure (slow consumer doesn't OOM); essential for logs/DB exports
- Use Cases: Large file processing (GB logs, DB dumps), streaming APIs (server-sent events), incremental parsing, log aggregation, ETL pipelines


[Serde Cheat Sheet](#serde-cheat-sheet)
 - common **serde** functions

## Overview
This chapter covers serialization patterns using serde—converting Rust types to/from JSON, binary formats, config files. Serde provides zero-cost abstraction: types separated from formats, derive macros generate optimal code, switch formats by changing one line.



## Pattern 1: Serde Patterns

**Problem**: Writing manual serialization code for every type and format is tedious. Converting Person to JSON requires writing to_json(). Adding TOML support duplicates effort. When you change struct fields, manual serialization code breaks. Supporting multiple formats (JSON, MessagePack, bincode) means 3x code duplication. Error-prone: forget to serialize a field, silent bugs.

**Solution**: Derive Serialize and Deserialize traits using #[derive] macros. Serde generates format-agnostic serialization code. Use field attributes: #[serde(rename)] changes field names in output, #[serde(skip)] omits fields, #[serde(default)] provides defaults for missing fields. Custom serializers via #[serde(serialize_with)] for special types. Serde data model separates type structure from format encoding.

**Why It Matters**: Zero-cost abstraction—compiled code as fast as hand-written. Switch formats by changing serde_json to serde_cbor—one line change. Type safety: deserialization validates types at runtime, catches mismatches. Adding fields doesn't break existing serialization code. Works with 50+ formats (JSON, TOML, YAML, bincode, MessagePack, etc.) with same derive. Reduces bugs: compiler ensures all fields handled.

**Use Cases**: REST APIs (JSON request/response), config files (TOML, YAML), RPC between Rust services (bincode—fastest), cross-language messaging (MessagePack, CBOR), database storage (serialize structs to JSONB), caching (bincode for speed), logging (structured logs to JSON).

### Example: Basic Derive Pattern

Add serialization to custom types with minimal code.

```rust
// Add to Cargo.toml:
// [dependencies]
// serde = { version = "1.0", features = ["derive"] }
// serde_json = "1.0"

use serde::{Serialize, Deserialize};

//==============================================================================
// Deriving Serialize + Deserialize makes this struct work with any serde format
//==============================================================================
#[derive(Serialize, Deserialize, Debug)]
struct Person {
    name: String,
    age: u32,
    email: String,
}

fn basic_serialization() -> Result<(), Box<dyn std::error::Error>> {
    let person = Person {
        name: "Alice".to_string(),
        age: 30,
        email: "alice@example.com".to_string(),
    };

    // Serialize to JSON (compact representation)
    let json = serde_json::to_string(&person)?;
    println!("JSON: {}", json);
    // Output: {"name":"Alice","age":30,"email":"alice@example.com"}

    // Pretty print (human-readable with indentation)
    let json_pretty = serde_json::to_string_pretty(&person)?;
    println!("Pretty JSON:\n{}", json_pretty);

    // Deserialize from JSON back to a Rust struct
    // Serde validates types: wrong types or missing fields → error
    let deserialized: Person = serde_json::from_str(&json)?;
    println!("Deserialized: {:?}", deserialized);

    Ok(())
}
```

### Example: Field Attributes

Field attributes give you fine-grained control over how individual fields are serialized without writing custom code.

**Common use cases:**
- **API compatibility**: Your Rust names don't match the external API (e.g., `user_name` vs `username`)
- **Optional fields**: Omit `None` values to reduce payload size
- **Sensitive data**: Skip serializing passwords or secrets
- **Backward compatibility**: Accept old field names when deserializing
- **Flattening**: Merge nested structs into a flat structure

```rust
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
struct User {
    // Rename: use "username" in JSON, but "name" in Rust code
    #[serde(rename = "username")]
    name: String,

    // Skip serializing if None (reduces JSON size)
    // Field is serialized as "middleName": "value" only when Some
    #[serde(skip_serializing_if = "Option::is_none")]
    middle_name: Option<String>,

    // Provide default value when deserializing if field is missing
    // If JSON doesn't include "age", this becomes 0
    #[serde(default)]
    age: u32,

    // Skip this field entirely (never serialize or deserialize)
    // Useful for runtime-only data or secrets
    #[serde(skip)]
    password_hash: String,

    // Accept multiple names during deserialization
    // Deserializes from "mail", "e-mail", or "email"
    #[serde(alias = "mail", alias = "e-mail")]
    email: String,

    // Flatten: merge nested struct's fields into parent
    // Instead of "metadata": {"created_at": ...}
    // you get "created_at": ... at the top level
    #[serde(flatten)]
    metadata: Metadata,
}

#[derive(Serialize, Deserialize, Debug, Default)]
struct Metadata {
    created_at: Option<String>,
    updated_at: Option<String>,
}

fn field_attributes_example() -> Result<(), Box<dyn std::error::Error>> {
    let user = User {
        name: "Bob".to_string(),
        middle_name: None,  // Won't appear in JSON
        age: 25,
        password_hash: "secret".to_string(),  // Never serialized
        email: "bob@example.com".to_string(),
        metadata: Metadata {
            created_at: Some("2024-01-01".to_string()),
            updated_at: None,  // Won't appear in JSON
        },
    };

    let json = serde_json::to_string_pretty(&user)?;
    println!("{}", json);
    // Output:
    // {
    //   "username": "Bob",
    //   "age": 25,
    //   "email": "bob@example.com",
    //   "created_at": "2024-01-01"
    //==
    // }
    //==
    // Note: middle_name, password_hash, updated_at are omitted

    Ok(())
}
```

### Example: Container Attributes

Container attributes apply to the entire struct or enum, affecting how all fields are handled.

**Common patterns:**
- **Case conversion**: Convert Rust's `snake_case` to `camelCase` for JavaScript APIs
- **Strict deserialization**: Reject unknown fields to catch API changes early
- **Enum representation**: Control how enum variants are encoded

```rust
use serde::{Serialize, Deserialize};

//=============================================
// Rename all fields to camelCase automatically
//=============================================
// Rust: status_code → JSON: statusCode
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ApiResponse {
    status_code: u32,        // → statusCode in JSON
    error_message: Option<String>,  // → errorMessage in JSON
    response_data: Vec<String>,     // → responseData in JSON
}

//===========================================
// Deny unknown fields during deserialization
//===========================================
// Fails if JSON contains fields not defined in the struct
//================================================
// Use this to detect API version mismatches early
//================================================
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct StrictConfig {
    host: String,
    port: u16,
}

//=========================================================
// Tagged enum: adds a "type" field to identify the variant
//=========================================================
// Useful for discriminated unions in JSON
#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
enum Message {
    Text { content: String },
    Image { url: String, width: u32, height: u32 },
    Video { url: String, duration: u32 },
}
//==============================================================================
// Serializes as: {"type": "Image", "url": "...", "width": 1920, "height": 1080}
//==============================================================================

//==============================================================
// Untagged enum: no type field, variant determined by structure
//==============================================================
// Serde tries to deserialize as each variant until one succeeds
#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
enum Value {
    Integer(i64),
    Float(f64),
    String(String),
    Bool(bool),
}
//========================================================
// Serializes as: 42, 3.14, "hello", or true (no type tag)
//========================================================

fn enum_serialization() -> Result<(), Box<dyn std::error::Error>> {
    let message = Message::Image {
        url: "https://example.com/image.jpg".to_string(),
        width: 1920,
        height: 1080,
    };

    let json = serde_json::to_string_pretty(&message)?;
    println!("Tagged enum:\n{}", json);
    // Output:
    // {
    //   "type": "Image",
    //   "url": "https://example.com/image.jpg",
    //   "width": 1920,
    //   "height": 1080
    //==
    // }
    //==

    let value = Value::String("hello".to_string());
    let json = serde_json::to_string(&value)?;
    println!("Untagged enum: {}", json);
    // Output: "hello" (no type information)

    Ok(())
}
```

### Example: Custom Serialization Functions

Sometimes derive attributes aren't enough—you need to transform data during serialization. Custom functions give you precise control.

**Use custom serializers for:**
- **Type conversion**: Serialize `Duration` as seconds instead of nanos
- **Format conversion**: Serialize dates in a specific format
- **Validation**: Ensure data meets constraints during deserialization
- **Legacy compatibility**: Match quirky formats from old systems

```rust
use serde::{Serialize, Deserialize, Serializer, Deserializer};
use serde::de::{self, Visitor};
use std::fmt;

#[derive(Serialize, Deserialize, Debug)]
struct Config {
    // Serialize Duration as seconds (u64) instead of nanos
    // Makes JSON more readable: "timeout": 300 instead of "timeout": 300000000000
    #[serde(serialize_with = "serialize_duration", deserialize_with = "deserialize_duration")]
    timeout: std::time::Duration,

    // Custom date format
    #[serde(serialize_with = "serialize_date", deserialize_with = "deserialize_date")]
    created_at: chrono::NaiveDate,
}

//==============================================
// Convert Duration to seconds for serialization
//==============================================
fn serialize_duration<S>(duration: &std::time::Duration, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    // Convert to seconds and serialize as a simple number
    serializer.serialize_u64(duration.as_secs())
}

//========================================================
// Convert seconds back to Duration during deserialization
//========================================================
fn deserialize_duration<'de, D>(deserializer: D) -> Result<std::time::Duration, D::Error>
where
    D: Deserializer<'de>,
{
    let secs = u64::deserialize(deserializer)?;
    Ok(std::time::Duration::from_secs(secs))
}

//===============================
// Using chrono for date handling
//===============================
use chrono::NaiveDate;

//======================================
// Serialize date as "YYYY-MM-DD" string
//======================================
fn serialize_date<S>(date: &NaiveDate, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&date.format("%Y-%m-%d").to_string())
}

//==========================================
// Deserialize date from "YYYY-MM-DD" string
//==========================================
fn deserialize_date<'de, D>(deserializer: D) -> Result<NaiveDate, D::Error>
where
    D: Deserializer<'de>,
{
    // Implement Visitor pattern for type-safe deserialization
    struct DateVisitor;

    impl<'de> Visitor<'de> for DateVisitor {
        type Value = NaiveDate;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a date string in YYYY-MM-DD format")
        }

        fn visit_str<E>(self, value: &str) -> Result<NaiveDate, E>
        where
            E: de::Error,
        {
            // Parse string, convert parse errors to serde errors
            NaiveDate::parse_from_str(value, "%Y-%m-%d")
                .map_err(|e| E::custom(format!("Invalid date: {}", e)))
        }
    }

    deserializer.deserialize_str(DateVisitor)
}
```

### Example: Custom Serialize/Deserialize Implementation

For complete control, implement `Serialize` and `Deserialize` manually. This is necessary for types with complex invariants or non-standard representations.

**When to write manual implementations:**
- Your type has internal invariants that need validation
- The default serialization doesn't match your needs
- You need to support a legacy format
- You want to serialize computed fields or skip internal state

```rust
use serde::{Serialize, Deserialize, Serializer, Deserializer};
use serde::ser::SerializeStruct;
use serde::de::{self, MapAccess, Visitor};
use std::fmt;

#[derive(Debug)]
struct Point {
    x: f64,
    y: f64,
}

//================================
// Manual Serialize implementation
//================================
impl Serialize for Point {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Serialize as a struct with 2 fields
        let mut state = serializer.serialize_struct("Point", 2)?;
        state.serialize_field("x", &self.x)?;
        state.serialize_field("y", &self.y)?;
        state.end()
    }
}

//==================================
// Manual Deserialize implementation
//==================================
impl<'de> Deserialize<'de> for Point {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        // Define field identifiers
        enum Field { X, Y }

        // Implement Deserialize for Field enum
        impl<'de> Deserialize<'de> for Field {
            fn deserialize<D>(deserializer: D) -> Result<Field, D::Error>
            where
                D: Deserializer<'de>,
            {
                struct FieldVisitor;

                impl<'de> Visitor<'de> for FieldVisitor {
                    type Value = Field;

                    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                        formatter.write_str("`x` or `y`")
                    }

                    fn visit_str<E>(self, value: &str) -> Result<Field, E>
                    where
                        E: de::Error,
                    {
                        match value {
                            "x" => Ok(Field::X),
                            "y" => Ok(Field::Y),
                            _ => Err(de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }

                deserializer.deserialize_identifier(FieldVisitor)
            }
        }

        // Implement visitor for the Point struct
        struct PointVisitor;

        impl<'de> Visitor<'de> for PointVisitor {
            type Value = Point;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct Point")
            }

            fn visit_map<V>(self, mut map: V) -> Result<Point, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut x = None;
                let mut y = None;

                // Read each field from the map
                while let Some(key) = map.next_key()? {
                    match key {
                        Field::X => {
                            if x.is_some() {
                                return Err(de::Error::duplicate_field("x"));
                            }
                            x = Some(map.next_value()?);
                        }
                        Field::Y => {
                            if y.is_some() {
                                return Err(de::Error::duplicate_field("y"));
                            }
                            y = Some(map.next_value()?);
                        }
                    }
                }

                // Ensure required fields are present
                let x = x.ok_or_else(|| de::Error::missing_field("x"))?;
                let y = y.ok_or_else(|| de::Error::missing_field("y"))?;

                Ok(Point { x, y })
            }
        }

        const FIELDS: &[&str] = &["x", "y"];
        deserializer.deserialize_struct("Point", FIELDS, PointVisitor)
    }
}
```

### Example: Serializing with State

Sometimes you need to include computed data or context during serialization. Custom `Serialize` implementations make this possible.

```rust
use serde::{Serialize, Serializer};
use std::collections::HashMap;

struct Database {
    users: HashMap<u64, String>,
}

//=================================================
// Custom serialization that includes computed data
//=================================================
// Adds a "user_count" field that isn't stored in the struct
impl Serialize for Database {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use serde::ser::SerializeStruct;

        let mut state = serializer.serialize_struct("Database", 2)?;
        state.serialize_field("users", &self.users)?;
        // Serialize computed field
        state.serialize_field("user_count", &self.users.len())?;
        state.end()
    }
}

//=========================================
// Wrapper for custom serialization context
//=========================================
// Allows passing configuration to serialization logic
struct SerializeWithContext<'a, T> {
    value: &'a T,
    include_metadata: bool,
}

impl<'a, T: Serialize> Serialize for SerializeWithContext<'a, T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if self.include_metadata {
            // Wrap value with metadata
            use serde::ser::SerializeStruct;
            let mut state = serializer.serialize_struct("WithMetadata", 2)?;
            state.serialize_field("data", self.value)?;
            state.serialize_field("serialized_at", &chrono::Utc::now().to_rfc3339())?;
            state.end()
        } else {
            // Serialize value directly
            self.value.serialize(serializer)
        }
    }
}
```

---

## Pattern 2: Zero-Copy Deserialization

**Problem**: Deserializing allocates—parsing JSON with "name":"Alice" allocates String for "Alice". Processing 100K log lines allocates 100K strings wastefully. Input buffer (file, network) lives long enough to borrow from. Allocation overhead dominates parsing time for small records. Heap allocation in hot path hurts performance. Embedded systems have limited RAM.

**Solution**: Use &str and &[u8] in structs instead of String and Vec. Add #[serde(borrow)] attribute to enable borrowing. Use serde_json::from_slice (not from_str) with byte slices. Lifetime-aware types borrow from input buffer. For nested borrows, use #[serde(borrow)] on container fields. Works when input buffer outlives deserialized data.

**Why It Matters**: 10x faster for large inputs—no heap allocation. Constant memory: O(1) vs O(N) for allocating. Critical for high-throughput: parsing 1M logs with zero-copy uses 10MB, allocating uses 100MB. CPU cache friendly: borrowed data in same memory region. Essential for embedded (limited RAM). Enables zero-allocation parsers for protocols.

**Use Cases**: Log parsing (borrow from mmap'd file), HTTP request parsing (borrow from socket buffer), streaming data (process without allocating), embedded systems (RAM-constrained), high-throughput parsers (network protocols), zero-allocation servers.

### Example: Zero-Copy Borrowing Pattern

**Problem**: Deserialize without allocating by borrowing from input buffer.

```rust
use serde::{Deserialize, Serialize};

//=========================================
// Zero-copy: borrows from the input string
//=========================================
// Lifetimes ensure the struct can't outlive the input
#[derive(Deserialize, Debug)]
struct BorrowedData<'a> {
    // #[serde(borrow)] tells serde to borrow instead of copying
    #[serde(borrow)]
    name: &'a str,

    #[serde(borrow)]
    description: &'a str,

    count: u32,  // Primitive types are always copied
}

fn zero_copy_example() -> Result<(), Box<dyn std::error::Error>> {
    let json = r#"{"name": "Product", "description": "A great product", "count": 42}"#;

    // No string allocation happens here!
    // name and description borrow from the `json` string
    let data: BorrowedData = serde_json::from_str(json)?;

    println!("Name: {}", data.name);
    println!("Description: {}", data.description);
    println!("Count: {}", data.count);

    // data can't outlive json due to lifetime 'a
    // This won't compile: return data;

    Ok(())
}

//============================================
// Cow (Clone on Write) for flexible ownership
//============================================
// Borrows when possible, owns when necessary
#[derive(Deserialize, Serialize, Debug)]
struct FlexibleData<'a> {
    #[serde(borrow)]
    name: std::borrow::Cow<'a, str>,

    #[serde(borrow)]
    tags: std::borrow::Cow<'a, [String]>,
}

fn cow_example() -> Result<(), Box<dyn std::error::Error>> {
    let json = r#"{"name": "Item", "tags": ["tag1", "tag2"]}"#;

    let data: FlexibleData = serde_json::from_str(json)?;

    // Borrowed if input didn't need transformation
    // Owned if input needed unescaping or conversion
    println!("Name: {}", data.name);
    println!("Tags: {:?}", data.tags);

    Ok(())
}
```

### Example: Using Bytes and ByteBuf

Binary data benefits even more from zero-copy deserialization. `serde_bytes` provides specialized handling for byte slices.

```rust
use serde::{Deserialize, Serialize};
use serde_bytes::{ByteBuf, Bytes};

#[derive(Serialize, Deserialize, Debug)]
struct BinaryData<'a> {
    // Serialize as compact binary array instead of JSON array of numbers
    #[serde(with = "serde_bytes")]
    owned_data: Vec<u8>,

    // Borrow from input buffer (zero-copy)
    #[serde(borrow, with = "serde_bytes")]
    borrowed_data: &'a [u8],
}

//===============================
// More efficient for binary data
//===============================
// Without serde_bytes: [1, 2, 3, 4, 5] in JSON (13 bytes)
//============================================================================
// With serde_bytes: "\u0001\u0002\u0003\u0004\u0005" (compact representation)
//============================================================================
#[derive(Serialize, Deserialize, Debug)]
struct OptimizedBinaryData {
    #[serde(with = "serde_bytes")]
    data: Vec<u8>,
}

fn binary_data_example() -> Result<(), Box<dyn std::error::Error>> {
    let data = OptimizedBinaryData {
        data: vec![1, 2, 3, 4, 5],
    };

    // With serde_bytes, binary data is more efficiently encoded
    let json = serde_json::to_string(&data)?;
    println!("Serialized: {}", json);

    Ok(())
}
```

### Example: Zero-Copy with bincode

Bincode is particularly well-suited for zero-copy deserialization because it's a binary format that doesn't need escape sequences or UTF-8 validation.

```rust
//===================
// Add to Cargo.toml:
//===================
// bincode = "1.3"

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
struct Record<'a> {
    id: u64,
    #[serde(borrow)]
    name: &'a str,
    #[serde(borrow)]
    data: &'a [u8],
}

fn bincode_zero_copy() -> Result<(), Box<dyn std::error::Error>> {
    let record = Record {
        id: 123,
        name: "Test",
        data: &[1, 2, 3, 4, 5],
    };

    // Serialize to bytes (very compact)
    let encoded = bincode::serialize(&record)?;

    // Zero-copy deserialization: borrows from `encoded` buffer
    let decoded: Record = bincode::deserialize(&encoded)?;

    println!("Decoded: {:?}", decoded);

    Ok(())
}
```

### Example: Custom Zero-Copy Deserializer

For advanced cases, implement custom deserializers that borrow from the input.

```rust
use serde::de::{self, Deserializer, Visitor};
use std::fmt;

//========================================
// Custom deserializer for borrowed slices
//========================================
fn deserialize_borrowed_slice<'de, D>(deserializer: D) -> Result<&'de [u8], D::Error>
where
    D: Deserializer<'de>,
{
    struct BorrowedSliceVisitor;

    impl<'de> Visitor<'de> for BorrowedSliceVisitor {
        type Value = &'de [u8];

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a borrowed byte slice")
        }

        // This borrows directly from the input buffer
        fn visit_borrowed_bytes<E>(self, v: &'de [u8]) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(v)
        }
    }

    deserializer.deserialize_bytes(BorrowedSliceVisitor)
}

#[derive(Deserialize)]
struct CustomBorrowed<'a> {
    #[serde(deserialize_with = "deserialize_borrowed_slice")]
    data: &'a [u8],
}
```

---

## Pattern 3: Schema Evolution

**Problem**: API changes break clients—adding "phone" field to User fails deserialization. Renaming "username" to "user_name" breaks all existing JSON. Old clients can't parse new responses. Database schema migrations require downtime. Version v2 incompatible with v1 clients. Refactoring field names breaks wire format. Gradual rollout impossible: update breaks old clients immediately.

**Solution**: Use #[serde(default)] for new optional fields—deserializes missing as default(). Use #[serde(rename = "old_name")] to keep wire format when refactoring. Use #[serde(alias = "old")] to accept both old and new names during transition. Option<T> for truly optional fields. Versioning with tagged enums for breaking changes. Flatten attributes merge nested structs.

**Why It Matters**: Enables gradual rollout—old clients work with new servers during migration. Adding fields backward compatible: v1 clients ignore new fields, v2 clients get defaults. Renaming internally doesn't break API: rename="..." preserves wire format. Allows versioned APIs (v1/v2 coexist). Database migrations non-breaking. Essential for production: can't coordinate simultaneous updates of all clients.

**Use Cases**: Versioned REST APIs (v1→v2 migration), database schema migrations (add columns without breaking old code), config file evolution (new options without breaking existing configs), backward-compatible protocols, gradual service updates (rolling deployment), refactoring without API breaks.

### Example: Optional Field Pattern

 Add new fields without breaking existing serialized data.

```rust
use serde::{Deserialize, Serialize};

//===========================
// Version 1: Original schema
//===========================
#[derive(Serialize, Deserialize, Debug)]
struct ConfigV1 {
    host: String,
    port: u16,
}

//===========================================
// Version 2: Add optional field with default
//===========================================
// Can deserialize V1 data without errors
#[derive(Serialize, Deserialize, Debug)]
struct ConfigV2 {
    host: String,
    port: u16,

    // New field - defaults to None if missing
    // Old JSON without "timeout" → timeout: None
    #[serde(default)]
    timeout: Option<u32>,
}

//=======================================
// Version 3: Required field with default
//=======================================
#[derive(Serialize, Deserialize, Debug)]
struct ConfigV3 {
    host: String,
    port: u16,

    #[serde(default)]
    timeout: Option<u32>,

    // Defaults to 10 if missing
    // Allows reading old configs without this field
    #[serde(default = "default_max_connections")]
    max_connections: u32,
}

fn default_max_connections() -> u32 {
    10
}

fn schema_evolution_example() -> Result<(), Box<dyn std::error::Error>> {
    // Old JSON (v1) can be deserialized into new struct (v3)
    let old_json = r#"{"host": "localhost", "port": 8080}"#;
    let config: ConfigV3 = serde_json::from_str(old_json)?;

    println!("Config: {:?}", config);
    println!("Max connections (defaulted): {}", config.max_connections);
    // Output: max_connections: 10 (from default function)

    Ok(())
}
```

### Example: Versioned Enums

```rust
use serde::{Deserialize, Serialize};

//=======================================================
// Tag-based versioning: each variant is a schema version
//=======================================================
// The "version" field discriminates between versions
#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "version")]
enum VersionedMessage {
    #[serde(rename = "1")]
    V1 { content: String },

    #[serde(rename = "2")]
    V2 {
        content: String,
        timestamp: u64,
    },

    #[serde(rename = "3")]
    V3 {
        content: String,
        timestamp: u64,
        metadata: std::collections::HashMap<String, String>,
    },
}

impl VersionedMessage {
    // Migrate any version to the latest
    fn to_latest(self) -> MessageV3 {
        match self {
            VersionedMessage::V1 { content } => MessageV3 {
                content,
                timestamp: 0,  // Default for old messages
                metadata: Default::default(),
            },
            VersionedMessage::V2 { content, timestamp } => MessageV3 {
                content,
                timestamp,
                metadata: Default::default(),
            },
            VersionedMessage::V3 { content, timestamp, metadata } => MessageV3 {
                content,
                timestamp,
                metadata,
            },
        }
    }
}

#[derive(Debug)]
struct MessageV3 {
    content: String,
    timestamp: u64,
    metadata: std::collections::HashMap<String, String>,
}
```

### Example: Handling Renamed Fields

```rust
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
struct UserProfile {
    // Accept both old and new names during deserialization
    // Deserializes from "user_name", "userName", or "name"
    #[serde(alias = "user_name", alias = "userName")]
    name: String,

    // Serialize as "emailAddress", accept "email" or "emailAddress" when deserializing
    // This allows gradual migration: old clients use "email", new ones use "emailAddress"
    #[serde(rename = "emailAddress", alias = "email")]
    email_address: String,
}

fn renamed_fields_example() -> Result<(), Box<dyn std::error::Error>> {
    // Old format (uses old field names)
    let old_json = r#"{"user_name": "Alice", "email": "alice@example.com"}"#;
    let profile: UserProfile = serde_json::from_str(old_json)?;

    // New format (uses new field names)
    let new_json = serde_json::to_string_pretty(&profile)?;
    println!("New format:\n{}", new_json);
    // Output uses renamed fields:
    // {
    //   "name": "Alice",
    //   "emailAddress": "alice@example.com"
    //==
    // }
    //==

    Ok(())
}
```

### Example: Custom Migration Logic

For complex migrations, implement custom deserialization logic.

```rust
use serde::{Deserialize, Deserializer};
use serde::de::{self, MapAccess, Visitor};
use std::fmt;

#[derive(Debug)]
struct MigratableConfig {
    host: String,
    port: u16,
    connection_string: String,
}

impl<'de> Deserialize<'de> for MigratableConfig {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(field_identifier, rename_all = "snake_case")]
        enum Field { Host, Port, ConnectionString }

        struct ConfigVisitor;

        impl<'de> Visitor<'de> for ConfigVisitor {
            type Value = MigratableConfig;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct MigratableConfig")
            }

            fn visit_map<V>(self, mut map: V) -> Result<MigratableConfig, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut host = None;
                let mut port = None;
                let mut connection_string = None;

                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Host => host = Some(map.next_value()?),
                        Field::Port => port = Some(map.next_value()?),
                        Field::ConnectionString => connection_string = Some(map.next_value()?),
                    }
                }

                // Migration logic: build connection_string from host and port if missing
                // This allows old configs (with host+port) to work with new code (expects connection_string)
                let connection_string = if let Some(cs) = connection_string {
                    cs
                } else {
                    let host = host.ok_or_else(|| de::Error::missing_field("host"))?;
                    let port = port.ok_or_else(|| de::Error::missing_field("port"))?;
                    format!("{}:{}", host, port)
                };

                let host = host.ok_or_else(|| de::Error::missing_field("host"))?;
                let port = port.ok_or_else(|| de::Error::missing_field("port"))?;

                Ok(MigratableConfig {
                    host,
                    port,
                    connection_string,
                })
            }
        }

        deserializer.deserialize_struct(
            "MigratableConfig",
            &["host", "port", "connection_string"],
            ConfigVisitor,
        )
    }
}
```

---

## Pattern 4: Binary vs Text Formats

**Problem**: JSON human-readable but large and slow—100KB JSON → 40KB binary. Need cross-language format (bincode Rust-only). Size matters for bandwidth/storage costs. Parse speed critical for high-throughput. Want debuggability (text) but need performance (binary). Tradeoff between human-readable vs compact. Self-describing (MessagePack) vs minimal (bincode).

**Solution**: Use JSON for APIs and debugging (human-readable, universal). Use bincode for Rust-to-Rust IPC (smallest, fastest—10x faster than JSON). Use MessagePack/CBOR for cross-language binary (60% of JSON size, many language bindings). Use TOML for simple configs (readable, minimal). Use YAML for complex nested configs. Choose based on: readability needs, size constraints, parse speed, interop requirements.

**Why It Matters**: JSON 2-5x larger than binary (100KB → 40KB MessagePack). Bincode 10x faster parse than JSON for Rust types. Bandwidth costs: binary saves 60% network transfer. Storage: binary DB fields smaller. Latency: faster parse means lower tail latency. Debugging: text formats inspectable. Cross-language: MessagePack/CBOR work everywhere, bincode Rust-only. Format choice directly impacts performance.

**Use Cases**: JSON (REST APIs, web configs, debugging), bincode (Rust microservice IPC, caching, session storage), MessagePack (cross-language RPC, binary APIs), CBOR (IoT protocols, embedded systems), TOML (application configs), YAML (complex configs like Kubernetes), Protocol Buffers (Google services, strict schemas).

### Example: Format Comparison Pattern

Choose optimal serialization format for use case.

| Format      | Size | Speed | Human-readable | Interop | Self-describing |
|-------------|------|-------|----------------|---------|-----------------|
| JSON        | Large| Slow  | Yes            | Excellent| Yes            |
| Bincode     | Tiny | Fast  | No             | Rust-only| No             |
| MessagePack | Small| Fast  | No             | Excellent| Yes            |
| CBOR        | Small| Fast  | No             | Good    | Yes             |
| YAML        | Large| Slow  | Yes            | Good    | Yes             |
| TOML        | Medium| Medium| Yes           | Good    | Yes             |

### Example: JSON Pattern

Need human-readable format for APIs and configs.

**Use JSON when:**
- Building web APIs (de facto standard)
- Storing human-editable config files
- Debugging (can inspect payloads easily)
- Interoperating with JavaScript/browsers

```rust
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
struct Product {
    id: u64,
    name: String,
    price: f64,
    in_stock: bool,
}

fn json_format() -> Result<(), Box<dyn std::error::Error>> {
    let product = Product {
        id: 12345,
        name: "Widget".to_string(),
        price: 29.99,
        in_stock: true,
    };

    // Serialize to JSON with pretty formatting
    let json = serde_json::to_string_pretty(&product)?;
    println!("JSON ({} bytes):\n{}", json.len(), json);
    // Output (~80 bytes with whitespace):
    // {
    //   "id": 12345,
    //   "name": "Widget",
    //   "price": 29.99,
    //   "in_stock": true
    //==
    // }
    //==

    // Deserialize back to Rust struct
    let deserialized: Product = serde_json::from_str(&json)?;
    println!("Deserialized: {:?}", deserialized);

    Ok(())
}
```

### Example: Bincode (Binary Format)

Bincode is the most compact binary format for Rust-to-Rust communication. Not self-describing—you must know the exact type to deserialize.

**Use Bincode when:**
- Communicating between Rust services
- Storing data where you control both writer and reader
- Maximum performance is critical
- Size matters (smallest format)

```rust
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
struct Product {
    id: u64,
    name: String,
    price: f64,
    in_stock: bool,
}

fn bincode_format() -> Result<(), Box<dyn std::error::Error>> {
    let product = Product {
        id: 12345,
        name: "Widget".to_string(),
        price: 29.99,
        in_stock: true,
    };

    // Serialize to compact binary
    let encoded = bincode::serialize(&product)?;
    println!("Bincode ({} bytes): {:?}", encoded.len(), encoded);
    // Output: ~30 bytes (vs ~80 bytes JSON)

    // Deserialize from binary
    let decoded: Product = bincode::deserialize(&encoded)?;
    println!("Deserialized: {:?}", decoded);

    Ok(())
}
```

### Example: MessagePack (Binary Format)

MessagePack is a binary format with broad language support. Good balance between size, speed, and interoperability.

**Use MessagePack when:**
- Building cross-language binary protocols
- Need compact format with better interop than Bincode
- Real-time systems (gaming, IoT)

```rust
//===================
// Add to Cargo.toml:
//===================
// rmp-serde = "1.1"

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
struct Product {
    id: u64,
    name: String,
    price: f64,
    in_stock: bool,
}

fn messagepack_format() -> Result<(), Box<dyn std::error::Error>> {
    let product = Product {
        id: 12345,
        name: "Widget".to_string(),
        price: 29.99,
        in_stock: true,
    };

    // Serialize to MessagePack
    let encoded = rmp_serde::to_vec(&product)?;
    println!("MessagePack ({} bytes): {:?}", encoded.len(), encoded);
    // Output: ~35 bytes (compact, self-describing)

    // Deserialize
    let decoded: Product = rmp_serde::from_slice(&encoded)?;
    println!("Deserialized: {:?}", decoded);

    Ok(())
}
```

### Example: CBOR (Binary Format)

CBOR (Concise Binary Object Representation) is similar to MessagePack but with more features (tags, indefinite-length encoding). Used in IoT and embedded systems.

```rust
//===================
// Add to Cargo.toml:
//===================
// serde_cbor = "0.11"

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
struct Product {
    id: u64,
    name: String,
    price: f64,
    in_stock: bool,
}

fn cbor_format() -> Result<(), Box<dyn std::error::Error>> {
    let product = Product {
        id: 12345,
        name: "Widget".to_string(),
        price: 29.99,
        in_stock: true,
    };

    // Serialize to CBOR
    let encoded = serde_cbor::to_vec(&product)?;
    println!("CBOR ({} bytes): {:?}", encoded.len(), encoded);

    // Deserialize
    let decoded: Product = serde_cbor::from_slice(&encoded)?;
    println!("Deserialized: {:?}", decoded);

    Ok(())
}
```

### Example: YAML (Text Format)

YAML is very human-readable with minimal syntax. Great for config files, but the complex spec makes parsing slow and error-prone.

```rust
//===================
// Add to Cargo.toml:
//===================
// serde_yaml = "0.9"

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
struct Product {
    id: u64,
    name: String,
    price: f64,
    in_stock: bool,
}

fn yaml_format() -> Result<(), Box<dyn std::error::Error>> {
    let product = Product {
        id: 12345,
        name: "Widget".to_string(),
        price: 29.99,
        in_stock: true,
    };

    // Serialize to YAML
    let yaml = serde_yaml::to_string(&product)?;
    println!("YAML ({} bytes):\n{}", yaml.len(), yaml);
    // Output:
    // id: 12345
    // name: Widget
    // price: 29.99
    // in_stock: true

    // Deserialize
    let deserialized: Product = serde_yaml::from_str(&yaml)?;
    println!("Deserialized: {:?}", deserialized);

    Ok(())
}
```

### Example: TOML (Text Format)

TOML is designed for config files. Minimal, unambiguous syntax. Limited nesting makes it unsuitable for complex data structures.

```rust
//===================
// Add to Cargo.toml:
//===================
// toml = "0.8"

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
struct Config {
    database: DatabaseConfig,
    server: ServerConfig,
}

#[derive(Serialize, Deserialize, Debug)]
struct DatabaseConfig {
    host: String,
    port: u16,
    username: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct ServerConfig {
    host: String,
    port: u16,
    workers: u32,
}

fn toml_format() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config {
        database: DatabaseConfig {
            host: "localhost".to_string(),
            port: 5432,
            username: "admin".to_string(),
        },
        server: ServerConfig {
            host: "0.0.0.0".to_string(),
            port: 8080,
            workers: 4,
        },
    };

    // Serialize to TOML
    let toml = toml::to_string_pretty(&config)?;
    println!("TOML ({} bytes):\n{}", toml.len(), toml);
    // Output:
    // [database]
    // host = "localhost"
    // port = 5432
    // username = "admin"
    //
    // [server]
    // host = "0.0.0.0"
    // port = 8080
    // workers = 4

    // Deserialize
    let deserialized: Config = toml::from_str(&toml)?;
    println!("Deserialized: {:?}", deserialized);

    Ok(())
}
```

### Example: Format Comparison

Benchmark different formats to see the size difference:

```rust
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
struct BenchmarkData {
    id: u64,
    name: String,
    values: Vec<f64>,
    metadata: std::collections::HashMap<String, String>,
}

fn format_comparison() -> Result<(), Box<dyn std::error::Error>> {
    let data = BenchmarkData {
        id: 12345,
        name: "Test Data".to_string(),
        values: vec![1.0, 2.0, 3.0, 4.0, 5.0],
        metadata: {
            let mut map = std::collections::HashMap::new();
            map.insert("key1".to_string(), "value1".to_string());
            map.insert("key2".to_string(), "value2".to_string());
            map
        },
    };

    // Compare sizes across formats
    let json = serde_json::to_string(&data)?;
    println!("JSON: {} bytes", json.len());

    let bincode = bincode::serialize(&data)?;
    println!("Bincode: {} bytes", bincode.len());

    let msgpack = rmp_serde::to_vec(&data)?;
    println!("MessagePack: {} bytes", msgpack.len());

    let cbor = serde_cbor::to_vec(&data)?;
    println!("CBOR: {} bytes", cbor.len());

    println!("\nBinary formats are typically 30-50% smaller than JSON");
    // Typical output:
    // JSON: 120 bytes
    // Bincode: 65 bytes (45% smaller)
    // MessagePack: 75 bytes (38% smaller)
    // CBOR: 78 bytes (35% smaller)

    Ok(())
}
```

---

## Pattern 5: Streaming Serialization

**Problem**: Serializing GB dataset exhausts memory—loading 10GB JSON into RAM fails. Can't process files larger than RAM. Parsing large JSON array allocates all elements at once. Need to start processing before all data arrives. Latency: waiting for entire response before parsing first item. Backpressure: fast producer overwhelms slow consumer. All-at-once deserialization doesn't fit memory constraints.

**Solution**: Use streaming APIs—serialize/deserialize incrementally. serde_json::Deserializer::from_reader with streaming_iterator pulls one item at time. Write iterators serialize as you iterate, not buffer-then-write. For JSON arrays, use StreamDeserializer to yield elements lazily. CSV processing with serde streaming reads row-by-row. Combine with channels for backpressure.

**Why It Matters**: O(1) memory vs O(N) for full load—process 10GB file in 10MB RAM. Enables processing files larger than RAM (logs, DB exports). Lower latency: start processing first item immediately, don't wait for full download. Backpressure: slow consumer doesn't cause producer to OOM. Essential for logs/analytics. Streaming servers (SSE, WebSocket) can't buffer all messages.

**Use Cases**: Large file processing (GB log files, database dumps), streaming APIs (server-sent events, WebSocket messages), incremental parsing (start processing before download completes), log aggregation (process logs as they arrive), ETL pipelines (transform data in stream), real-time analytics (process events as they occur).

### Example: Streaming JSON Pattern
Process large JSON arrays without loading entire array into memory.

```rust
use serde::Serialize;
use std::io::{self, Write};

#[derive(Serialize)]
struct Record {
    id: u64,
    name: String,
    value: f64,
}

//===========================================================================
// Stream records as a JSON array without building the entire array in memory
//===========================================================================
fn stream_json_array<W: Write>(mut writer: W, records: &[Record]) -> io::Result<()> {
    writer.write_all(b"[")?;

    for (i, record) in records.iter().enumerate() {
        if i > 0 {
            writer.write_all(b",")?;
        }

        // Serialize each record individually
        let json = serde_json::to_string(record)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        writer.write_all(json.as_bytes())?;
    }

    writer.write_all(b"]")?;
    writer.flush()?;

    Ok(())
}

fn streaming_array_example() -> io::Result<()> {
    let records = vec![
        Record { id: 1, name: "Alice".to_string(), value: 100.0 },
        Record { id: 2, name: "Bob".to_string(), value: 200.0 },
        Record { id: 3, name: "Carol".to_string(), value: 300.0 },
    ];

    let mut output = Vec::new();
    stream_json_array(&mut output, &records)?;

    println!("Streamed JSON: {}", String::from_utf8_lossy(&output));

    Ok(())
}
```

### Example: Streaming to File

JSON Lines (newline-delimited JSON) is perfect for streaming: one JSON object per line.

```rust
use serde::Serialize;
use std::fs::File;
use std::io::{self, BufWriter, Write};

#[derive(Serialize)]
struct LogEntry {
    timestamp: u64,
    level: String,
    message: String,
}

//===============================================
// Stream log entries to file (JSON Lines format)
//===============================================
fn stream_to_file(path: &str) -> io::Result<()> {
    let file = File::create(path)?;
    let mut writer = BufWriter::new(file);

    // Write JSON lines (one JSON object per line)
    for i in 0..1000 {
        let entry = LogEntry {
            timestamp: i,
            level: "INFO".to_string(),
            message: format!("Log message {}", i),
        };

        let json = serde_json::to_string(&entry)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        writeln!(writer, "{}", json)?;
    }

    writer.flush()?;

    Ok(())
}
```

### Example: Streaming Deserialization

```rust
use serde::Deserialize;
use std::io::{self, BufRead, BufReader};
use std::fs::File;

#[derive(Deserialize, Debug)]
struct LogEntry {
    timestamp: u64,
    level: String,
    message: String,
}

//===============================
// Stream-process JSON Lines file
//===============================
fn stream_from_file(path: &str) -> io::Result<()> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    // Process one line at a time (constant memory usage)
    for (line_num, line) in reader.lines().enumerate() {
        let line = line?;

        match serde_json::from_str::<LogEntry>(&line) {
            Ok(entry) => {
                // Process entry (filter, transform, aggregate, etc.)
                println!("Entry {}: {:?}", line_num, entry);
            }
            Err(e) => {
                eprintln!("Error parsing line {}: {}", line_num, e);
            }
        }
    }

    Ok(())
}
```

### Example: Streaming with serde_json::Deserializer

serde_json provides a streaming deserializer for processing multiple JSON values.

```rust
use serde::Deserialize;
use std::io::{self, Cursor};

#[derive(Deserialize, Debug)]
struct Item {
    id: u64,
    name: String,
}

fn streaming_deserializer() -> Result<(), Box<dyn std::error::Error>> {
    // Multiple JSON objects (not in an array)
    let json = r#"
        {"id": 1, "name": "Item 1"}
        {"id": 2, "name": "Item 2"}
        {"id": 3, "name": "Item 3"}
    "#;

    let cursor = Cursor::new(json);
    let deserializer = serde_json::Deserializer::from_reader(cursor);

    // Stream items one at a time
    for item in deserializer.into_iter::<Item>() {
        match item {
            Ok(item) => println!("Deserialized: {:?}", item),
            Err(e) => eprintln!("Error: {}", e),
        }
    }

    Ok(())
}
```

### Example: Async Streaming with Tokio

Combine streaming serialization with async I/O for maximum efficiency.

```rust
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::fs::File;

#[derive(Serialize, Deserialize, Debug)]
struct Record {
    id: u64,
    data: String,
}

//======================
// Async streaming write
//======================
async fn async_stream_write(path: &str) -> tokio::io::Result<()> {
    let mut file = File::create(path).await?;

    for i in 0..100 {
        let record = Record {
            id: i,
            data: format!("Data {}", i),
        };

        let json = serde_json::to_string(&record)
            .map_err(|e| tokio::io::Error::new(tokio::io::ErrorKind::Other, e))?;

        file.write_all(json.as_bytes()).await?;
        file.write_all(b"\n").await?;
    }

    file.flush().await?;
    Ok(())
}

//=====================
// Async streaming read
//=====================
async fn async_stream_read(path: &str) -> tokio::io::Result<()> {
    let file = File::open(path).await?;
    let reader = BufReader::new(file);
    let mut lines = reader.lines();

    while let Some(line) = lines.next_line().await? {
        match serde_json::from_str::<Record>(&line) {
            Ok(record) => println!("Record: {:?}", record),
            Err(e) => eprintln!("Parse error: {}", e),
        }
    }

    Ok(())
}
```

### Example: Large Dataset Streaming

For very large datasets, implement custom streaming writers with buffering and periodic flushing.

```rust
use serde::Serialize;
use std::io::{self, Write};

#[derive(Serialize)]
struct DataPoint {
    x: f64,
    y: f64,
    timestamp: u64,
}

//================================================
// Custom streaming writer with automatic flushing
//================================================
struct DataStreamWriter<W: Write> {
    writer: W,
    count: usize,
}

impl<W: Write> DataStreamWriter<W> {
    fn new(mut writer: W) -> io::Result<Self> {
        writer.write_all(b"[")?;
        Ok(DataStreamWriter { writer, count: 0 })
    }

    fn write_point(&mut self, point: &DataPoint) -> io::Result<()> {
        if self.count > 0 {
            self.writer.write_all(b",")?;
        }

        let json = serde_json::to_string(point)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        self.writer.write_all(json.as_bytes())?;
        self.count += 1;

        // Flush every 100 records to balance memory and I/O
        if self.count % 100 == 0 {
            self.writer.flush()?;
        }

        Ok(())
    }

    fn finish(mut self) -> io::Result<()> {
        self.writer.write_all(b"]")?;
        self.writer.flush()?;
        Ok(())
    }
}

fn stream_large_dataset() -> io::Result<()> {
    let file = std::fs::File::create("dataset.json")?;
    let mut writer = DataStreamWriter::new(file)?;

    // Stream 1 million points without loading them all into memory
    for i in 0..1_000_000 {
        let point = DataPoint {
            x: i as f64,
            y: (i as f64).sin(),
            timestamp: i,
        };

        writer.write_point(&point)?;
    }

    writer.finish()?;

    Ok(())
}
```

### Example: Custom Streaming Format

For maximum efficiency, implement length-prefixed binary streaming.

```rust
use serde::Serialize;
use std::io::{self, Write};

//============================================
// Length-prefixed binary format for streaming
//============================================
struct BinaryStreamWriter<W: Write> {
    writer: W,
}

impl<W: Write> BinaryStreamWriter<W> {
    fn new(writer: W) -> Self {
        BinaryStreamWriter { writer }
    }

    fn write_record<T: Serialize>(&mut self, record: &T) -> io::Result<()> {
        // Serialize to bytes
        let bytes = bincode::serialize(record)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

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

#[derive(Serialize)]
struct Message {
    id: u64,
    content: String,
}

fn binary_streaming_example() -> io::Result<()> {
    let mut writer = BinaryStreamWriter::new(Vec::new());

    for i in 0..10 {
        let msg = Message {
            id: i,
            content: format!("Message {}", i),
        };
        writer.write_record(&msg)?;
    }

    writer.flush()?;

    Ok(())
}
```

---

## Summary

This chapter covered serialization patterns using serde:

1. **Serde Patterns**: Derive Serialize/Deserialize, field attributes (rename, skip, default), custom serializers
2. **Zero-Copy Deserialization**: Borrow from input with &str, #[serde(borrow)], 10x faster, O(1) memory
3. **Schema Evolution**: #[serde(default)] for new fields, rename/alias for compatibility, versioned enums
4. **Binary vs Text Formats**: JSON (readable), bincode (smallest/fastest), MessagePack (cross-language binary)
5. **Streaming Serialization**: StreamDeserializer, process GB files in MB RAM, incremental parsing

**Key Takeaways**:
- Serde separates data structures from formats—one derive, all formats
- Zero-cost abstraction: compiled code as fast as hand-written
- Zero-copy deserialization 10x faster with O(1) memory
- Schema evolution via default/rename/alias enables gradual rollout
- Binary formats 2-5x smaller, 10x faster than JSON
- Streaming essential for large files (process > RAM size)

**Format Selection Guide**:
- **JSON**: REST APIs, debugging, human-readable configs
- **Bincode**: Rust-to-Rust IPC, caching (smallest, fastest)
- **MessagePack/CBOR**: Cross-language binary RPC
- **TOML**: Simple application configs
- **YAML**: Complex nested configs (Kubernetes)

**Performance Guidelines**:
- Use zero-copy (&str) for high-throughput parsing
- Binary formats for bandwidth/storage-constrained
- Streaming for files > available RAM
- JSON for debugging/development, binary for production

**Production Patterns**:
- Schema evolution with #[serde(default)] for backward compatibility
- Versioned APIs with tagged enums
- Zero-copy for log parsing (10x throughput)
- Streaming for large dataset processing
- Format-agnostic types (support multiple formats)

**Common Mistakes**:
- Forgetting #[serde(default)] when adding fields → breaks old data
- Using String when &str would work → unnecessary allocation
- Loading entire file before parsing → OOM for large files
- Not versioning schemas → breaking changes painful
- Choosing wrong format (JSON for everything) → performance problems

## Serde Cheat Sheet

```rust
use serde::{Serialize, Deserialize, Serializer, Deserializer};
use serde_json;
use serde_yaml;
use toml;

// ===== BASIC DERIVE MACROS =====
#[derive(Serialize, Deserialize)]                   // Basic serialization
struct Person {
    name: String,
    age: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]     // Common derives
struct Data {
    field: String,
}

// ===== JSON SERIALIZATION =====
// Serialize to JSON
let person = Person { name: "Alice".into(), age: 30 };
let json = serde_json::to_string(&person)?          // Compact JSON string
let json = serde_json::to_string_pretty(&person)?   // Pretty-printed JSON
let json = serde_json::to_vec(&person)?             // JSON as Vec<u8>
let json = serde_json::to_vec_pretty(&person)?      // Pretty Vec<u8>
serde_json::to_writer(writer, &person)?             // Write to io::Write
serde_json::to_writer_pretty(writer, &person)?      // Pretty write

// Deserialize from JSON
let person: Person = serde_json::from_str(json_str)?; // From &str
let person: Person = serde_json::from_slice(bytes)?;  // From &[u8]
let person: Person = serde_json::from_reader(reader)?; // From io::Read

// JSON Value (dynamic/untyped)
use serde_json::Value;
let v: Value = serde_json::from_str(r#"{"a": 1}"#)?;
v["a"]                                               // Access by key
v.get("a")                                           // Safe access, returns Option
v.as_i64()                                           // Convert to i64
v.as_str()                                           // Convert to &str
v.is_object()                                        // Check type
v.is_array()                                         // Check if array
v.is_null()                                          // Check if null

// Construct JSON Value
let v = serde_json::json!({                         // json! macro
    "name": "Alice",
    "age": 30,
    "active": true,
    "items": [1, 2, 3]
});

// ===== FIELD ATTRIBUTES =====
#[derive(Serialize, Deserialize)]
struct User {
    #[serde(rename = "userName")]                   // Rename field in serialized form
    user_name: String,
    
    #[serde(skip)]                                  // Skip serialization/deserialization
    internal: String,
    
    #[serde(skip_serializing)]                      // Skip only when serializing
    password: String,
    
    #[serde(skip_deserializing)]                    // Skip only when deserializing
    computed: String,
    
    #[serde(default)]                               // Use Default::default() if missing
    count: u32,
    
    #[serde(default = "default_age")]               // Custom default function
    age: u32,
    
    #[serde(skip_serializing_if = "Option::is_none")] // Skip if None
    optional: Option<String>,
    
    #[serde(skip_serializing_if = "Vec::is_empty")] // Skip if empty
    tags: Vec<String>,
    
    #[serde(flatten)]                               // Flatten nested struct
    metadata: Metadata,
    
    #[serde(with = "custom_serializer")]            // Use custom serializer
    special: SpecialType,
    
    #[serde(serialize_with = "serialize_fn")]       // Custom serialize only
    #[serde(deserialize_with = "deserialize_fn")]   // Custom deserialize only
    custom: CustomType,
}

fn default_age() -> u32 { 18 }

// ===== CONTAINER ATTRIBUTES =====
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]                  // Rename all fields
struct Config {
    user_name: String,       // Becomes "userName"
    api_key: String,         // Becomes "apiKey"
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]                 // snake_case
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]       // SCREAMING_SNAKE_CASE
#[serde(rename_all = "kebab-case")]                 // kebab-case
#[serde(rename_all = "PascalCase")]                 // PascalCase

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]                       // Error on unknown fields
struct Strict {
    field: String,
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]                              // Tagged enum
enum Message {
    Request { id: u32, body: String },
    Response { id: u32, result: String },
}
// Serializes as: {"type": "Request", "id": 1, "body": "..."}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]            // Adjacently tagged
enum Event {
    Click(u32),
    Scroll { x: i32, y: i32 },
}
// Serializes as: {"type": "Click", "data": 42}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]                                  // Untagged enum (tries each variant)
enum Value {
    Integer(i64),
    String(String),
    Array(Vec<Value>),
}

// ===== ENUM REPRESENTATIONS =====
#[derive(Serialize, Deserialize)]
enum Status {
    Active,                                          // Default: "Active"
    Inactive,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]                  // Lowercase variants
enum State {
    Running,    // "running"
    Stopped,    // "stopped"
}

// Enum with data
#[derive(Serialize, Deserialize)]
enum Action {
    Move { x: i32, y: i32 },
    Click(u32, u32),
    None,
}

// ===== CUSTOM SERIALIZATION =====
// Custom serialize function
fn serialize_fn<S>(value: &CustomType, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&value.to_string())
}

// Custom deserialize function
fn deserialize_fn<'de, D>(deserializer: D) -> Result<CustomType, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    CustomType::from_str(&s).map_err(serde::de::Error::custom)
}

// Implementing Serialize manually
impl Serialize for MyType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("MyType", 2)?;
        state.serialize_field("field1", &self.field1)?;
        state.serialize_field("field2", &self.field2)?;
        state.end()
    }
}

// Implementing Deserialize manually
impl<'de> Deserialize<'de> for MyType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(field_identifier, rename_all = "lowercase")]
        enum Field { Field1, Field2 }
        
        struct MyTypeVisitor;
        
        impl<'de> serde::de::Visitor<'de> for MyTypeVisitor {
            type Value = MyType;
            
            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct MyType")
            }
            
            fn visit_map<V>(self, mut map: V) -> Result<MyType, V::Error>
            where
                V: serde::de::MapAccess<'de>,
            {
                let mut field1 = None;
                let mut field2 = None;
                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Field1 => field1 = Some(map.next_value()?),
                        Field::Field2 => field2 = Some(map.next_value()?),
                    }
                }
                Ok(MyType {
                    field1: field1.ok_or_else(|| serde::de::Error::missing_field("field1"))?,
                    field2: field2.ok_or_else(|| serde::de::Error::missing_field("field2"))?,
                })
            }
        }
        
        deserializer.deserialize_struct("MyType", &["field1", "field2"], MyTypeVisitor)
    }
}

// ===== YAML SERIALIZATION =====
let yaml = serde_yaml::to_string(&data)?            // To YAML string
let yaml = serde_yaml::to_vec(&data)?               // To Vec<u8>
serde_yaml::to_writer(writer, &data)?               // Write to io::Write

let data: MyStruct = serde_yaml::from_str(yaml)?    // From &str
let data: MyStruct = serde_yaml::from_slice(bytes)?; // From &[u8]
let data: MyStruct = serde_yaml::from_reader(reader)?; // From io::Read

// ===== TOML SERIALIZATION =====
let toml = toml::to_string(&data)?                  // To TOML string
let toml = toml::to_string_pretty(&data)?           // Pretty TOML
let toml = toml::to_vec(&data)?                     // To Vec<u8>

let data: MyStruct = toml::from_str(toml)?          // From &str
let data: MyStruct = toml::from_slice(bytes)?       // From &[u8]

// TOML Value (dynamic)
use toml::Value;
let v: Value = toml::from_str(toml_str)?;

// ===== MESSAGEPACK (requires rmp-serde) =====
use rmp_serde;
let bytes = rmp_serde::to_vec(&data)?               // To MessagePack
let data: MyStruct = rmp_serde::from_slice(&bytes)?; // From MessagePack

// ===== BINCODE (binary format) =====
use bincode;
let bytes = bincode::serialize(&data)?              // To binary
let data: MyStruct = bincode::deserialize(&bytes)?  // From binary

// ===== WORKING WITH HASHMAPS =====
use std::collections::HashMap;

#[derive(Serialize, Deserialize)]
struct Config {
    #[serde(flatten)]
    extra: HashMap<String, String>,                  // Capture unknown fields
}

// ===== LIFETIMES =====
#[derive(Serialize, Deserialize)]
struct Borrowed<'a> {
    #[serde(borrow)]
    name: &'a str,                                   // Borrow during deserialization
}

#[derive(Serialize, Deserialize)]
struct WithCow<'a> {
    #[serde(borrow)]
    data: Cow<'a, str>,                              // Cow for efficient borrowing
}

// ===== COMMON PATTERNS =====
// Optional fields with defaults
#[derive(Serialize, Deserialize)]
struct Settings {
    #[serde(default = "default_timeout")]
    timeout: u64,
    
    #[serde(default)]
    verbose: bool,                                   // false if missing
}

fn default_timeout() -> u64 { 30 }

// Versioning
#[derive(Serialize, Deserialize)]
#[serde(tag = "version")]
enum DataVersion {
    #[serde(rename = "1")]
    V1(DataV1),
    #[serde(rename = "2")]
    V2(DataV2),
}

// Remote derive (for external types)
#[derive(Serialize, Deserialize)]
#[serde(remote = "SystemTime")]
struct SystemTimeDef {
    #[serde(getter = "SystemTime::duration_since")]
    duration_since_epoch: Duration,
}

// Serialize map/dict
use serde::ser::SerializeMap;
fn serialize_map<S>(map: &MyMap, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut map_ser = serializer.serialize_map(Some(map.len()))?;
    for (k, v) in map.iter() {
        map_ser.serialize_entry(k, v)?;
    }
    map_ser.end()
}

// Serialize sequence/array
use serde::ser::SerializeSeq;
fn serialize_seq<S>(seq: &MySeq, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut seq_ser = serializer.serialize_seq(Some(seq.len()))?;
    for item in seq.iter() {
        seq_ser.serialize_element(item)?;
    }
    seq_ser.end()
}

// Transform during serialization
#[derive(Serialize)]
struct Output {
    #[serde(serialize_with = "to_uppercase")]
    text: String,
}

fn to_uppercase<S>(text: &str, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&text.to_uppercase())
}

// Validate during deserialization
#[derive(Deserialize)]
struct Validated {
    #[serde(deserialize_with = "validate_email")]
    email: String,
}

fn validate_email<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    if s.contains('@') {
        Ok(s)
    } else {
        Err(serde::de::Error::custom("invalid email"))
    }
}

// Generic serialization
#[derive(Serialize, Deserialize)]
struct Wrapper<T> {
    #[serde(bound = "T: Serialize + Deserialize<'de>")]
    data: T,
}

// Serialize/deserialize with pretty JSON
let json = serde_json::to_string_pretty(&data)?;
println!("{}", json);

// Partial deserialization (ignore extra fields)
#[derive(Deserialize)]
struct Partial {
    name: String,
    // Other fields in JSON are ignored
}

// Collect into HashMap
let map: HashMap<String, Value> = serde_json::from_str(json)?;

// Merge JSON objects
let mut base: Value = serde_json::from_str(base_json)?;
let overlay: Value = serde_json::from_str(overlay_json)?;
json_patch::merge(&mut base, &overlay);
```