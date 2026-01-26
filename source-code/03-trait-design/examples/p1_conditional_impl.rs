//! Pattern 1: Trait Inheritance and Bounds
//! Example: Conditional Implementation with Trait Bounds
//!
//! Run with: cargo run --example p1_conditional_impl

use std::fmt::Debug;

struct Wrapper<T>(T);

// Only implement Clone if T is Clone
impl<T: Clone> Clone for Wrapper<T> {
    fn clone(&self) -> Self {
        Wrapper(self.0.clone())
    }
}

// Only implement Debug if T is Debug
impl<T: Debug> Debug for Wrapper<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Wrapper({:?})", self.0)
    }
}

// Only implement Default if T is Default
impl<T: Default> Default for Wrapper<T> {
    fn default() -> Self {
        Wrapper(T::default())
    }
}

// A type that is Clone but not Copy
#[derive(Clone, Debug)]
struct MyData {
    value: String,
}

// A type that is neither Clone nor Debug
struct NonCloneable {
    _data: *const (),
}

fn main() {
    // Usage: Wrapper gains Clone and Debug only when inner type has them.
    let w = Wrapper("hello".to_string());
    let w_clone = w.clone(); // Works because String is Clone
    println!("{:?}", w); // Works because String is Debug
    println!("{:?}", w_clone);

    // With custom type
    let data = MyData {
        value: "test".to_string(),
    };
    let wrapped = Wrapper(data);
    let wrapped_clone = wrapped.clone(); // Works: MyData is Clone
    println!("{:?}", wrapped); // Works: MyData is Debug
    println!("{:?}", wrapped_clone);

    // Default works when inner type has Default
    let default_wrapper: Wrapper<i32> = Wrapper::default();
    println!("Default wrapper: {:?}", default_wrapper);

    // NonCloneable type - Wrapper<NonCloneable> won't have Clone or Debug
    let _nc = Wrapper(NonCloneable {
        _data: std::ptr::null(),
    });
    // _nc.clone(); // Won't compile: NonCloneable is not Clone
    // println!("{:?}", _nc); // Won't compile: NonCloneable is not Debug

    println!("\nThe compiler automatically determines which impls apply!");
}
