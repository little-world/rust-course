//! Pattern 2: Generic Structs and Enums
//! Example: Generic Wrapper with Transformation
//!
//! Run with: cargo run --example p2_wrapper

#[derive(Debug)]
struct Wrapper<T> {
    value: T,
}

impl<T> Wrapper<T> {
    fn new(value: T) -> Self {
        Wrapper { value }
    }

    fn into_inner(self) -> T {
        self.value
    }

    // map transforms inner type via closure
    fn map<U, F: FnOnce(T) -> U>(self, f: F) -> Wrapper<U> {
        Wrapper { value: f(self.value) }
    }

    fn as_ref(&self) -> Wrapper<&T> {
        Wrapper { value: &self.value }
    }
}

impl<T: Clone> Wrapper<T> {
    fn clone_inner(&self) -> T {
        self.value.clone()
    }
}

impl<T: Default> Wrapper<T> {
    fn default_wrapper() -> Self {
        Wrapper { value: T::default() }
    }
}

fn main() {
    println!("=== Basic Wrapper ===");
    let w = Wrapper::new(5);
    println!("Created: {:?}", w);

    println!("\n=== Map Transformation ===");
    // Usage: map transforms inner value; type changes if closure returns different type.
    let w = Wrapper::new(5);
    let doubled = w.map(|x| x * 2);
    println!("Wrapper(5).map(|x| x * 2) = {:?}", doubled);

    let w = Wrapper::new(42);
    let stringified = w.map(|x| x.to_string());
    println!("Wrapper(42).map(|x| x.to_string()) = {:?}", stringified);

    let w = Wrapper::new("hello");
    let uppercased = w.map(|s| s.to_uppercase());
    println!("Wrapper(\"hello\").map(|s| s.to_uppercase()) = {:?}", uppercased);

    println!("\n=== Chained Transformations ===");
    let result = Wrapper::new(10)
        .map(|x| x + 5)      // 15
        .map(|x| x * 2)      // 30
        .map(|x| format!("Result: {}", x));
    println!("Wrapper(10).map(+5).map(*2).map(format) = {:?}", result);

    println!("\n=== as_ref ===");
    let w = Wrapper::new(vec![1, 2, 3]);
    let ref_wrapper = w.as_ref();
    println!("Wrapper(vec![1, 2, 3]).as_ref() = Wrapper {{ value: {:?} }}", ref_wrapper.value);

    println!("\n=== into_inner ===");
    let w = Wrapper::new("extracted");
    let inner = w.into_inner();
    println!("Wrapper(\"extracted\").into_inner() = \"{}\"", inner);

    println!("\n=== Conditional Methods ===");
    let w: Wrapper<String> = Wrapper::default_wrapper();
    println!("Wrapper::<String>::default_wrapper() = {:?}", w);

    let w = Wrapper::new(vec![1, 2, 3]);
    let cloned = w.clone_inner();
    println!("Wrapper(vec![1, 2, 3]).clone_inner() = {:?}", cloned);
}
