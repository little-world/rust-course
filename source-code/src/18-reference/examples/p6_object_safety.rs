// Pattern 6: Trait Object Receivers and Object Safety
trait ObjectSafe {
    // These are allowed in trait objects:
    fn by_ref(&self);
    fn by_mut(&mut self);
    fn by_box(self: Box<Self>);
}

// This trait is NOT object-safe (commented out to allow compilation)
// trait NotObjectSafe {
//     // NOT allowed in trait objects:
//     fn by_value(self);  // Requires knowing size at compile time
//     fn generic<T>(&self, t: T);  // Generic methods can't use vtable
//     fn returns_self(&self) -> Self;  // Size of Self unknown
// }

struct MyType;

impl ObjectSafe for MyType {
    fn by_ref(&self) {
        println!("by_ref called");
    }
    fn by_mut(&mut self) {
        println!("by_mut called");
    }
    fn by_box(self: Box<Self>) {
        println!("by_box called");
    }
}

// Usage:
// let obj: Box<dyn ObjectSafe> = Box::new(MyType);
// obj.by_ref();  // vtable dispatch
// obj.by_box();  // consumes the Box

fn main() {
    let mut obj: Box<dyn ObjectSafe> = Box::new(MyType);
    obj.by_ref();
    obj.by_mut();
    obj.by_box(); // consumes

    println!("Object safety example completed");
}
