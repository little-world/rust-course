//! Pattern 5: Sealed Traits
//! Example: Basic Sealed Trait
//!
//! Run with: cargo run --example p5_sealed_trait

mod sealed {
    pub trait Sealed {}
}

pub trait MyTrait: sealed::Sealed {
    fn my_method(&self);

    // Can add new methods without breaking external code
    fn new_method(&self) {
        println!("Default implementation");
    }
}

struct MyType {
    value: i32,
}

// Must implement Sealed first (only possible within this crate)
impl sealed::Sealed for MyType {}

impl MyTrait for MyType {
    fn my_method(&self) {
        println!("Value: {}", self.value);
    }
}

struct AnotherType;

impl sealed::Sealed for AnotherType {}
impl MyTrait for AnotherType {
    fn my_method(&self) {
        println!("AnotherType impl");
    }
}

// External crates can USE MyTrait but cannot IMPLEMENT it

fn use_trait<T: MyTrait>(item: &T) {
    item.my_method();
    item.new_method();
}

fn main() {
    // Usage: External crates can use MyTrait but cannot implement it.
    println!("=== MyType ===");
    let my = MyType { value: 42 };
    my.my_method(); // Prints "Value: 42"
    my.new_method(); // Default implementation works

    println!("\n=== AnotherType ===");
    let another = AnotherType;
    another.my_method();
    another.new_method();

    println!("\n=== Using generic function ===");
    use_trait(&my);
    use_trait(&another);
}
