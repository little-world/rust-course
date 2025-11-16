# 19. Declarative Macros

## Macro Patterns and Repetition

### Basic Macro Syntax

```rust
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
    say_hello!();
    greet!("Alice");

    let sum = calculate!(add 5, 3);
    let product = calculate!(mul 4, 7);
    println!("Sum: {}, Product: {}", sum, product);
}
```

### Fragment Specifiers

```rust
// Different fragment types
macro_rules! fragment_examples {
    // expr - expression
    ($e:expr) => {
        println!("Expression: {}", $e);
    };
    // ident - identifier
    ($i:ident) => {
        let $i = 42;
    };
    // ty - type
    ($t:ty) => {
        std::mem::size_of::<$t>()
    };
    // pat - pattern
    ($p:pat) => {
        match Some(42) {
            $p => println!("Matched!"),
            _ => println!("Not matched"),
        }
    };
    // stmt - statement
    ($s:stmt) => {
        $s
    };
    // block - block expression
    ($b:block) => {
        $b
    };
    // item - item (function, struct, etc.)
    ($it:item) => {
        $it
    };
    // meta - attribute contents
    ($m:meta) => {
        #[$m]
        fn dummy() {}
    };
    // tt - token tree (single token or group in delimiters)
    ($tt:tt) => {
        stringify!($tt)
    };
}

fn fragment_usage() {
    fragment_examples!(5 + 3);
    fragment_examples!(x);

    let size = fragment_examples!(i32);
    println!("Size of i32: {}", size);
}
```

### Repetition Patterns

```rust
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

// Multiple repetitions
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
    let v = create_vec![1, 2, 3, 4, 5];
    println!("Vector: {:?}", v);

    let total = sum!(1, 2, 3, 4, 5);
    println!("Sum: {}", total);

    let map = hash_map! {
        "name" => "Alice",
        "role" => "Developer",
    };
    println!("Map: {:?}", map);
}
```

### Nested Repetitions

```rust
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

// Multiple types of repetitions
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
    let mat = matrix![
        [1, 2, 3],
        [4, 5, 6],
        [7, 8, 9],
    ];

    for row in &mat {
        println!("{:?}", row);
    }

    function_table! {
        fn add(a: i32, b: i32) -> i32 {
            a + b
        }

        fn multiply(x: i32, y: i32) -> i32 {
            x * y
        }
    }

    println!("Add: {}", add(5, 3));
    println!("Multiply: {}", multiply(4, 7));
}
```

### Counting and Indexing

```rust
// Count arguments using recursive expansion
macro_rules! count {
    () => (0);
    ($head:tt $($tail:tt)*) => (1 + count!($($tail)*));
}

// Generate indexed names
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
    let count = count!(a b c d e);
    println!("Count: {}", count);

    let tuple = (1, "hello", 3.14, true);
    println!("First: {}", tuple_access!(tuple, 0));
    println!("Second: {}", tuple_access!(tuple, 1));
}
```

### Pattern Matching with Guards

```rust
// Match specific literals
macro_rules! match_literal {
    (true) => { "It's true!" };
    (false) => { "It's false!" };
    ($other:expr) => { "It's something else" };
}

// Match different types of expressions
macro_rules! describe_expr {
    ($e:expr) => {{
        println!("Expression: {}", stringify!($e));
        $e
    }};
}

// Complex pattern matching
macro_rules! operation {
    // Match literal operators
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
    println!("{}", match_literal!(true));
    println!("{}", match_literal!(42));

    let result = operation!(10, +, 5);
    println!("Result: {}", result);
}
```

## Hygiene and Scoping

### Variable Hygiene

```rust
// Macro-generated variables are hygienic
macro_rules! hygienic_example {
    () => {
        let x = 42; // This x doesn't conflict with outer x
        println!("Inner x: {}", x);
    };
}

fn hygiene_test() {
    let x = 100;
    println!("Outer x: {}", x);

    hygienic_example!();

    println!("Outer x again: {}", x); // Still 100
}
```

### Breaking Hygiene with $name:ident

```rust
// Intentionally capture variables from caller's scope
macro_rules! set_value {
    ($var:ident = $val:expr) => {
        let $var = $val;
    };
}

macro_rules! increment {
    ($var:ident) => {
        $var += 1;
    };
}

fn breaking_hygiene() {
    set_value!(counter = 0);
    println!("Counter: {}", counter); // Works because we used ident

    increment!(counter);
    println!("Counter: {}", counter);
}
```

### Macro Scope and Ordering

```rust
// Macros must be defined before use
macro_rules! early_macro {
    () => {
        println!("Defined early");
    };
}

fn can_use_early() {
    early_macro!(); // Works
}

// This won't compile if called before definition:
// late_macro!();

macro_rules! late_macro {
    () => {
        println!("Defined late");
    };
}

fn can_use_late() {
    late_macro!(); // Works
}
```

### Module Visibility

```rust
// Macros can be exported from modules
mod macros {
    #[macro_export]
    macro_rules! public_macro {
        () => {
            println!("Public macro from module");
        };
    }

    // Non-exported macros are private to the module
    macro_rules! private_macro {
        () => {
            println!("Private macro");
        };
    }

    pub fn use_private() {
        private_macro!();
    }
}

// Can use public_macro anywhere in the crate
fn visibility_example() {
    public_macro!();
    // private_macro!(); // Error: not in scope
    macros::use_private(); // But can call function that uses it
}
```

### Macro Re-export

```rust
// In lib.rs or main module
#[macro_use]
extern crate some_crate;

// Or selectively import
#[macro_use(specific_macro)]
extern crate some_crate;

// Re-export macro from dependency
#[doc(inline)]
pub use some_crate::their_macro;
```

### Context Capture

```rust
// Capture context intentionally
macro_rules! with_context {
    ($name:ident, $body:block) => {
        {
            let context = "macro context";
            let $name = context;
            $body
        }
    };
}

fn context_example() {
    with_context!(ctx, {
        println!("Context: {}", ctx);
    });
}
```

## DSL Construction

### SQL-like DSL

```rust
macro_rules! select {
    ($($field:ident),+ from $table:ident where $condition:expr) => {
        {
            let results = $table
                .iter()
                .filter(|row| $condition(row))
                .map(|row| {
                    ($(row.$field,)+)
                })
                .collect::<Vec<_>>();
            results
        }
    };
}

#[derive(Debug)]
struct User {
    id: u32,
    name: String,
    age: u32,
}

fn sql_dsl_example() {
    let users = vec![
        User { id: 1, name: "Alice".to_string(), age: 30 },
        User { id: 2, name: "Bob".to_string(), age: 25 },
        User { id: 3, name: "Carol".to_string(), age: 35 },
    ];

    let results = select!(name, age from users where |u: &User| u.age > 26);
    println!("Results: {:?}", results);
}
```

### Configuration DSL

```rust
macro_rules! config {
    {
        $(
            $section:ident {
                $(
                    $key:ident: $value:expr
                ),* $(,)?
            }
        )*
    } => {
        {
            use std::collections::HashMap;

            let mut config = HashMap::new();

            $(
                let mut section = HashMap::new();
                $(
                    section.insert(stringify!($key), $value.to_string());
                )*
                config.insert(stringify!($section), section);
            )*

            config
        }
    };
}

fn config_dsl_example() {
    let settings = config! {
        database {
            host: "localhost",
            port: 5432,
            name: "mydb",
        }
        server {
            host: "0.0.0.0",
            port: 8080,
            workers: 4,
        }
    };

    println!("Config: {:?}", settings);
}
```

### HTML-like DSL

```rust
macro_rules! html {
    // Self-closing tag
    (<$tag:ident />) => {
        format!("<{} />", stringify!($tag))
    };

    // Tag with content
    (<$tag:ident> $($content:tt)* </$close:ident>) => {
        format!("<{}>{}</{}>",
            stringify!($tag),
            html!($($content)*),
            stringify!($close))
    };

    // Text content
    ($text:expr) => {
        $text.to_string()
    };

    // Multiple elements
    ($($element:tt)*) => {
        {
            let mut result = String::new();
            $(
                result.push_str(&html!($element));
            )*
            result
        }
    };
}

fn html_dsl_example() {
    let page = html! {
        <html>
            <body>
                <h1>"Hello, World!"</h1>
                <p>"This is a paragraph."</p>
                <br />
            </body>
        </html>
    };

    println!("{}", page);
}
```

### Builder Pattern DSL

```rust
macro_rules! builder {
    (
        struct $name:ident {
            $(
                $field:ident: $type:ty
            ),* $(,)?
        }
    ) => {
        struct $name {
            $(
                $field: $type,
            )*
        }

        paste::paste! {
            struct [<$name Builder>] {
                $(
                    $field: Option<$type>,
                )*
            }

            impl [<$name Builder>] {
                fn new() -> Self {
                    Self {
                        $(
                            $field: None,
                        )*
                    }
                }

                $(
                    fn $field(mut self, value: $type) -> Self {
                        self.$field = Some(value);
                        self
                    }
                )*

                fn build(self) -> Result<$name, String> {
                    Ok($name {
                        $(
                            $field: self.$field
                                .ok_or_else(|| format!("Missing field: {}", stringify!($field)))?,
                        )*
                    })
                }
            }
        }
    };
}

// Note: This example requires the paste crate for identifier concatenation
// Add to Cargo.toml: paste = "1.0"
```

### State Machine DSL

```rust
macro_rules! state_machine {
    (
        states: [$($state:ident),* $(,)?]
        transitions: {
            $(
                $from:ident -> $to:ident on $event:ident
            )*
        }
    ) => {
        #[derive(Debug, Clone, Copy, PartialEq)]
        enum State {
            $($state),*
        }

        #[derive(Debug)]
        enum Event {
            $($event),*
        }

        struct StateMachine {
            current_state: State,
        }

        impl StateMachine {
            fn new(initial: State) -> Self {
                StateMachine { current_state: initial }
            }

            fn transition(&mut self, event: Event) -> Result<(), String> {
                let new_state = match (self.current_state, event) {
                    $(
                        (State::$from, Event::$event) => State::$to,
                    )*
                    _ => return Err(format!("Invalid transition from {:?}", self.current_state)),
                };

                self.current_state = new_state;
                Ok(())
            }

            fn current(&self) -> State {
                self.current_state
            }
        }
    };
}

fn state_machine_example() {
    state_machine! {
        states: [Idle, Running, Paused, Stopped]
        transitions: {
            Idle -> Running on Start
            Running -> Paused on Pause
            Paused -> Running on Resume
            Running -> Stopped on Stop
            Paused -> Stopped on Stop
        }
    }

    let mut sm = StateMachine::new(State::Idle);
    println!("Initial state: {:?}", sm.current());

    sm.transition(Event::Start).unwrap();
    println!("After start: {:?}", sm.current());

    sm.transition(Event::Pause).unwrap();
    println!("After pause: {:?}", sm.current());
}
```

## Code Generation Patterns

### Generating Enums with Associated Data

```rust
macro_rules! generate_enum {
    (
        $name:ident {
            $(
                $variant:ident($($field:ty),*)
            ),* $(,)?
        }
    ) => {
        enum $name {
            $(
                $variant($($field),*),
            )*
        }

        impl $name {
            $(
                paste::paste! {
                    fn [<is_ $variant:lower>](&self) -> bool {
                        matches!(self, $name::$variant(..))
                    }

                    fn [<as_ $variant:lower>](&self) -> Option<&($($field),*)> {
                        if let $name::$variant(ref data) = self {
                            Some(data)
                        } else {
                            None
                        }
                    }
                }
            )*
        }
    };
}
```

### Generating Struct Accessors

```rust
macro_rules! accessors {
    (
        struct $name:ident {
            $(
                $field:ident: $type:ty
            ),* $(,)?
        }
    ) => {
        struct $name {
            $(
                $field: $type,
            )*
        }

        impl $name {
            // Generate getters
            $(
                pub fn $field(&self) -> &$type {
                    &self.$field
                }

                paste::paste! {
                    pub fn [<$field _mut>](&mut self) -> &mut $type {
                        &mut self.$field
                    }

                    pub fn [<set_ $field>](&mut self, value: $type) {
                        self.$field = value;
                    }
                }
            )*
        }
    };
}

accessors! {
    struct Person {
        name: String,
        age: u32,
    }
}

fn accessor_example() {
    let mut person = Person {
        name: "Alice".to_string(),
        age: 30,
    };

    println!("Name: {}", person.name());
    person.set_age(31);
    println!("Age: {}", person.age());
}
```

### Trait Implementation Generation

```rust
macro_rules! impl_from {
    ($from:ty => $to:ty, $variant:ident) => {
        impl From<$from> for $to {
            fn from(value: $from) -> Self {
                <$to>::$variant(value)
            }
        }
    };
}

enum Value {
    Integer(i64),
    Float(f64),
    String(String),
    Bool(bool),
}

impl_from!(i64 => Value, Integer);
impl_from!(f64 => Value, Float);
impl_from!(String => Value, String);
impl_from!(bool => Value, Bool);

fn trait_impl_example() {
    let int_value: Value = 42i64.into();
    let string_value: Value = "hello".to_string().into();
}
```

### Test Generation

```rust
macro_rules! generate_tests {
    (
        $fn_name:ident {
            $(
                $test_name:ident: ($($input:expr),*) => $expected:expr
            ),* $(,)?
        }
    ) => {
        #[cfg(test)]
        mod tests {
            use super::*;

            $(
                #[test]
                fn $test_name() {
                    let result = $fn_name($($input),*);
                    assert_eq!(result, $expected);
                }
            )*
        }
    };
}

fn add(a: i32, b: i32) -> i32 {
    a + b
}

generate_tests! {
    add {
        test_add_positive: (2, 3) => 5,
        test_add_negative: (-2, -3) => -5,
        test_add_zero: (0, 5) => 5,
    }
}
```

### Bitflags Pattern

```rust
macro_rules! bitflags {
    (
        $(#[$attr:meta])*
        $vis:vis struct $name:ident: $type:ty {
            $(
                $(#[$flag_attr:meta])*
                const $flag:ident = $value:expr;
            )*
        }
    ) => {
        $(#[$attr])*
        #[derive(Copy, Clone, PartialEq, Eq)]
        $vis struct $name {
            bits: $type,
        }

        impl $name {
            $(
                $(#[$flag_attr])*
                pub const $flag: Self = Self { bits: $value };
            )*

            pub const fn empty() -> Self {
                Self { bits: 0 }
            }

            pub const fn all() -> Self {
                Self { bits: $(0 | $value)|* }
            }

            pub const fn contains(&self, other: Self) -> bool {
                (self.bits & other.bits) == other.bits
            }

            pub const fn insert(&mut self, other: Self) {
                self.bits |= other.bits;
            }

            pub const fn remove(&mut self, other: Self) {
                self.bits &= !other.bits;
            }
        }

        impl std::ops::BitOr for $name {
            type Output = Self;

            fn bitor(self, rhs: Self) -> Self::Output {
                Self { bits: self.bits | rhs.bits }
            }
        }
    };
}

bitflags! {
    pub struct Permissions: u32 {
        const READ = 0b001;
        const WRITE = 0b010;
        const EXECUTE = 0b100;
    }
}

fn bitflags_example() {
    let perms = Permissions::READ | Permissions::WRITE;
    println!("Has read: {}", perms.contains(Permissions::READ));
    println!("Has execute: {}", perms.contains(Permissions::EXECUTE));
}
```

## Macro Debugging

### Using cargo expand

```bash
# Install cargo-expand
cargo install cargo-expand

# Expand macros in your code
cargo expand

# Expand specific module
cargo expand module_name

# Expand with color output
cargo expand --color always
```

### Debug Printing with compile_error!

```rust
macro_rules! debug_macro {
    ($($tt:tt)*) => {
        compile_error!(concat!("Macro input: ", stringify!($($tt)*)));
    };
}

// This will show the exact input at compile time
// debug_macro!(some input here);
```

### Tracing Macro Expansion

```rust
macro_rules! trace {
    ($($arg:tt)*) => {
        {
            eprintln!("Macro trace: {}", stringify!($($arg)*));
            $($arg)*
        }
    };
}

fn tracing_example() {
    let x = trace!(5 + 3);
    println!("Result: {}", x);
}
```

### Log Macro Pattern

```rust
macro_rules! log_call {
    ($func:ident($($arg:expr),*)) => {
        {
            println!("Calling {}({:?})", stringify!($func), ($($arg,)*));
            let result = $func($($arg),*);
            println!("{} returned: {:?}", stringify!($func), result);
            result
        }
    };
}

fn add(a: i32, b: i32) -> i32 {
    a + b
}

fn logging_example() {
    let result = log_call!(add(5, 3));
}
```

### Assertion Macros

```rust
macro_rules! debug_assert_matches {
    ($expr:expr, $pat:pat) => {
        if cfg!(debug_assertions) {
            match $expr {
                $pat => {},
                ref e => panic!(
                    "assertion failed: `{:?}` does not match `{}`",
                    e,
                    stringify!($pat)
                ),
            }
        }
    };
}

fn assertion_example() {
    let value = Some(42);
    debug_assert_matches!(value, Some(_));
}
```

### Stringify and Debugging

```rust
macro_rules! explain {
    ($expr:expr) => {
        {
            let value = $expr;
            println!("{} = {:?}", stringify!($expr), value);
            value
        }
    };
}

fn stringify_example() {
    let result = explain!(2 + 2);
    let complex = explain!({
        let x = 10;
        let y = 20;
        x + y
    });
}
```

### Compile-Time Type Checking

```rust
macro_rules! ensure_type {
    ($expr:expr, $type:ty) => {
        {
            let value: $type = $expr;
            value
        }
    };
}

fn type_checking_example() {
    let x = ensure_type!(42, i32);
    // This would fail at compile time:
    // let y = ensure_type!("hello", i32);
}
```

### Conditional Compilation Debugging

```rust
macro_rules! debug {
    ($($arg:tt)*) => {
        if cfg!(debug_assertions) {
            eprintln!("[DEBUG] {}", format!($($arg)*));
        }
    };
}

fn conditional_debug() {
    debug!("This appears only in debug builds");
    debug!("Value: {}, Other: {}", 42, "test");
}
```

### Macro Recursion Debugging

```rust
// Track recursion depth
macro_rules! recursive_debug {
    (@depth 0, $($tt:tt)*) => {
        compile_error!("Maximum recursion depth reached");
    };
    (@depth $depth:expr, []) => {
        println!("Base case reached at depth {}", $depth);
    };
    (@depth $depth:expr, [$head:expr $(, $tail:expr)*]) => {
        {
            println!("Depth {}: processing {}", $depth, stringify!($head));
            $head;
            recursive_debug!(@depth $depth - 1, [$($tail),*]);
        }
    };
    ($($tt:tt)*) => {
        recursive_debug!(@depth 10, [$($tt)*]);
    };
}
```

### Testing Macro Output

```rust
#[cfg(test)]
mod macro_tests {
    macro_rules! test_expansion {
        () => {
            {
                let x = 42;
                x * 2
            }
        };
    }

    #[test]
    fn test_macro_result() {
        let result = test_expansion!();
        assert_eq!(result, 84);
    }
}
```

### Pretty-Print with syn (Procedural Macro Context)

```rust
// For debugging declarative macros that generate complex code
macro_rules! generate_struct {
    ($name:ident { $($field:ident: $type:ty),* }) => {
        // Use stringify to see the generated code
        const _: &str = stringify! {
            struct $name {
                $($field: $type),*
            }
        };

        struct $name {
            $($field: $type),*
        }
    };
}
```

### Common Debugging Patterns

```rust
// 1. Echo pattern - see what the macro receives
macro_rules! echo {
    ($($tt:tt)*) => {
        {
            println!("Macro received: {}", stringify!($($tt)*));
            $($tt)*
        }
    };
}

// 2. Type introspection
macro_rules! show_type {
    ($expr:expr) => {
        {
            fn type_of<T>(_: &T) -> &'static str {
                std::any::type_name::<T>()
            }
            let value = $expr;
            println!("Type of {}: {}", stringify!($expr), type_of(&value));
            value
        }
    };
}

// 3. Count token trees
macro_rules! count_tts {
    () => { 0 };
    ($odd:tt $($rest:tt)*) => { 1 + count_tts!($($rest)*) };
}

fn debugging_patterns() {
    echo!(println!("Hello"));

    let x = show_type!(vec![1, 2, 3]);

    const COUNT: usize = count_tts!(a b c d e);
    println!("Token count: {}", COUNT);
}
```

### Error Messages with Custom Diagnostics

```rust
macro_rules! validate_input {
    (empty) => {
        compile_error!("Input cannot be empty!");
    };
    ($($valid:tt)*) => {
        // Process valid input
        $($valid)*
    };
}

// Better error messages
macro_rules! require_literal {
    ($lit:literal) => { $lit };
    ($other:expr) => {
        compile_error!(concat!(
            "Expected a literal value, got expression: ",
            stringify!($other)
        ));
    };
}
```

This comprehensive guide covers all essential patterns for declarative macros in Rust, from basic usage to advanced DSL construction and debugging techniques.
