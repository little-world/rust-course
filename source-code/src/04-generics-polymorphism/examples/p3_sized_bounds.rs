//! Pattern 3: Trait Bounds and Constraints
//! Example: Sized and ?Sized Bounds
//!
//! Run with: cargo run --example p3_sized_bounds

// By default, T must be Sized (known size at compile time)
fn takes_sized<T>(value: T) {
    drop(value);
}

// ?Sized accepts unsized types like str, [u8], or dyn Trait
fn takes_unsized<T: ?Sized>(value: &T) -> usize {
    std::mem::size_of_val(value)
}

// Works with both sized and unsized string types
fn print_str<T: AsRef<str> + ?Sized>(s: &T) {
    println!("{}", s.as_ref());
}

// Accepting any slice type
fn sum_slice<T: ?Sized>(slice: &T) -> i32
where
    T: AsRef<[i32]>,
{
    slice.as_ref().iter().sum()
}

// Trait that can be implemented for unsized types
// (Traits are NOT implicitly Sized, so no ?Sized needed on trait definition)
trait Describe {
    fn describe(&self) -> String;
}

impl Describe for str {
    fn describe(&self) -> String {
        format!("str of length {}", self.len())
    }
}

impl Describe for [i32] {
    fn describe(&self) -> String {
        format!("i32 slice of length {}", self.len())
    }
}

// Generic impl for any Debug type (sized only)
impl<T: std::fmt::Debug> Describe for Vec<T> {
    fn describe(&self) -> String {
        format!("Vec of {} elements: {:?}", self.len(), self)
    }
}

// Function accepting trait object (unsized)
fn describe_thing(thing: &dyn Describe) -> String {
    thing.describe()
}

fn main() {
    println!("=== Sized Types ===");
    // Regular sized types work with takes_sized
    takes_sized(42);
    takes_sized(String::from("hello"));
    takes_sized(vec![1, 2, 3]);
    println!("takes_sized works with i32, String, Vec<i32>");

    println!("\n=== Unsized Types ===");
    // Usage: ?Sized accepts dynamically-sized types via reference.
    let s: &str = "hello";
    let size_str = takes_unsized(s);
    println!("size_of_val(\"hello\") = {} bytes", size_str);

    let arr: &[i32] = &[1, 2, 3, 4, 5];
    let size_arr = takes_unsized(arr);
    println!("size_of_val(&[1, 2, 3, 4, 5]) = {} bytes", size_arr);

    // Also works with sized types
    let sized_val = 42_i32;
    let size_i32 = takes_unsized(&sized_val);
    println!("size_of_val(&42i32) = {} bytes", size_i32);

    println!("\n=== print_str with Various String Types ===");
    print_str("string literal");
    print_str(&String::from("String"));
    print_str(&"boxed str".to_string());

    println!("\n=== sum_slice with Various Slice Types ===");
    let vec = vec![1, 2, 3, 4, 5];
    let arr = [10, 20, 30];
    let slice: &[i32] = &[100, 200];

    println!("sum of vec![1, 2, 3, 4, 5] = {}", sum_slice(&vec));
    println!("sum of [10, 20, 30] = {}", sum_slice(&arr));
    println!("sum of &[100, 200] = {}", sum_slice(slice));

    println!("\n=== Describe Trait ===");
    let s: &str = "hello world";
    println!("\"hello world\".describe() = {}", s.describe());

    let nums: &[i32] = &[1, 2, 3, 4, 5];
    println!("[1, 2, 3, 4, 5].describe() = {}", nums.describe());

    let v = vec![1, 2, 3];
    println!("vec![1, 2, 3].describe() = {}", v.describe());

    println!("\n=== Trait Objects (dyn Describe) ===");
    // Use boxed trait objects for the heterogeneous collection
    let things: Vec<Box<dyn Describe>> = vec![
        Box::new(vec![1, 2, 3]),
    ];
    for thing in &things {
        println!("  {}", thing.describe());
    }

    // For unsized types, call describe() directly (not through trait object)
    println!("\n  Calling describe() directly on unsized types:");
    let s: &str = "hello";
    let nums: &[i32] = &[1, 2, 3];
    println!("  {}", s.describe());
    println!("  {}", nums.describe());

    println!("\n=== Key Points ===");
    println!("- By default, generic T must be Sized");
    println!("- T: ?Sized relaxes this, allowing unsized types");
    println!("- Unsized types can only be used via reference (&T)");
    println!("- Common unsized types: str, [T], dyn Trait");
}
