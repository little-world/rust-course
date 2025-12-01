
### Serde Cheat Sheet

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