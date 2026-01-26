//! Pattern 2: Exhaustiveness and Match Ergonomics
//! Example: Match Ergonomics (Automatic Reference Handling)
//!
//! Run with: cargo run --example p2_match_ergonomics

fn process_option(opt: &Option<String>) {
    match opt {
        // With match ergonomics, `s` is automatically `&String`, not `String`.
        // No need to write `&Some(ref s)` or `Some(s)` explicitly.
        Some(s) => println!("Got a string: {}", s),
        None => println!("Got nothing."),
    }
}

fn process_result(res: &Result<i32, String>) {
    match res {
        // `n` is `&i32`, `e` is `&String`
        Ok(n) => println!("Success: {}", n),
        Err(e) => println!("Error: {}", e),
    }
}

#[derive(Debug)]
struct Person {
    name: String,
    age: u32,
}

fn describe_person(person: &Person) {
    match person {
        // Destructuring a reference: name and age are references
        Person { name, age } => {
            println!("{} is {} years old", name, age);
        }
    }
}

fn find_adult(people: &[Person]) -> Option<&Person> {
    for person in people {
        match person {
            // `age` is `&u32`, so we dereference with `*` in the guard
            Person { age, .. } if *age >= 18 => return Some(person),
            _ => continue,
        }
    }
    None
}

// Comparing old style vs new style
fn old_style_matching(opt: &Option<String>) {
    match opt {
        // Old style required explicit `ref` to borrow
        &Some(ref s) => println!("Old style: {}", s),
        &None => println!("Old style: nothing"),
    }
}

fn new_style_matching(opt: &Option<String>) {
    // Modern Rust (match ergonomics) - cleaner!
    match opt {
        Some(s) => println!("New style: {}", s),
        None => println!("New style: nothing"),
    }
}

fn main() {
    println!("=== Match Ergonomics: Option ===");
    // Usage: match on references with automatic binding mode
    let opt = Some("hello".to_string());
    process_option(&opt);
    process_option(&None);

    // The original `opt` is still valid because we only borrowed it
    println!("Original opt is still valid: {:?}", opt);

    println!("\n=== Match Ergonomics: Result ===");
    let ok_result: Result<i32, String> = Ok(42);
    let err_result: Result<i32, String> = Err("oops".to_string());
    process_result(&ok_result);
    process_result(&err_result);

    println!("\n=== Match Ergonomics: Structs ===");
    let alice = Person {
        name: "Alice".to_string(),
        age: 30,
    };
    describe_person(&alice);

    println!("\n=== Finding Adults in Slice ===");
    let people = [
        Person {
            name: "Child".to_string(),
            age: 10,
        },
        Person {
            name: "Teen".to_string(),
            age: 16,
        },
        Person {
            name: "Adult".to_string(),
            age: 25,
        },
    ];

    if let Some(adult) = find_adult(&people) {
        println!("Found adult: {:?}", adult);
    }

    println!("\n=== Old Style vs New Style ===");
    let opt = Some("test".to_string());
    old_style_matching(&opt);
    new_style_matching(&opt);

    println!("\n=== How Match Ergonomics Work ===");
    println!("When matching on &T, the pattern's binding mode changes:");
    println!("  &Option<String> matched with Some(s) => s is &String");
    println!("  &Result<T, E>   matched with Ok(v)   => v is &T");
    println!();
    println!("The default binding mode \"propagates\" through nested patterns.");

    println!("\n=== When You Still Need Explicit ref/& ===");
    println!("1. When you want to move out of a reference (use `*`)");
    println!("2. When you want to take ownership in a by-value match");
    println!("3. When the automatic inference isn't what you want");
}
