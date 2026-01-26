// Pattern 3: Schema Evolution
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// Version 1: Original schema
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
    #[serde(default)]
    timeout: Option<u32>,
}

// Version 3: Required field with default function
fn default_max_connections() -> u32 {
    10
}

#[derive(Serialize, Deserialize, Debug)]
struct ConfigV3 {
    host: String,
    port: u16,
    #[serde(default)]
    timeout: Option<u32>,
    #[serde(default = "default_max_connections")]
    max_connections: u32,
}

fn schema_evolution_demo() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Schema Evolution Demo ===\n");

    // Original V1 JSON
    let v1_json = r#"{"host": "localhost", "port": 8080}"#;
    println!("V1 JSON: {}", v1_json);

    // V1 JSON works with V1 struct
    let v1_config: ConfigV1 = serde_json::from_str(v1_json)?;
    println!("As ConfigV1: {:?}", v1_config);

    // V1 JSON works with V2 struct (timeout defaults to None)
    let v2_config: ConfigV2 = serde_json::from_str(v1_json)?;
    println!("As ConfigV2: {:?}", v2_config);

    // V1 JSON works with V3 struct (both optional fields get defaults)
    let v3_config: ConfigV3 = serde_json::from_str(v1_json)?;
    println!("As ConfigV3: {:?}", v3_config);
    println!("  timeout (defaulted): {:?}", v3_config.timeout);
    println!("  max_connections (defaulted): {}", v3_config.max_connections);

    // V3 JSON with all fields
    println!("\n--- Full V3 JSON ---");
    let v3_json = r#"{"host": "0.0.0.0", "port": 3000, "timeout": 60, "max_connections": 100}"#;
    let full_config: ConfigV3 = serde_json::from_str(v3_json)?;
    println!("Full ConfigV3: {:?}", full_config);

    Ok(())
}

// Tag-based versioning
#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "version")]
enum VersionedMessage {
    #[serde(rename = "1")]
    V1 { content: String },

    #[serde(rename = "2")]
    V2 { content: String, timestamp: u64 },

    #[serde(rename = "3")]
    V3 {
        content: String,
        timestamp: u64,
        metadata: HashMap<String, String>,
    },
}

#[derive(Debug)]
struct MessageLatest {
    content: String,
    timestamp: u64,
    metadata: HashMap<String, String>,
}

impl VersionedMessage {
    fn to_latest(self) -> MessageLatest {
        match self {
            VersionedMessage::V1 { content } => MessageLatest {
                content,
                timestamp: 0,
                metadata: HashMap::new(),
            },
            VersionedMessage::V2 { content, timestamp } => MessageLatest {
                content,
                timestamp,
                metadata: HashMap::new(),
            },
            VersionedMessage::V3 {
                content,
                timestamp,
                metadata,
            } => MessageLatest {
                content,
                timestamp,
                metadata,
            },
        }
    }
}

fn versioned_message_demo() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== Versioned Message Demo ===\n");

    let messages = vec![
        r#"{"version": "1", "content": "Hello from V1"}"#,
        r#"{"version": "2", "content": "Hello from V2", "timestamp": 1234567890}"#,
        r#"{"version": "3", "content": "Hello from V3", "timestamp": 1234567890, "metadata": {"source": "api"}}"#,
    ];

    for json in messages {
        println!("JSON: {}", json);
        let msg: VersionedMessage = serde_json::from_str(json)?;
        println!("Parsed: {:?}", msg);
        let latest = msg.to_latest();
        println!("As latest: {:?}\n", latest);
    }

    Ok(())
}

// Handling renamed fields with aliases
#[derive(Serialize, Deserialize, Debug)]
struct UserProfile {
    // Accept both old and new names during deserialization
    #[serde(alias = "user_name", alias = "userName")]
    name: String,

    // Serialize as new name, accept old name when deserializing
    #[serde(rename = "emailAddress", alias = "email")]
    email_address: String,
}

fn renamed_fields_demo() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== Renamed Fields Demo ===\n");

    // Old format
    let old_json = r#"{"user_name": "Alice", "email": "alice@example.com"}"#;
    println!("Old JSON: {}", old_json);
    let profile: UserProfile = serde_json::from_str(old_json)?;
    println!("Parsed: {:?}", profile);

    // Serialize to new format
    let new_json = serde_json::to_string_pretty(&profile)?;
    println!("\nSerialized (new format):\n{}", new_json);

    // New format can still be parsed
    let reparsed: UserProfile = serde_json::from_str(&new_json)?;
    println!("\nReparsed: {:?}", reparsed);

    Ok(())
}

// Adding fields without breaking existing code
#[derive(Serialize, Deserialize, Debug)]
struct ApiResponseV1 {
    data: String,
    success: bool,
}

#[derive(Serialize, Deserialize, Debug)]
struct ApiResponseV2 {
    data: String,
    success: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    error_code: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    trace_id: Option<String>,
}

fn forward_backward_compatible_demo() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== Forward/Backward Compatibility Demo ===\n");

    // Old client sends V1 response
    let v1_response = ApiResponseV1 {
        data: "result".to_string(),
        success: true,
    };
    let v1_json = serde_json::to_string(&v1_response)?;
    println!("V1 JSON: {}", v1_json);

    // New server can parse V1 as V2
    let as_v2: ApiResponseV2 = serde_json::from_str(&v1_json)?;
    println!("Parsed as V2: {:?}", as_v2);
    println!("  error_code: {:?} (defaulted)", as_v2.error_code);

    // New server sends V2 response
    let v2_response = ApiResponseV2 {
        data: "result".to_string(),
        success: false,
        error_code: Some(404),
        trace_id: Some("abc-123".to_string()),
    };
    let v2_json = serde_json::to_string(&v2_response)?;
    println!("\nV2 JSON: {}", v2_json);

    // Old client can still parse it (ignores unknown fields by default)
    let as_v1: ApiResponseV1 = serde_json::from_str(&v2_json)?;
    println!("Parsed as V1: {:?}", as_v1);

    Ok(())
}

// Deprecating fields
#[derive(Serialize, Deserialize, Debug)]
struct LegacyConfig {
    // New field
    database_url: Option<String>,

    // Old fields - kept for backward compatibility
    #[serde(default, skip_serializing)]
    db_host: Option<String>,
    #[serde(default, skip_serializing)]
    db_port: Option<u16>,
    #[serde(default, skip_serializing)]
    db_name: Option<String>,
}

impl LegacyConfig {
    fn get_database_url(&self) -> Option<String> {
        // Prefer new field, fall back to constructing from old fields
        self.database_url.clone().or_else(|| {
            match (&self.db_host, &self.db_port, &self.db_name) {
                (Some(host), Some(port), Some(name)) => {
                    Some(format!("postgres://{}:{}/{}", host, port, name))
                }
                _ => None,
            }
        })
    }
}

fn deprecation_demo() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== Deprecation Demo ===\n");

    // Old config format
    let old_json = r#"{"db_host": "localhost", "db_port": 5432, "db_name": "myapp"}"#;
    println!("Old config: {}", old_json);
    let old_config: LegacyConfig = serde_json::from_str(old_json)?;
    println!("Parsed: {:?}", old_config);
    println!("Effective URL: {:?}", old_config.get_database_url());

    // New config format
    let new_json = r#"{"database_url": "postgres://localhost:5432/myapp"}"#;
    println!("\nNew config: {}", new_json);
    let new_config: LegacyConfig = serde_json::from_str(new_json)?;
    println!("Parsed: {:?}", new_config);
    println!("Effective URL: {:?}", new_config.get_database_url());

    // When serialized, only new format is output
    println!("\nSerialized (only new format):");
    println!("{}", serde_json::to_string_pretty(&new_config)?);

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Schema Evolution Demo ===\n");

    schema_evolution_demo()?;
    versioned_message_demo()?;
    renamed_fields_demo()?;
    forward_backward_compatible_demo()?;
    deprecation_demo()?;

    println!("\nSchema evolution demo completed");
    Ok(())
}
