// Pattern 1: Macro Patterns and Repetition
// Pattern matching on syntax to generate code at compile-time

// Simple macro without arguments
macro_rules! say_hello {
    () => {
        println!("Hello, World!");
    };
}

// Macro with a single argument
macro_rules! greet {
    ($name:expr) => {
        println!("Hello, {}!", $name);
    };
}

// Macro with multiple patterns
macro_rules! calculate {
    (add $a:expr, $b:expr) => {
        $a + $b
    };
    (sub $a:expr, $b:expr) => {
        $a - $b
    };
    (mul $a:expr, $b:expr) => {
        $a * $b
    };
}

fn basic_examples() {
    println!("=== Basic Pattern Matching ===\n");

    say_hello!();
    greet!("Alice");

    let sum = calculate!(add 5, 3);
    let product = calculate!(mul 4, 7);
    println!("Sum: {}, Product: {}", sum, product);
}

// Basic repetition with *
macro_rules! create_vec {
    ($($elem:expr),*) => {
        {
            let mut v = Vec::new();
            $(
                v.push($elem);
            )*
            v
        }
    };
}

// Repetition with + (one or more)
macro_rules! sum {
    ($($num:expr),+) => {
        {
            let mut total = 0;
            $(
                total += $num;
            )+
            total
        }
    };
}

// Optional repetition with ?
macro_rules! optional_value {
    ($val:expr $(, $default:expr)?) => {
        Some($val) $(.or(Some($default)))?
    };
}

// Multiple repetitions - HashMap literal
macro_rules! hash_map {
    ($($key:expr => $val:expr),* $(,)?) => {
        {
            let mut map = std::collections::HashMap::new();
            $(
                map.insert($key, $val);
            )*
            map
        }
    };
}

fn repetition_examples() {
    println!("\n=== Repetition Patterns ===\n");

    let v = create_vec![1, 2, 3, 4, 5];
    println!("Vector: {:?}", v);

    let total = sum!(1, 2, 3, 4, 5);
    println!("Sum: {}", total);

    let v1 = optional_value!(42);
    let v2 = optional_value!(42, 100);
    println!("Optional values: {:?}, {:?}", v1, v2);

    let map = hash_map! {
        "name" => "Alice",
        "role" => "Developer",
    };
    println!("Map: {:?}", map);
}

// Matrix creation with nested repetitions
macro_rules! matrix {
    ($([$($elem:expr),*]),* $(,)?) => {
        vec![
            $(
                vec![$($elem),*]
            ),*
        ]
    };
}

// Generate multiple functions from a template
macro_rules! function_table {
    {
        $(
            fn $name:ident($($param:ident: $type:ty),*) -> $ret:ty $body:block
        )*
    } => {
        $(
            fn $name($($param: $type),*) -> $ret $body
        )*
    };
}

fn nested_examples() {
    println!("\n=== Nested Repetitions ===\n");

    let mat = matrix![
        [1, 2, 3],
        [4, 5, 6],
        [7, 8, 9],
    ];

    for row in &mat {
        println!("{:?}", row);
    }

    function_table! {
        fn add_fn(a: i32, b: i32) -> i32 {
            a + b
        }

        fn multiply_fn(x: i32, y: i32) -> i32 {
            x * y
        }
    }

    println!("\nAdd: {}", add_fn(5, 3));
    println!("Multiply: {}", multiply_fn(4, 7));
}

// Count arguments using recursive expansion
macro_rules! count {
    () => (0);
    ($head:tt $($tail:tt)*) => (1 + count!($($tail)*));
}

// Creates a struct with the specified field names
macro_rules! create_fields {
    ($($name:ident),*) => {
        struct GeneratedStruct {
            $(
                $name: i32,
            )*
        }
    };
}

// Tuple indexing pattern
macro_rules! tuple_access {
    ($tuple:expr, 0) => { $tuple.0 };
    ($tuple:expr, 1) => { $tuple.1 };
    ($tuple:expr, 2) => { $tuple.2 };
    ($tuple:expr, 3) => { $tuple.3 };
}

fn counting_examples() {
    println!("\n=== Counting and Indexing ===\n");

    const N: usize = count!(a b c d e);
    println!("Count: {}", N);

    let tuple = (1, "hello", 3.14, true);
    println!("First: {}", tuple_access!(tuple, 0));
    println!("Second: {}", tuple_access!(tuple, 1));
}

// Match specific literals
macro_rules! match_literal {
    (true) => { "It's true!" };
    (false) => { "It's false!" };
    ($other:expr) => { "It's something else" };
}

// stringify! turns code into a string literal
macro_rules! describe_expr {
    ($e:expr) => {{
        println!("Expression: {}", stringify!($e));
        $e
    }};
}

// Match literal operators as tokens
macro_rules! operation {
    ($a:expr, +, $b:expr) => {
        $a + $b
    };
    ($a:expr, -, $b:expr) => {
        $a - $b
    };
    ($a:expr, *, $b:expr) => {
        $a * $b
    };
    ($a:expr, /, $b:expr) => {
        $a / $b
    };
}

fn pattern_matching_examples() {
    println!("\n=== Pattern Matching with Guards ===\n");

    println!("{}", match_literal!(true));
    println!("{}", match_literal!(42));

    let x = describe_expr!(2 + 2);
    println!("Result: {}", x);

    let result = operation!(10, +, 5);
    println!("Operation result: {}", result);
}

fn main() {
    println!("=== Declarative Macros: Patterns and Repetition ===\n");

    basic_examples();
    repetition_examples();
    nested_examples();
    counting_examples();
    pattern_matching_examples();

    println!("\nMacro patterns demo completed");
}
