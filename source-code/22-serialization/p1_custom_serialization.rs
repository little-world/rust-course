// Pattern 1: Custom Serialization Functions
use serde::de::{self, MapAccess, Visitor};
use serde::ser::SerializeStruct;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::HashMap;
use std::fmt;
use std::time::Duration;

// Custom serialization for Duration as seconds
fn serialize_duration<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_u64(duration.as_secs())
}

fn deserialize_duration<'de, D>(deserializer: D) -> Result<Duration, D::Error>
where
    D: Deserializer<'de>,
{
    let secs = u64::deserialize(deserializer)?;
    Ok(Duration::from_secs(secs))
}

#[derive(Serialize, Deserialize, Debug)]
struct Config {
    name: String,
    #[serde(
        serialize_with = "serialize_duration",
        deserialize_with = "deserialize_duration"
    )]
    timeout: Duration,
}

fn duration_serialization_demo() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Duration Serialization Demo ===\n");

    let config = Config {
        name: "MyApp".to_string(),
        timeout: Duration::from_secs(300),
    };

    let json = serde_json::to_string_pretty(&config)?;
    println!("Config JSON:\n{}", json);
    // timeout appears as 300, not a complex object

    let deserialized: Config = serde_json::from_str(&json)?;
    println!("\nDeserialized: {:?}", deserialized);
    println!("Timeout: {:?}", deserialized.timeout);

    Ok(())
}

// Custom date serialization
fn serialize_date<S>(date: &chrono::NaiveDate, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&date.format("%Y-%m-%d").to_string())
}

fn deserialize_date<'de, D>(deserializer: D) -> Result<chrono::NaiveDate, D::Error>
where
    D: Deserializer<'de>,
{
    struct DateVisitor;

    impl<'de> Visitor<'de> for DateVisitor {
        type Value = chrono::NaiveDate;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a date string in YYYY-MM-DD format")
        }

        fn visit_str<E>(self, value: &str) -> Result<chrono::NaiveDate, E>
        where
            E: de::Error,
        {
            chrono::NaiveDate::parse_from_str(value, "%Y-%m-%d")
                .map_err(|e| E::custom(format!("Invalid date: {}", e)))
        }
    }

    deserializer.deserialize_str(DateVisitor)
}

#[derive(Serialize, Deserialize, Debug)]
struct Event {
    name: String,
    #[serde(
        serialize_with = "serialize_date",
        deserialize_with = "deserialize_date"
    )]
    date: chrono::NaiveDate,
}

fn date_serialization_demo() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== Date Serialization Demo ===\n");

    let event = Event {
        name: "Conference".to_string(),
        date: chrono::NaiveDate::from_ymd_opt(2024, 6, 15).unwrap(),
    };

    let json = serde_json::to_string_pretty(&event)?;
    println!("Event JSON:\n{}", json);

    let deserialized: Event = serde_json::from_str(&json)?;
    println!("\nDeserialized: {:?}", deserialized);

    Ok(())
}

// Manual Serialize implementation
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
        enum Field {
            X,
            Y,
        }

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

fn manual_impl_demo() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== Manual Implementation Demo ===\n");

    let point = Point { x: 10.5, y: 20.3 };

    let json = serde_json::to_string_pretty(&point)?;
    println!("Point JSON:\n{}", json);

    let deserialized: Point = serde_json::from_str(&json)?;
    println!("\nDeserialized: {:?}", deserialized);

    // Test error cases
    println!("\nError handling:");

    let duplicate = r#"{"x": 1.0, "x": 2.0, "y": 3.0}"#;
    match serde_json::from_str::<Point>(duplicate) {
        Ok(p) => println!("  Duplicate field (JSON allows): {:?}", p),
        Err(e) => println!("  Duplicate field error: {}", e),
    }

    let missing = r#"{"x": 1.0}"#;
    match serde_json::from_str::<Point>(missing) {
        Ok(_) => println!("  Unexpected success"),
        Err(e) => println!("  Missing field error: {}", e),
    }

    Ok(())
}

// Serialize with computed data
struct Database {
    users: HashMap<u64, String>,
}

impl Serialize for Database {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("Database", 2)?;
        state.serialize_field("users", &self.users)?;
        // Serialize computed field
        state.serialize_field("user_count", &self.users.len())?;
        state.end()
    }
}

fn computed_fields_demo() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== Computed Fields Demo ===\n");

    let mut users = HashMap::new();
    users.insert(1, "Alice".to_string());
    users.insert(2, "Bob".to_string());
    users.insert(3, "Carol".to_string());

    let db = Database { users };

    let json = serde_json::to_string_pretty(&db)?;
    println!("Database JSON (with computed user_count):\n{}", json);

    Ok(())
}

// Context-aware serialization
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
            let mut state = serializer.serialize_struct("WithMetadata", 2)?;
            state.serialize_field("data", self.value)?;
            state.serialize_field("serialized_at", &chrono::Utc::now().to_rfc3339())?;
            state.end()
        } else {
            self.value.serialize(serializer)
        }
    }
}

fn context_serialization_demo() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== Context-Aware Serialization Demo ===\n");

    let data = vec!["item1", "item2", "item3"];

    let without_meta = SerializeWithContext {
        value: &data,
        include_metadata: false,
    };
    println!("Without metadata:");
    println!("{}\n", serde_json::to_string_pretty(&without_meta)?);

    let with_meta = SerializeWithContext {
        value: &data,
        include_metadata: true,
    };
    println!("With metadata:");
    println!("{}", serde_json::to_string_pretty(&with_meta)?);

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Custom Serialization Demo ===\n");

    duration_serialization_demo()?;
    date_serialization_demo()?;
    manual_impl_demo()?;
    computed_fields_demo()?;
    context_serialization_demo()?;

    println!("\nCustom serialization demo completed");
    Ok(())
}
