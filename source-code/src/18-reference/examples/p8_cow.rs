// Pattern 8: ToOwned and Cow
use std::borrow::Cow;

// ToOwned: create owned from borrowed
// Generalization of Clone for borrowed types
fn to_owned_demo() {
    let s: &str = "hello";
    let owned: String = s.to_owned();  // &str -> String
    println!("Owned string: {}", owned);

    let slice: &[i32] = &[1, 2, 3];
    let owned: Vec<i32> = slice.to_owned();  // &[T] -> Vec<T>
    println!("Owned vec: {:?}", owned);
}

// Cow: clone on write, avoids allocation when possible
fn process_name(name: &str) -> Cow<'_, str> {
    if name.contains(' ') {
        // Must allocate to modify
        Cow::Owned(name.replace(' ', "_"))
    } else {
        // No allocation needed
        Cow::Borrowed(name)
    }
}

fn cow_in_structs() {
    #[derive(Debug)]
    struct Config<'a> {
        name: Cow<'a, str>,
        data: Cow<'a, [u8]>,
    }

    // Can own or borrow flexibly
    let config1 = Config {
        name: Cow::Borrowed("static"),
        data: Cow::Borrowed(&[1, 2, 3]),
    };

    let config2 = Config {
        name: Cow::Owned(String::from("dynamic")),
        data: Cow::Owned(vec![4, 5, 6]),
    };

    // Both have the same type: Config<'_>
    println!("Config1: {:?}", config1);
    println!("Config2: {:?}", config2);
}

fn main() {
    to_owned_demo();

    // Cow examples
    let name1 = process_name("John");
    let name2 = process_name("Jane Doe");

    println!("Name1 (no space): {} - is_borrowed: {}", name1, matches!(name1, Cow::Borrowed(_)));
    println!("Name2 (has space): {} - is_borrowed: {}", name2, matches!(name2, Cow::Borrowed(_)));

    cow_in_structs();

    println!("Cow example completed");
}
