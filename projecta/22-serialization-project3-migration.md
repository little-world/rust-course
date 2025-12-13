# Data Migration with Schema Evolution

### Problem Statement

Build a robust data migration tool that can read different historical versions of a configuration format and transparently upgrade them to the latest version. You will define multiple versions of a Rust struct, using Serde attributes like `#[serde(default)]`, `#[serde(rename)]`, and `#[serde(alias)]` to handle schema changes gracefully. The final tool will be able to scan a directory of JSON files, identify their version, and rewrite them in the latest format.

### Use Cases

- **Software Upgrades**: When a new version of an application changes its configuration format, it must be able to read the user's old configuration file without losing data.
- **Long-Term Data Archiving**: Ensuring that data saved years ago can still be read and understood by current software.
- **API Development**: A server can accept multiple versions of a JSON payload from different clients and handle them all through a unified internal representation.
- **Distributed Systems**: Allowing different services to be updated independently, even if they share data structures that are evolving.

### Why It Matters

**Backward Compatibility is Key**: Forcing users to manually update their configuration files after a software update is a terrible user experience. A system that can handle old formats automatically is robust and user-friendly. Breaking changes should be a last resort.

**Prevents Data Loss**: Without proper schema evolution, deploying a code change can effectively "delete" old data that the new code can no longer read. The patterns in this project prevent that catastrophe.

**Real-World Software Maintenance**: Software is never static. Requirements change, fields are added, and names are improved for clarity. `serde`'s attributes provide a powerful, declarative way to manage this evolution directly in your data structures, making code easier to maintain and reason about. This is a far more common scenario than writing a new serialization format from scratch.

---

## Milestone 1: The Initial Schema (V1)

### Introduction

Every evolving system starts somewhere. This milestone defines the first version of our data structure, `UserConfigV1`. This simple, initial schema will be the foundation upon which all future changes are built.

**Why Start Here**: We need a baseline to evolve *from*. This V1 struct and its corresponding data represent the "old" format that our future code will need to support.

### Architecture

**Structs:**
- `UserConfigV1`: The original version of our user configuration.
  - **Fields**: `username: String`, `login_attempts: u32`.

**Key Functions:**
- A `main` function or test to demonstrate serializing a `UserConfigV1` instance to a JSON string.

**Role Each Plays:**
- **`UserConfigV1`**: Represents the data format as it existed at the beginning of the project.
- **`serde_json::to_string_pretty`**: Used to create a sample `user_v1.json` file that we will use as a test case in later milestones.

### Checkpoint Tests

```rust
#[test]
fn test_create_v1_data() {
    let config_v1 = UserConfigV1 {
        username: "alice".to_string(),
        login_attempts: 5,
    };
    
    let json_output = serde_json::to_string_pretty(&config_v1).unwrap();
    println!("V1 JSON:\n{}", json_output);

    assert!(json_output.contains("username"));
    assert!(json_output.contains("login_attempts"));
}
```

### Starter Code

```rust
use serde::{Serialize, Deserialize};

// Add to Cargo.toml:
// serde = { version = "1.0", features = ["derive"] }
// serde_json = "1.0"

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct UserConfigV1 {
    pub username: String,
    pub login_attempts: u32,
}

pub fn create_v1_example() -> String {
    // TODO: Create an instance of UserConfigV1.
    let config = todo!();

    // TODO: Serialize it to a pretty JSON string.
    let json_string: String = todo!();
    
    json_string
}
```

---

## Milestone 2: Adding a Field (V2)

### Introduction

**Why Milestone 1 Isn't Enough**: Our product has a new requirement: we need to store the user's display name. This means adding a new field to our config struct. Simply adding the field would break deserialization for all existing V1 files.

**The Improvement**: We introduce `UserConfigV2` with a new `display_name` field. By using the `#[serde(default)]` attribute, we tell Serde to use the `Default::default()` value for `display_name` if it's missing from the JSON file. This makes the new field backward compatible.

### Architecture

**Structs:**
- `UserConfigV2`: The second version of our configuration.
  - **Fields**: `username: String`, `login_attempts: u32`, `display_name: String`.

**Key Attributes:**
- `#[serde(default)]`: Applied to `display_name`. When deserializing, if this field is not present in the input, Serde will call `String::default()` to provide a value, thus preventing an error.

### Checkpoint Tests

```rust
#[test]
fn test_deserialize_v1_into_v2() {
    // This is the JSON from a V1 config, without "display_name"
    let json_v1 = r#"{
  "username": "alice",
  "login_attempts": 5
}"#;

    let config_v2: UserConfigV2 = serde_json::from_str(json_v1).unwrap();

    // Check that the new field was given its default value.
    assert_eq!(config_v2.username, "alice");
    assert_eq!(config_v2.login_attempts, 5);
    assert_eq!(config_v2.display_name, ""); // Default for String is empty
}
```

### Starter Code

```rust
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct UserConfigV2 {
    pub username: String,
    pub login_attempts: u32,
    
    // TODO: Add the `display_name` field.
    // TODO: Add the necessary Serde attribute to make it backward compatible.
    pub display_name: String,
}
```

**Implementation Hints:**
1.  The `Default` trait is required for the struct containing the field that uses `#[serde(default)]` if the default is generated by a function. Here, `String` already implements `Default`.
2.  The attribute is simply `#[serde(default)]`.

---

## Milestone 3: Renaming a Field (V3)

### Introduction

**Why Milestone 2 Isn't Enough**: The team has decided `login_attempts` is a confusing name. The new standard name is `failed_logins`. If we just rename the field in our struct, we'll break compatibility with both V1 and V2 files.

**The Improvement**: We create `UserConfigV3` and use `#[serde(alias = "...")]` to allow deserialization from the old field name (`login_attempts`) while using the new name (`failed_logins`) internally. The `#[serde(rename = "...")]` attribute is not strictly needed here but is good practice to show the canonical name.

### Architecture

**Structs:**
- `UserConfigV3`: The latest version.
  - **Fields**: Contains `failed_logins`.

**Key Attributes:**
- `#[serde(alias = "login_attempts")]`: This is the key. It tells Serde, "When you're deserializing a `UserConfigV3` and you see a field named `login_attempts`, please put its value into the field I've decorated."

### Checkpoint Tests

```rust
#[test]
fn test_deserialize_v2_into_v3() {
    // This JSON uses the V2 field name "login_attempts"
    let json_v2 = r#"{
  "username": "bob",
  "login_attempts": 3,
  "display_name": "Bob B."
}"#;

    let config_v3: UserConfigV3 = serde_json::from_str(json_v2).unwrap();

    assert_eq!(config_v3.username, "bob");
    // The value from "login_attempts" should be in "failed_logins"
    assert_eq!(config_v3.failed_logins, 3);
    assert_eq!(config_v3.display_name, "Bob B.");
}
```

### Starter Code

```rust
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct UserConfigV3 {
    pub username: String,
    
    // TODO: This is the new field name.
    // TODO: Add an attribute to allow deserializing from the old name "login_attempts".
    pub failed_logins: u32,
    
    #[serde(default)]
    pub display_name: String,
}
```

---

## Milestone 4: The Version-Dispatch Enum

### Introduction

**Why Milestone 3 Isn't Enough**: We have three different structs. How do we deserialize a file when we don't know which version it is? We could try deserializing into each one until it works, but there's a much cleaner, more idiomatic way.

**The Improvement**: We'll create an enum, `VersionedConfig`, and use `#[serde(untagged)]`. This tells Serde to try deserializing the data into each variant of the enum in order. The first one that succeeds without errors is chosen. This is a powerful pattern for handling different data shapes.

### Architecture

**Enums:**
- `VersionedConfig`: An enum with variants for `V1(UserConfigV1)`, `V2(UserConfigV2)`, and `V3(UserConfigV3)`.

**Key Attributes:**
- `#[serde(untagged)]`: This instructs Serde not to look for a special field identifying the variant, but to simply try each one in sequence.

### Checkpoint Tests

```rust
#[test]
fn test_untagged_deserialization() {
    let json_v1 = r#