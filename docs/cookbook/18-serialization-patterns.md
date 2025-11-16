# 18. Serialization Patterns

## Serde Patterns (Derive, Custom Serializers)

### Basic Derive Usage

```rust
// Add to Cargo.toml:
// [dependencies]
// serde = { version = "1.0", features = ["derive"] }
// serde_json = "1.0"

use serde::{Serialize, Deserialize};

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

    // Serialize to JSON
    let json = serde_json::to_string(&person)?;
    println!("JSON: {}", json);

    // Pretty print
    let json_pretty = serde_json::to_string_pretty(&person)?;
    println!("Pretty JSON:\n{}", json_pretty);

    // Deserialize from JSON
    let deserialized: Person = serde_json::from_str(&json)?;
    println!("Deserialized: {:?}", deserialized);

    Ok(())
}
```

### Field Attributes

```rust
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
struct User {
    // Rename field in serialized output
    #[serde(rename = "username")]
    name: String,

    // Skip serializing if None
    #[serde(skip_serializing_if = "Option::is_none")]
    middle_name: Option<String>,

    // Provide default value when deserializing
    #[serde(default)]
    age: u32,

    // Skip this field entirely
    #[serde(skip)]
    password_hash: String,

    // Use a different name for serialization vs deserialization
    #[serde(alias = "mail", alias = "e-mail")]
    email: String,

    // Flatten nested structure
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
        middle_name: None,
        age: 25,
        password_hash: "secret".to_string(),
        email: "bob@example.com".to_string(),
        metadata: Metadata {
            created_at: Some("2024-01-01".to_string()),
            updated_at: None,
        },
    };

    let json = serde_json::to_string_pretty(&user)?;
    println!("{}", json);

    Ok(())
}
```

### Container Attributes

```rust
use serde::{Serialize, Deserialize};

// Rename all fields to camelCase
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ApiResponse {
    status_code: u32,
    error_message: Option<String>,
    response_data: Vec<String>,
}

// Deny unknown fields during deserialization
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct StrictConfig {
    host: String,
    port: u16,
}

// Tag enum variants
#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
enum Message {
    Text { content: String },
    Image { url: String, width: u32, height: u32 },
    Video { url: String, duration: u32 },
}

// Untagged enum (use content to determine variant)
#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
enum Value {
    Integer(i64),
    Float(f64),
    String(String),
    Bool(bool),
}

fn enum_serialization() -> Result<(), Box<dyn std::error::Error>> {
    let message = Message::Image {
        url: "https://example.com/image.jpg".to_string(),
        width: 1920,
        height: 1080,
    };

    let json = serde_json::to_string_pretty(&message)?;
    println!("Tagged enum:\n{}", json);

    let value = Value::String("hello".to_string());
    let json = serde_json::to_string(&value)?;
    println!("Untagged enum: {}", json);

    Ok(())
}
```

### Custom Serialization Functions

```rust
use serde::{Serialize, Deserialize, Serializer, Deserializer};
use serde::de::{self, Visitor};
use std::fmt;

#[derive(Serialize, Deserialize, Debug)]
struct Config {
    // Serialize duration as seconds
    #[serde(serialize_with = "serialize_duration", deserialize_with = "deserialize_duration")]
    timeout: std::time::Duration,

    // Custom date format
    #[serde(serialize_with = "serialize_date", deserialize_with = "deserialize_date")]
    created_at: chrono::NaiveDate,
}

fn serialize_duration<S>(duration: &std::time::Duration, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_u64(duration.as_secs())
}

fn deserialize_duration<'de, D>(deserializer: D) -> Result<std::time::Duration, D::Error>
where
    D: Deserializer<'de>,
{
    let secs = u64::deserialize(deserializer)?;
    Ok(std::time::Duration::from_secs(secs))
}

// Using chrono for this example
use chrono::NaiveDate;

fn serialize_date<S>(date: &NaiveDate, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&date.format("%Y-%m-%d").to_string())
}

fn deserialize_date<'de, D>(deserializer: D) -> Result<NaiveDate, D::Error>
where
    D: Deserializer<'de>,
{
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
            NaiveDate::parse_from_str(value, "%Y-%m-%d")
                .map_err(|e| E::custom(format!("Invalid date: {}", e)))
        }
    }

    deserializer.deserialize_str(DateVisitor)
}
```

### Custom Serialize/Deserialize Implementation

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

impl Serialize for Point {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("Point", 2)?;
        state.serialize_field("x", &self.x)?;
        state.serialize_field("y", &self.y)?;
        state.end()
    }
}

impl<'de> Deserialize<'de> for Point {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        enum Field { X, Y }

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

### Serializing with State

```rust
use serde::{Serialize, Serializer};
use std::collections::HashMap;

struct Database {
    users: HashMap<u64, String>,
}

// Custom serialization that includes computed data
impl Serialize for Database {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use serde::ser::SerializeStruct;

        let mut state = serializer.serialize_struct("Database", 2)?;
        state.serialize_field("users", &self.users)?;
        state.serialize_field("user_count", &self.users.len())?;
        state.end()
    }
}

// Wrapper for custom serialization context
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
            use serde::ser::SerializeStruct;
            let mut state = serializer.serialize_struct("WithMetadata", 2)?;
            state.serialize_field("data", self.value)?;
            state.serialize_field("serialized_at", &chrono::Utc::now().to_rfc3339())?;
            state.end()
        } else {
            self.value.serialize(serializer)
        }
    }
}
```

## Zero-Copy Deserialization

### Borrowing from Input

```rust
use serde::{Deserialize, Serialize};

// Zero-copy: borrows from the input string
#[derive(Deserialize, Debug)]
struct BorrowedData<'a> {
    #[serde(borrow)]
    name: &'a str,

    #[serde(borrow)]
    description: &'a str,

    count: u32,
}

fn zero_copy_example() -> Result<(), Box<dyn std::error::Error>> {
    let json = r#"{"name": "Product", "description": "A great product", "count": 42}"#;

    // No string allocation - borrows from json
    let data: BorrowedData = serde_json::from_str(json)?;

    println!("Name: {}", data.name);
    println!("Description: {}", data.description);
    println!("Count: {}", data.count);

    Ok(())
}

// Cow for flexible ownership
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

    // Can be borrowed or owned depending on the data
    println!("Name: {}", data.name);
    println!("Tags: {:?}", data.tags);

    Ok(())
}
```

### Using Bytes and ByteBuf

```rust
use serde::{Deserialize, Serialize};
use serde_bytes::{ByteBuf, Bytes};

#[derive(Serialize, Deserialize, Debug)]
struct BinaryData<'a> {
    #[serde(with = "serde_bytes")]
    owned_data: Vec<u8>,

    #[serde(borrow, with = "serde_bytes")]
    borrowed_data: &'a [u8],
}

// More efficient for binary data
#[derive(Serialize, Deserialize, Debug)]
struct OptimizedBinaryData {
    // Uses compact binary representation
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

### Zero-Copy with bincode

```rust
// Add to Cargo.toml:
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

    // Serialize to bytes
    let encoded = bincode::serialize(&record)?;

    // Zero-copy deserialization
    let decoded: Record = bincode::deserialize(&encoded)?;

    println!("Decoded: {:?}", decoded);

    Ok(())
}
```

### Custom Zero-Copy Deserializer

```rust
use serde::de::{self, Deserializer, Visitor};
use std::fmt;

// Custom deserializer for borrowed slices
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

## Schema Evolution

### Adding Optional Fields

```rust
use serde::{Deserialize, Serialize};

// Version 1
#[derive(Serialize, Deserialize, Debug)]
struct ConfigV1 {
    host: String,
    port: u16,
}

// Version 2: Add optional field with default
#[derive(Serialize, Deserialize, Debug)]
struct ConfigV2 {
    host: String,
    port: u16,

    // New field - defaults to None if missing
    #[serde(default)]
    timeout: Option<u32>,
}

// Version 3: Required field with default
#[derive(Serialize, Deserialize, Debug)]
struct ConfigV3 {
    host: String,
    port: u16,

    #[serde(default)]
    timeout: Option<u32>,

    // Defaults to 10 if missing
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

    Ok(())
}
```

### Versioned Enums

```rust
use serde::{Deserialize, Serialize};

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
    fn to_latest(self) -> MessageV3 {
        match self {
            VersionedMessage::V1 { content } => MessageV3 {
                content,
                timestamp: 0,
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

### Handling Renamed Fields

```rust
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
struct UserProfile {
    // Accept both old and new names during deserialization
    #[serde(alias = "user_name", alias = "userName")]
    name: String,

    // Serialize as "emailAddress", accept "email" or "emailAddress"
    #[serde(rename = "emailAddress", alias = "email")]
    email_address: String,
}

fn renamed_fields_example() -> Result<(), Box<dyn std::error::Error>> {
    // Old format
    let old_json = r#"{"user_name": "Alice", "email": "alice@example.com"}"#;
    let profile: UserProfile = serde_json::from_str(old_json)?;

    // New format
    let new_json = serde_json::to_string_pretty(&profile)?;
    println!("New format:\n{}", new_json);

    Ok(())
}
```

### Custom Migration Logic

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

                // Migrate: build connection_string from host and port if missing
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

## Binary vs Text Formats

### JSON (Text Format)

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

    // Serialize
    let json = serde_json::to_string_pretty(&product)?;
    println!("JSON ({} bytes):\n{}", json.len(), json);

    // Deserialize
    let deserialized: Product = serde_json::from_str(&json)?;
    println!("Deserialized: {:?}", deserialized);

    Ok(())
}
```

### Bincode (Binary Format)

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

    // Serialize
    let encoded = bincode::serialize(&product)?;
    println!("Bincode ({} bytes): {:?}", encoded.len(), encoded);

    // Deserialize
    let decoded: Product = bincode::deserialize(&encoded)?;
    println!("Deserialized: {:?}", decoded);

    Ok(())
}
```

### MessagePack (Binary Format)

```rust
// Add to Cargo.toml:
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

    // Serialize
    let encoded = rmp_serde::to_vec(&product)?;
    println!("MessagePack ({} bytes): {:?}", encoded.len(), encoded);

    // Deserialize
    let decoded: Product = rmp_serde::from_slice(&encoded)?;
    println!("Deserialized: {:?}", decoded);

    Ok(())
}
```

### CBOR (Binary Format)

```rust
// Add to Cargo.toml:
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

    // Serialize
    let encoded = serde_cbor::to_vec(&product)?;
    println!("CBOR ({} bytes): {:?}", encoded.len(), encoded);

    // Deserialize
    let decoded: Product = serde_cbor::from_slice(&encoded)?;
    println!("Deserialized: {:?}", decoded);

    Ok(())
}
```

### YAML (Text Format)

```rust
// Add to Cargo.toml:
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

    // Serialize
    let yaml = serde_yaml::to_string(&product)?;
    println!("YAML ({} bytes):\n{}", yaml.len(), yaml);

    // Deserialize
    let deserialized: Product = serde_yaml::from_str(&yaml)?;
    println!("Deserialized: {:?}", deserialized);

    Ok(())
}
```

### TOML (Text Format)

```rust
// Add to Cargo.toml:
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

    // Serialize
    let toml = toml::to_string_pretty(&config)?;
    println!("TOML ({} bytes):\n{}", toml.len(), toml);

    // Deserialize
    let deserialized: Config = toml::from_str(&toml)?;
    println!("Deserialized: {:?}", deserialized);

    Ok(())
}
```

### Format Comparison

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

    // JSON
    let json = serde_json::to_string(&data)?;
    println!("JSON: {} bytes", json.len());

    // Bincode
    let bincode = bincode::serialize(&data)?;
    println!("Bincode: {} bytes", bincode.len());

    // MessagePack
    let msgpack = rmp_serde::to_vec(&data)?;
    println!("MessagePack: {} bytes", msgpack.len());

    // CBOR
    let cbor = serde_cbor::to_vec(&data)?;
    println!("CBOR: {} bytes", cbor.len());

    println!("\nBinary formats are typically 30-50% smaller than JSON");

    Ok(())
}
```

## Streaming Serialization

### Streaming JSON Arrays

```rust
use serde::Serialize;
use std::io::{self, Write};

#[derive(Serialize)]
struct Record {
    id: u64,
    name: String,
    value: f64,
}

fn stream_json_array<W: Write>(mut writer: W, records: &[Record]) -> io::Result<()> {
    writer.write_all(b"[")?;

    for (i, record) in records.iter().enumerate() {
        if i > 0 {
            writer.write_all(b",")?;
        }

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

### Streaming to File

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

### Streaming Deserialization

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

fn stream_from_file(path: &str) -> io::Result<()> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    for (line_num, line) in reader.lines().enumerate() {
        let line = line?;

        match serde_json::from_str::<LogEntry>(&line) {
            Ok(entry) => {
                // Process entry
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

### Streaming with serde_json::Deserializer

```rust
use serde::Deserialize;
use std::io::{self, Cursor};

#[derive(Deserialize, Debug)]
struct Item {
    id: u64,
    name: String,
}

fn streaming_deserializer() -> Result<(), Box<dyn std::error::Error>> {
    let json = r#"
        {"id": 1, "name": "Item 1"}
        {"id": 2, "name": "Item 2"}
        {"id": 3, "name": "Item 3"}
    "#;

    let cursor = Cursor::new(json);
    let deserializer = serde_json::Deserializer::from_reader(cursor);

    for item in deserializer.into_iter::<Item>() {
        match item {
            Ok(item) => println!("Deserialized: {:?}", item),
            Err(e) => eprintln!("Error: {}", e),
        }
    }

    Ok(())
}
```

### Async Streaming with Tokio

```rust
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::fs::File;

#[derive(Serialize, Deserialize, Debug)]
struct Record {
    id: u64,
    data: String,
}

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

### Large Dataset Streaming

```rust
use serde::Serialize;
use std::io::{self, Write};

#[derive(Serialize)]
struct DataPoint {
    x: f64,
    y: f64,
    timestamp: u64,
}

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

        // Flush every 100 records
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

### Custom Streaming Format

```rust
use serde::Serialize;
use std::io::{self, Write};

// Length-prefixed binary format for streaming
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

This comprehensive guide covers all essential serialization patterns in Rust, from basic serde usage to advanced streaming and zero-copy techniques.
