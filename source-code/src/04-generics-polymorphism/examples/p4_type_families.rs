//! Pattern 4: Associated Types vs Generic Parameters
//! Example: Type Families with Associated Types
//!
//! Run with: cargo run --example p4_type_families

// Type family pattern: marker type maps to its associated member type
trait Family {
    type Member;
}

struct IntFamily;
impl Family for IntFamily {
    type Member = i32;
}

struct StringFamily;
impl Family for StringFamily {
    type Member = String;
}

struct VecFamily<T>(std::marker::PhantomData<T>);
impl<T> Family for VecFamily<T> {
    type Member = Vec<T>;
}

// Generic function over families
fn create_member<F: Family>() -> F::Member
where
    F::Member: Default,
{
    F::Member::default()
}

// More complex example: Collection family
trait CollectionFamily {
    type Collection<T>;
}

struct VecCollections;
impl CollectionFamily for VecCollections {
    type Collection<T> = Vec<T>;
}

struct OptionCollections;
impl CollectionFamily for OptionCollections {
    type Collection<T> = Option<T>;
}

// Using collection families
fn wrap_in_collection<F: CollectionFamily, T>(value: T) -> F::Collection<T>
where
    F::Collection<T>: From<T>,
{
    F::Collection::<T>::from(value)
}

// Database family example
trait DatabaseFamily {
    type Connection;
    type Query;
    type Result;
}

struct PostgresFamily;
impl DatabaseFamily for PostgresFamily {
    type Connection = PostgresConnection;
    type Query = PostgresQuery;
    type Result = PostgresResult;
}

struct SqliteFamily;
impl DatabaseFamily for SqliteFamily {
    type Connection = SqliteConnection;
    type Query = SqliteQuery;
    type Result = SqliteResult;
}

// Placeholder types
struct PostgresConnection;
struct PostgresQuery;
struct PostgresResult;
struct SqliteConnection;
struct SqliteQuery;
struct SqliteResult;

// Generic database operations
fn describe_db<F: DatabaseFamily>() -> String {
    format!(
        "Database family with Connection, Query, and Result types"
    )
}

fn main() {
    println!("=== Simple Type Families ===");
    // Usage: Type family maps marker type to its associated member type.
    let int_val: i32 = create_member::<IntFamily>();
    let str_val: String = create_member::<StringFamily>();
    let vec_val: Vec<i32> = create_member::<VecFamily<i32>>();

    println!("create_member::<IntFamily>() = {}", int_val);
    println!("create_member::<StringFamily>() = \"{}\"", str_val);
    println!("create_member::<VecFamily<i32>>() = {:?}", vec_val);

    println!("\n=== Collection Families ===");
    // Vec collection family
    let vec_wrapped: Vec<i32> = vec![42];  // Using From for simplicity
    println!("VecCollections wraps 42 into: {:?}", vec_wrapped);

    // Option collection family
    let opt_wrapped: Option<i32> = Some(42);
    println!("OptionCollections wraps 42 into: {:?}", opt_wrapped);

    println!("\n=== Database Family Pattern ===");
    println!("PostgresFamily: {}", describe_db::<PostgresFamily>());
    println!("SqliteFamily: {}", describe_db::<SqliteFamily>());

    println!("\n=== Why Type Families? ===");
    println!("Type families create type-level mappings:");
    println!("  IntFamily    -> i32");
    println!("  StringFamily -> String");
    println!("  VecFamily<T> -> Vec<T>");
    println!("\nUse cases:");
    println!("  - Database abstraction (Postgres vs SQLite families)");
    println!("  - Collection abstractions");
    println!("  - Protocol families (HTTP vs WebSocket)");
    println!("  - Serialization format families (JSON vs YAML)");
}
