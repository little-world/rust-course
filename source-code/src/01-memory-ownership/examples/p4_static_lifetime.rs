// Pattern 4: 'static Lifetime
// 'static means "lives for entire program"

// String literals are 'static
fn static_literal() {
    let s: &'static str = "I live forever";
    println!("{}", s);
}

// Owned data can satisfy 'static (it's not borrowed)
fn needs_static<T: 'static>(_value: T) {
    // T contains no non-'static references
}

fn static_examples() {
    needs_static(String::from("owned"));  // OK: owned data
    needs_static(42i32);                   // OK: Copy type
    needs_static(vec![1, 2, 3]);          // OK: owned Vec

    let _local = String::from("local");
    // needs_static(&local);  // Error: &local is not 'static
}

// Common with threads - data must be 'static or moved
use std::thread;

fn thread_static() {
    let data = vec![1, 2, 3];

    // Move ownership into thread (data becomes 'static-like)
    thread::spawn(move || {
        println!("{:?}", data);
    }).join().unwrap();
}

fn main() {
    static_literal();
    static_examples();
    thread_static();
    println!("Static lifetime example completed");
}
