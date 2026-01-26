// Pattern 7: Returning Iterators with Type Inference

// Return position impl Trait
fn even_numbers(limit: i32) -> impl Iterator<Item = i32> {
    (0..limit).filter(|n| n % 2 == 0)
}

// With lifetime
fn filter_prefix<'a>(
    strings: &'a [String],
    prefix: &'a str
) -> impl Iterator<Item = &'a String> + 'a {
    strings.iter().filter(move |s| s.starts_with(prefix))
}

// Named iterator type for trait implementations
struct Evens {
    current: i32,
    limit: i32,
}

impl Iterator for Evens {
    type Item = i32;
    fn next(&mut self) -> Option<i32> {
        if self.current < self.limit {
            let val = self.current;
            self.current += 2;
            Some(val)
        } else {
            None
        }
    }
}

fn main() {
    // Using impl Trait return
    println!("Even numbers up to 10:");
    for n in even_numbers(10) {
        print!("{} ", n);
    }
    println!();

    // Using filter_prefix
    let strings: Vec<String> = vec![
        "hello".into(), "help".into(), "world".into(), "hero".into()
    ];
    println!("Strings starting with 'he':");
    for s in filter_prefix(&strings, "he") {
        print!("{} ", s);
    }
    println!();

    // Using named iterator
    let evens = Evens { current: 0, limit: 10 };
    println!("Evens from named iterator:");
    for n in evens {
        print!("{} ", n);
    }
    println!();

    println!("Returning iterators example completed");
}
