# Chapter 22: Serialization Patterns - Programming Projects

## Project 1: Custom INI Serialization Engine

### Problem Statement

Build a complete serialization and deserialization engine for the INI file format. Your engine will use Serde's core traits (`Serialize`, `Deserialize`, `Serializer`, `Deserializer`) to convert Rust structs to and from INI-formatted strings. The final product should be able to handle sections, key-value pairs, and basic data types like strings, numbers, and booleans.

### Use Cases

- **Application Configuration**: Many applications (especially in the Windows ecosystem) use `.ini` files for human-readable configuration.
- **Game Development**: Simple game settings (graphics, controls) are often stored in INI format.
- **Embedded Systems**: Lightweight configuration for devices where a full JSON or TOML parser is too heavy.
- **Legacy System Integration**: Interfacing with older systems that use INI as their data exchange format.

### Why It Matters

**Understanding Serde's Core**: Implementing `Serializer` and `Deserializer` demystifies how Serde works. You'll learn how data structures are broken down into primitives and reassembled, giving you the power to support any data format.

**Performance & Control**: While libraries exist, writing your own deserializer can be highly optimized for specific use cases. A parser that only handles the expected format can be faster and smaller than a general-purpose one.

**Extending the Ecosystem**: The skills learned here allow you to contribute new format support to the Rust ecosystem. If you need to interface with a proprietary or obscure format, you'll know exactly how to do it.

A simple INI file looks like this:
```ini
[database]
host = localhost
port = 5432
user = admin

[server]
enabled = true
```

---

## Milestone 1: Data Model for INI

### Introduction

Before we can serialize or deserialize, we need a Rust representation of an INI file's structure. This milestone focuses on creating the data structures that will hold the parsed INI data in memory.

**Why Start Here**: A solid data model is the foundation of any parser or serializer. It defines the boundaries and capabilities of your engine. We will represent the INI file as a map of section names to another map of key-value pairs.

### Architecture

**Structs:**
- `Ini`: The top-level container for an INI file.
  - **Field** `sections: HashMap<String, HashMap<String, String>>` - A map where keys are section names (e.g., "database") and values are another map containing the key-value pairs for that section.

**Key Functions:**
- `impl Ini::new() -> Self` - Creates an empty `Ini` object.
- `impl Ini::get(&self, section: &str, key: &str) -> Option<&String>` - Retrieves a value.
- `impl Ini::set(&mut self, section: String, key: String, value: String)` - Inserts or updates a value.

**Role Each Plays:**
- **`Ini`**: Represents the entire INI file in a structured way, making it easy to query and manipulate.
- **`HashMap<String, ...>`**: An efficient choice for looking up sections and keys by name.

### Checkpoint Tests

```rust
#[test]
fn test_ini_data_model() {
    let mut ini = Ini::new();
    ini.set("database".to_string(), "host".to_string(), "localhost".to_string());
    ini.set("database".to_string(), "port".to_string(), "5432".to_string());
    ini.set("server".to_string(), "enabled".to_string(), "true".to_string());

    assert_eq!(ini.get("database", "host"), Some(&"localhost".to_string()));
    assert_eq!(ini.get("database", "port"), Some(&"5432".to_string()));
    assert_eq!(ini.get("server", "enabled"), Some(&"true".to_string()));
    assert_eq!(ini.get("database", "nonexistent"), None);
}
```

### Starter Code

```rust
use std::collections::HashMap;

#[derive(Debug, PartialEq)]
pub struct Ini {
    pub sections: HashMap<String, HashMap<String, String>>,
}

impl Ini {
    pub fn new() -> Self {
        // TODO: Initialize an empty Ini struct.
        todo!("Implement Ini::new");
    }

    pub fn get(&self, section: &str, key: &str) -> Option<&String> {
        // TODO: Get a value from the specified section and key.
        // Hint: Use HashMap's .get() method twice.
        todo!("Implement Ini::get");
    }

    pub fn set(&mut self, section: String, key: String, value: String) {
        // TODO: Insert a value into the specified section and key.
        // Hint: Use HashMap's .entry().or_default() pattern.
        todo!("Implement Ini::set");
    }
}
```

---

## Milestone 2: A Manual INI Deserializer

### Introduction

**Why Milestone 1 Isn't Enough**: We have a data model, but we can't populate it from a string yet. This milestone is about writing a simple, manual parser that reads an INI-formatted string and populates our `Ini` struct.

**The Improvement**: This step bridges the gap between raw text and our structured data model. It forces us to handle the INI format's syntax rules, like section headers (`[section]`) and key-value pairs (`key = value`).

### Architecture

**Key Functions:**
- `fn from_str(s: &str) -> Result<Ini, String>` - The core parsing function.

**Role Each Plays:**
- **`from_str`**: Iterates over the lines of the input string, maintaining the current section context. It parses each line and populates an `Ini` object. This function will be the foundation for our `serde::Deserializer` later.

### Checkpoint Tests

```rust
#[test]
fn test_manual_parser() {
    let ini_str = "
[database]
host = localhost
port = 5432

[server]
enabled = true
";
    let ini = from_str(ini_str).expect("Failed to parse");

    assert_eq!(ini.get("database", "host"), Some(&"localhost".to_string()));
    assert_eq!(ini.get("database", "port"), Some(&"5432".to_string()));
    assert_eq!(ini.get("server", "enabled"), Some(&"true".to_string()));
}

#[test]
fn test_parser_invalid_format() {
    let invalid_ini = "just some text";
    assert!(from_str(invalid_ini).is_err());

    let no_section = "key = value";
    assert!(from_str(no_section).is_err(), "Keys must be under a section");
}
```

### Starter Code

```rust
// Assume Ini struct from Milestone 1 is available

pub fn from_str(s: &str) -> Result<Ini, String> {
    let mut ini = Ini::new();
    let mut current_section = None;

    for line in s.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with(';') {
            // Ignore empty lines and comments
            continue;
        }

        // TODO: Handle section headers `[section_name]`
        // - If a line is a section header, update `current_section`.
        // - Remember to handle the closing `]` bracket.

        // TODO: Handle key-value pairs `key = value`
        // - If a line is a key-value pair, split it at the '='.
        // - Ensure a section has been declared before adding a key-value pair.
        // - Add the pair to the `current_section` in the `ini` object.

        // TODO: Return an error for malformed lines.
    }

    Ok(ini)
}
```

**Implementation Hints:**
1.  Use `line.starts_with('[')` and `line.ends_with(']')` to detect section headers.
2.  Use `line.split_once('=')` to parse key-value pairs.
3.  Keep track of the current section name in a variable. If a key-value pair is found before any section header, it's an error.

---

## Milestone 3: Implementing `serde::Deserializer`

### Introduction

**Why Milestone 2 Isn't Enough**: Our manual parser only produces `Ini`. It can't deserialize into *any* Rust struct. By implementing `serde::Deserializer`, we can leverage the full power of Serde to deserialize our `Ini` representation into any struct that derives `Deserialize`.

**The Improvement**: We are making our parser generic. Instead of a one-off `Ini` parser, we're creating a `de::from_str` function that works just like `serde_json::from_str`.

### Architecture

**Structs:**
- `Deserializer<'de>`: Our custom deserializer struct. It will hold the `Ini` data we parsed in the previous step.

**Key Traits & Functions:**
- `impl<'de> de::Deserializer<'de> for Deserializer<'de>`: The main implementation block where we teach Serde how to interpret our `Ini` data model.
- `pub fn from_str<'a, T>(s: &'a str) -> Result<T, Error>` where `T: Deserialize<'a>`: The public-facing function users will call.

**Role Each Plays:**
- **`Deserializer`**: The state machine that Serde calls into. Serde will say "I expect a struct", and our `Deserializer` will provide it by traversing our `Ini` data.
- **`de::from_str`**: The entry point. It first does a manual parse into our intermediate `Ini` representation, then passes that to our `Deserializer` to drive the `T::deserialize` process.

### Checkpoint Tests

```rust
use serde::Deserialize;

#[derive(Deserialize, Debug, PartialEq)]
struct Config {
    server: ServerConfig,
    database: DbConfig,
}

#[derive(Deserialize, Debug, PartialEq)]
struct ServerConfig {
    enabled: bool,
}

#[derive(Deserialize, Debug, PartialEq)]
struct DbConfig {
    host: String,
    port: u16,
}

#[test]
fn test_serde_deserialization() {
    let ini_str = "
[server]
enabled = true

[database]
host = localhost
port = 5432
";
    let config: Config = from_str(ini_str).unwrap();

    assert_eq!(config, Config {
        server: ServerConfig { enabled: true },
        database: DbConfig { host: "localhost".to_string(), port: 5432 },
    });
}
```

### Starter Code

```rust
use serde::de::{self, Deserializer as SerdeDeserializer, MapAccess, Visitor};
use serde::Deserialize;

// Error type for our deserializer
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("INI parsing error: {0}")]
    Message(String),
    // ... other error variants
}

// Our custom Deserializer
pub struct Deserializer<'de> {
    // We'll use our `Ini` data model as the input
    ini: Ini,
}

impl<'de> Deserializer<'de> {
    pub fn from_ini(ini: Ini) -> Self {
        Deserializer { ini }
    }
}

// Public API
pub fn from_str<'a, T>(s: &'a str) -> Result<T, Error>
where
    T: Deserialize<'a>,
{
    // First, parse the string into our intermediate `Ini` representation
    let ini = manual_from_str(s).map_err(Error::Message)?;
    // Then, use our custom deserializer
    let mut deserializer = Deserializer::from_ini(ini);
    T::deserialize(&mut deserializer)
}

// The core Serde implementation
impl<'de, 'a> SerdeDeserializer<'de> for &'a mut Deserializer<'de> {
    type Error = Error;

    // `deserialize_any` is not supported for this format
    fn deserialize_any<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(Error::Message("deserialize_any is not supported".to_string()))
    }

    // We are deserializing a struct (the top-level Config) from the INI file
    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        // TODO: The visitor expects a `MapAccess`. We need to create something
        // that wraps our `Ini` and implements `MapAccess`.
        // This is the most complex part!
        todo!("Implement deserialize_struct");
    }

    // Handle primitive types. These will be called when deserializing
    // the values within the sections (e.g., `enabled = true`).
    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, Self::Error> {
        // This is tricky because the Deserializer doesn't know which
        // key it's on. The `MapAccess` implementation will need to handle this.
        todo!("Forward to MapAccess");
    }

    // ... implement for i64, u64, f64, string, etc.
}

// You will also need a struct that implements `MapAccess` for both the
// top-level sections and the key-value pairs within them.
```

---

## Milestone 4: A Manual INI Serializer

### Introduction

**Why We Need This**: We can now read INI files, but we can't write them. This milestone focuses on the reverse process: taking our `Ini` data structure and converting it back into a formatted string.

**The Improvement**: This gives us a complete round-trip capability (read, modify, write). The logic developed here will form the basis of our `serde::Serializer` implementation.

### Architecture

**Key Functions:**
- `impl Ini::to_string(&self) -> String` - The core serialization function.

**Role Each Plays:**
- **`to_string`**: Iterates through the sections and key-value pairs in the `Ini` struct and builds a `String` that conforms to the INI format rules.

### Checkpoint Tests

```rust
#[test]
fn test_manual_serializer() {
    let mut ini = Ini::new();
    ini.set("database".to_string(), "host".to_string(), "localhost".to_string());
    ini.set("database".to_string(), "port".to_string(), "5432".to_string());
    ini.set("server".to_string(), "enabled".to_string(), "true".to_string());

    let expected = "\
[database]
host = localhost
port = 5432

[server]
enabled = true
";

    let actual = ini.to_string();
    
    // Note: HashMaps don't guarantee order, so we parse the output back
    // and check for equality of the data model.
    let reparsed_ini = manual_from_str(&actual).unwrap();
    assert_eq!(ini, reparsed_ini);
}
```

### Starter Code

```rust
// In impl Ini { ... }

pub fn to_string(&self) -> String {
    let mut result = String::new();

    // TODO: Iterate over the sections in `self.sections`.
    // The iteration order doesn't matter for the INI format.
    for (section_name, section_values) in &self.sections {
        // TODO: Append the section header to the result string, e.g., `[section_name]\n`.
        
        // TODO: Iterate over the key-value pairs in `section_values`.
        for (key, value) in section_values {
            // TODO: Append the key-value pair, e.g., `key = value\n`.
        }
        // TODO: Add a blank line after each section for readability.
    }

    result
}
```

**Implementation Hints:**
1.  Use a `StringBuilder` or `String::push_str` for efficient string construction.
2.  Remember to add newlines (`\n`) after each line and section.

---

## Milestone 5: Implementing `serde::Serializer`

### Introduction

**Why Milestone 4 Isn't Enough**: Our manual serializer only works with our intermediate `Ini` struct. To serialize *any* Rust struct that derives `Serialize`, we need to implement Serde's `Serializer` trait.

**The Improvement**: This turns our one-off serializer into a generic `ser::to_string` function, capable of serializing a wide variety of Rust types into the INI format.

### Architecture

**Structs:**
- `Serializer`: Our custom serializer. It will write the formatted output to a `String`.
- Helper structs for `serialize_struct` and `serialize_map`.

**Key Traits & Functions:**
- `impl ser::Serializer for &mut Serializer`: The main implementation block where we define how to handle Rust types (bools, strings, structs, etc.).
- `pub fn to_string<T>(value: &T) -> Result<String, Error>` where `T: Serialize`: The public-facing function.

**Role Each Plays:**
- **`Serializer`**: The state machine that receives Rust data types from Serde and writes them out as INI-formatted text.
- **`ser::to_string`**: The entry point. It creates a `Serializer` and calls `value.serialize()` to start the process.

### Checkpoint Tests

```rust
use serde::Serialize;

#[derive(Serialize)]
struct Config {
    server: ServerConfig,
    database: DbConfig,
}

#[derive(Serialize)]
struct ServerConfig {
    enabled: bool,
}

#[derive(Serialize)]
struct DbConfig {
    host: String,
    port: u16,
}

#[test]
fn test_serde_serialization() {
    let config = Config {
        server: ServerConfig { enabled: true },
        database: DbConfig {
            host: "localhost".to_string(),
            port: 5432,
        },
    };

    let ini_string = to_string(&config).unwrap();

    // Again, parse back to test correctness due to order.
    let deserialized_config: crate::milestone3::Config = crate::milestone3::from_str(&ini_string).unwrap();

    assert_eq!(deserialized_config.server.enabled, true);
    assert_eq!(deserialized_config.database.host, "localhost");
    assert_eq!(deserialized_config.database.port, 5432);
}
```

### Starter Code
```rust
use serde::ser;

// Our custom Serializer
pub struct Serializer {
    output: String,
    current_section: Option<String>,
}

// Public API
pub fn to_string<T>(value: &T) -> Result<String, Error>
where
    T: ser::Serialize,
{
    let mut serializer = Serializer { output: String::new(), current_section: None };
    value.serialize(&mut serializer)?;
    Ok(serializer.output)
}

impl ser::Serializer for &mut Serializer {
    type Ok = ();
    type Error = Error;

    // Helper types for compound values.
    type SerializeStruct = Self;
    // ... other helpers

    // This will be called for primitive values like `true`, `5432`, `"localhost"`.
    fn serialize_str(self, v: &str) -> Result<(), Error> {
        // TODO: Append the string value to the current line.
        // This is tricky: we need to know the key first. The `SerializeStruct`
        // implementation will handle writing `key = ` before calling this.
        self.output.push_str(v);
        self.output.push('\n');
        Ok(())
    }
    
    // ... implement serialize_bool, serialize_u64, etc. They will convert the
    // value to a string and call `serialize_str`.

    // This is called for the top-level `Config` struct.
    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Error> {
        // The `Config` struct's fields are the section names.
        // We will return `self` and let `serialize_field` handle it.
        Ok(self)
    }

    // Other methods can be left as `unsupported`.
}

// This is where the magic happens for structs.
impl ser::SerializeStruct for &mut Serializer {
    type Ok = ();
    type Error = Error;

    // Called for each field of a struct.
    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Error>
    where
        T: ?Sized + ser::Serialize,
    {
        // If we're at the top level, the `key` is a section name.
        if self.current_section.is_none() {
            // TODO: Start a new section. Store `key` as the current section name.
            // Write the `[key]` header to the output.
            // Then, serialize the `value` (which is another struct).
        } else {
            // If we are already in a section, the `key` is a key in a key-value pair.
            // TODO: Write `key = ` to the output.
            // Then, serialize the `value` (which is a primitive).
        }
        Ok(())
    }

    // Called after all fields are serialized.
    fn end(self) -> Result<(), Error> {
        // If we just finished a section, add a newline and reset current_section.
        if self.current_section.is_some() {
            self.output.push('\n');
            self.current_section = None;
        }
        Ok(())
    }
}
```

---

## Milestone 6: Handling Comments and Whitespace

### Introduction

**Why Milestone 5 Isn't Enough**: Our current implementation is functional but rigid. Real-world INI files often contain comments (lines starting with `;` or `#`) and extra whitespace, which our parser currently handles but our serializer doesn't write.

**The Improvement**: This milestone is about polishing the engine to gracefully handle and even preserve comments and formatting, making it more robust and user-friendly.

### Architecture

**Changes to Data Model:**
- Modify the `Ini` struct to store comments and the order of sections/keys. Instead of `HashMap`, we could use `Vec<(String, String)>` for key-value pairs and `Vec<(String, Section)>` for sections to preserve order. Comments could be stored in a special field. *For this project, we will focus on a simpler goal: adding comments programmatically.*

**Serializer/Deserializer Enhancements:**
- **Deserializer**: Modify the manual parser to ignore lines starting with `;` or `#`.
- **Serializer**: Add a method `add_comment` to our `Ini` struct to insert comments before sections or keys. The `to_string` method would then write these out.

### Checkpoint Tests
```rust
#[test]
fn test_parser_with_comments() {
    let ini_str = "
; Database configuration
[database]
host = localhost ; The server hostname
port = 5432
";
    let ini = manual_from_str(ini_str).unwrap();
    assert_eq!(ini.get("database", "host"), Some(&"localhost".to_string()));
    assert!(ini.to_string().contains("[database]"));
}

#[test]
fn test_serializer_with_programmatic_comments() {
    let mut ini = Ini::new();
    ini.set_with_comment(
        "database".to_string(), 
        "host".to_string(), 
        "localhost".to_string(),
        Some("The server hostname".to_string())
    );
    let output = ini.to_string();
    assert!(output.contains("; The server hostname"));
}
```

### Starter Code
```rust
// Modify the `Ini` struct to support comments.
// A simple way is to change the value type in the map.
// pub sections: HashMap<String, HashMap<String, (String, Option<String>)>>

// In the manual parser:
for line in s.lines() {
    let line = line.trim();
    // Your existing logic...
    
    // When parsing a key-value pair, check for a comment.
    let (pair, comment) = line.split_once(';').unwrap_or((line, ""));
    // ... then parse the pair
}

// In the manual serializer:
for (key, (value, comment)) in section_values {
    let mut line = format!("{} = {}", key, value);
    if let Some(c) = comment {
        line.push_str(" ; ");
        line.push_str(c);
    }
    line.push('\n');
    result.push_str(&line);
}
```
---
## Complete Working Example

Here is a complete, simplified implementation covering the core concepts of the milestones. Note that a production-ready library would have more extensive error handling and would be more feature-rich. This example focuses on demonstrating the `Serializer` and `Deserializer` implementations.

```rust
// Add to Cargo.toml:
// serde = { version = "1.0", features = ["derive"] }
// thiserror = "1.0"

use serde::{de, ser};
use std::collections::HashMap;

// --- Data Model (Milestone 1) ---
#[derive(Debug, PartialEq, Clone)]
pub struct Ini {
    pub sections: HashMap<String, HashMap<String, String>>,
}

impl Ini {
    pub fn new() -> Self {
        Ini {
            sections: HashMap::new(),
        }
    }
    pub fn get(&self, section: &str, key: &str) -> Option<&String> {
        self.sections.get(section).and_then(|s| s.get(key))
    }
    pub fn set(&mut self, section: String, key: String, value: String) {
        self.sections
            .entry(section)
            .or_default()
            .insert(key, value);
    }
}

// --- Error Type ---
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("INI parsing error: {0}")]
    Message(String),
    #[error("Unsupported type")]
    Unsupported,
}

impl de::Error for Error {
    fn custom<T: std::fmt::Display>(msg: T) -> Self {
        Error::Message(msg.to_string())
    }
}
impl ser::Error for Error {
    fn custom<T: std::fmt::Display>(msg: T) -> Self {
        Error::Message(msg.to_string())
    }
}

// --- Manual Deserializer (Milestone 2) ---
pub fn manual_from_str(s: &str) -> Result<Ini, String> {
    let mut ini = Ini::new();
    let mut current_section_name = None;

    for line in s.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with(';') {
            continue;
        }

        if line.starts_with('[') && line.ends_with(']') {
            current_section_name = Some(line[1..line.len() - 1].to_string());
        } else if let Some(name) = &current_section_name {
            let parts: Vec<_> = line.splitn(2, '=').collect();
            if parts.len() == 2 {
                ini.set(name.clone(), parts[0].trim().to_string(), parts[1].trim().to_string());
            } else {
                return Err(format!("Malformed line: {}", line));
            }
        } else {
            return Err("Line found outside of a section".to_string());
        }
    }
    Ok(ini)
}


// --- Serde Deserializer (Milestone 3) ---

pub fn from_str<'a, T>(s: &'a str) -> Result<T, Error>
where
    T: de::Deserialize<'a>,
{
    let ini = manual_from_str(s).map_err(Error::Message)?;
    let mut deserializer = Deserializer { ini: ini.clone() };
    T::deserialize(&mut deserializer)
}

pub struct Deserializer {
    ini: Ini,
}

impl<'de, 'a> de::Deserializer<'de> for &'a mut Deserializer {
    type Error = Error;

    fn deserialize_any<V>(self, _visitor: V) -> Result<V::Value, Self::Error> where V: de::Visitor<'de> {
        Err(Error::Unsupported)
    }

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error> where V: de::Visitor<'de> {
        visitor.visit_map(SectionMapAccess {
            iter: self.ini.sections.iter(),
            current_section_map: None,
        })
    }

    // Other deserialize_* methods are not needed at the top level
    serde::forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map enum identifier ignored_any
    }
}

// Helper to deserialize the map of sections
struct SectionMapAccess<'a> {
    iter: std::collections::hash_map::Iter<'a, String, HashMap<String, String>>,
    current_section_map: Option<&'a HashMap<String, String>>,
}

impl<'de, 'a> de::MapAccess<'de> for SectionMapAccess<'a> {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where K: de::DeserializeSeed<'de> {
        if let Some((key, value)) = self.iter.next() {
            self.current_section_map = Some(value);
            let key_de = de::IntoDeserializer::into_deserializer(key.as_str());
            seed.deserialize(key_de).map(Some)
        } else {
            Ok(None)
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where V: de::DeserializeSeed<'de> {
        let section_map = self.current_section_map.take().unwrap();
        let mut val_deserializer = ValueDeserializer { map: section_map.clone() };
        seed.deserialize(&mut val_deserializer)
    }
}

// We need another deserializer for the inner struct (the key-value pairs)
struct ValueDeserializer {
    map: HashMap<String, String>,
}

impl<'de, 'a> de::Deserializer<'de> for &'a mut ValueDeserializer {
    type Error = Error;

    fn deserialize_any<V>(self, _visitor: V) -> Result<V::Value, Self::Error> where V: de::Visitor<'de> {
        Err(Error::Unsupported)
    }

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error> where V: de::Visitor<'de> {
        visitor.visit_map(KeyValueMapAccess {
            iter: self.map.iter(),
            current_value: None,
        })
    }
    
    serde::forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map enum identifier ignored_any
    }
}

struct KeyValueMapAccess<'a> {
    iter: std::collections::hash_map::Iter<'a, String, String>,
    current_value: Option<&'a String>,
}

impl<'de, 'a> de::MapAccess<'de> for KeyValueMapAccess<'a> {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error> where K: de::DeserializeSeed<'de> {
        if let Some((key, value)) = self.iter.next() {
            self.current_value = Some(value);
            let key_de = de::IntoDeserializer::into_deserializer(key.as_str());
            seed.deserialize(key_de).map(Some)
        } else {
            Ok(None)
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error> where V: de::DeserializeSeed<'de> {
        let value = self.current_value.take().unwrap();
        let val_de = de::IntoDeserializer::into_deserializer(value.as_str());
        seed.deserialize(val_de)
    }
}

// --- MAIN function to tie it all together ---
fn main() {
    // --- Define a struct to deserialize into ---
    #[derive(serde::Deserialize, Debug, PartialEq)]
    struct Config {
        server: ServerConfig,
        database: DbConfig,
    }
    #[derive(serde::Deserialize, Debug, PartialEq)]
    struct ServerConfig {
        enabled: bool,
    }
    #[derive(serde::Deserialize, Debug, PartialEq)]
    struct DbConfig {
        host: String,
        port: u16,
    }

    let ini_data = "
; App Config
[server]
enabled = true

[database]
host = db.example.com
port = 5432
";

    let config: Config = from_str(ini_data).unwrap();
    println!("Deserialized Config: {:#?}", config);

    assert_eq!(config, Config {
        server: ServerConfig { enabled: true },
        database: DbConfig { host: "db.example.com".to_string(), port: 5432 }
    });
    
    println!("\nSuccessfully deserialized INI string into Rust struct!");
}
```

This project provides a deep dive into the mechanics of Serde. While complex, mastering these concepts allows you to integrate Rust with virtually any data format imaginable.
