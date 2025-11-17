# 19. Declarative Macros

## Overview

Declarative macros (also called "macros by example" or `macro_rules!` macros) are Rust's code generation system that operates through pattern matching. Unlike functions, which operate on values at runtime, macros operate on syntax at compile time—they take code as input, match it against patterns, and generate new code as output.

**Why Macros Exist**

Rust's type system is powerful, but it can't express everything. Consider these scenarios:

- **Repetitive boilerplate**: Writing `println!`, `vec!`, or `assert_eq!` with variable numbers of arguments would be impossible with functions alone
- **Compile-time code generation**: Creating specialized code for each invocation without runtime overhead
- **Domain-specific languages (DSLs)**: Building mini-languages that look natural in Rust but compile to efficient code
- **Zero-cost abstractions**: Generating code that's as fast as hand-written code

Functions can't do these things because they have fixed signatures—they can't accept a variable number of arguments of different types, and they can't generate new types or items.

**Declarative vs Procedural Macros**

Rust has two macro systems:

- **Declarative macros (`macro_rules!`)**: Pattern matching on syntax. Simpler to write, limited in power. Great for most use cases.
- **Procedural macros**: Full Rust code that manipulates syntax trees. More powerful but requires a separate crate. Covered in Chapter 20.

This chapter focuses on declarative macros.

**How Declarative Macros Work**

A declarative macro is a set of rules, each with a pattern and a template:

```rust
macro_rules! my_macro {
    (pattern1) => { template1 };
    (pattern2) => { template2 };
}
```

When you invoke `my_macro!(some code)`, Rust:
1. Tries to match `some code` against each pattern in order
2. When a pattern matches, expands to the corresponding template
3. Substitutes captured fragments from the pattern into the template
4. Returns the generated code to be compiled

**Key Concepts**

This chapter covers:
- **Pattern matching**: Fragment specifiers (`expr`, `ident`, `ty`, etc.) and repetitions
- **Hygiene**: How macros avoid variable name collisions
- **DSL construction**: Building mini-languages that compile to Rust
- **Code generation**: Automating repetitive code creation
- **Debugging**: Tools and techniques for understanding macro expansions

---

## Macro Patterns and Repetition

### Basic Macro Syntax

Every macro starts with pattern matching. The left side of `=>` is what you write when invoking the macro; the right side is what code gets generated.

**The simplest patterns:**
- `()` matches empty invocation: `my_macro!()`
- `($name:expr)` matches an expression and binds it to `$name`
- Multiple patterns create different "overloads" for different invocation styles

```rust
//===============================
// Simple macro without arguments
//===============================
macro_rules! say_hello {
    () => {
        println!("Hello, World!");
    };
}

//=============================
// Macro with a single argument
//=============================
// $name:expr means "match any expression, call it $name"
macro_rules! greet {
    ($name:expr) => {
        println!("Hello, {}!", $name);
    };
}

//=============================
// Macro with multiple patterns
//=============================
// This demonstrates how one macro can have different "overloads"
macro_rules! calculate {
    // Pattern 1: literal "add" followed by two expressions
    (add $a:expr, $b:expr) => {
        $a + $b
    };
    // Pattern 2: literal "sub" followed by two expressions
    (sub $a:expr, $b:expr) => {
        $a - $b
    };
    // Pattern 3: literal "mul" followed by two expressions
    (mul $a:expr, $b:expr) => {
        $a * $b
    };
}

fn basic_examples() {
    say_hello!();  // Expands to: println!("Hello, World!");
    greet!("Alice");  // Expands to: println!("Hello, {}!", "Alice");

    let sum = calculate!(add 5, 3);  // Expands to: 5 + 3
    let product = calculate!(mul 4, 7);  // Expands to: 4 * 7
    println!("Sum: {}, Product: {}", sum, product);
}
```

### Fragment Specifiers

Fragment specifiers tell the macro what kind of syntax to expect. Each specifier matches a different part of Rust's grammar.

**Why fragment specifiers matter:**
- **Type safety**: `$x:ty` only matches types, preventing runtime errors
- **Flexibility**: Different fragments let you match exactly what you need
- **Hygiene**: Some fragments (like `ident`) interact with macro hygiene

**Common specifiers:**
- `expr`: Any expression (`2 + 2`, `vec![1, 2, 3]`, `if x { y } else { z }`)
- `ident`: An identifier (variable name, function name, etc.)
- `ty`: A type (`i32`, `Vec<String>`, `&'a str`)
- `pat`: A pattern (used in `match`, `let`, function parameters)
- `stmt`: A statement (ends with `;`)
- `block`: A block expression (`{ ... }`)
- `item`: An item (function, struct, impl, etc.)
- `tt`: Token tree (a single token or group in delimiters)

```rust
//=========================
// Different fragment types
//=========================
macro_rules! fragment_examples {
    // expr - expression
    // Matches: 5 + 3, vec![1, 2], function_call()
    ($e:expr) => {
        println!("Expression: {}", $e);
    };
    // ident - identifier
    // Matches: x, my_var, SomeStruct
    // This creates a variable with that name
    ($i:ident) => {
        let $i = 42;
    };
    // ty - type
    // Matches: i32, Vec<String>, &str
    ($t:ty) => {
        std::mem::size_of::<$t>()
    };
    // pat - pattern
    // Matches: Some(42), _, x @ 1..=5
    ($p:pat) => {
        match Some(42) {
            $p => println!("Matched!"),
            _ => println!("Not matched"),
        }
    };
    // stmt - statement
    // Matches: let x = 5;, println!("hi");
    ($s:stmt) => {
        $s
    };
    // block - block expression
    // Matches: { let x = 5; x * 2 }
    ($b:block) => {
        $b
    };
    // item - item (function, struct, impl, etc.)
    // Matches: fn foo() {}, struct Bar {}, impl Trait for Type {}
    ($it:item) => {
        $it
    };
    // meta - attribute contents
    // Matches: derive(Debug), inline, cfg(test)
    ($m:meta) => {
        #[$m]
        fn dummy() {}
    };
    // tt - token tree (single token or group in delimiters)
    // Matches: x, (a, b), {code}, "string"
    ($tt:tt) => {
        stringify!($tt)
    };
}

fn fragment_usage() {
    fragment_examples!(5 + 3);  // expr: prints "Expression: 8"
    fragment_examples!(x);  // ident: creates variable x = 42

    let size = fragment_examples!(i32);  // ty: returns 4 (size of i32)
    println!("Size of i32: {}", size);
}
```

### Repetition Patterns

Repetitions are the most powerful feature of declarative macros. They let you match and generate variable amounts of code.

**Repetition syntax:**
- `$(...)*` matches zero or more times
- `$(...)+` matches one or more times
- `$(...)?` matches zero or one time (optional)
- Separator: `$(...),*` matches comma-separated items

**Why repetitions matter:**
Creating `vec![1, 2, 3]` or `println!("{} {}", a, b)` requires matching an arbitrary number of elements—impossible without repetitions.

```rust
//========================
// Basic repetition with *
//========================
// This is how vec! works internally
macro_rules! create_vec {
    // $($elem:expr),* means:
    // - Match zero or more expressions
    // - Separated by commas
    // - Bind each to $elem
    ($($elem:expr),*) => {
        {
            let mut v = Vec::new();
            $(
                v.push($elem);  // Repeat this for each matched $elem
            )*
            v
        }
    };
}

//================================
// Repetition with + (one or more)
//================================
// Requires at least one argument, unlike *
macro_rules! sum {
    ($($num:expr),+) => {
        {
            let mut total = 0;
            $(
                total += $num;  // Repeat for each number
            )+
            total
        }
    };
}

//===========================
// Optional repetition with ?
//===========================
// Allows an optional second argument
macro_rules! optional_value {
    ($val:expr $(, $default:expr)?) => {
        Some($val) $(.or(Some($default)))?
    };
}

//=====================
// Multiple repetitions
//=====================
// This is how HashMap literals could work
macro_rules! hash_map {
    // Trailing comma is optional: $(,)?
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
    // create_vec! accepts any number of arguments
    let v = create_vec![1, 2, 3, 4, 5];
    println!("Vector: {:?}", v);

    // sum! requires at least one argument
    let total = sum!(1, 2, 3, 4, 5);
    println!("Sum: {}", total);

    // hash_map! with optional trailing comma
    let map = hash_map! {
        "name" => "Alice",
        "role" => "Developer",  // Trailing comma works
    };
    println!("Map: {:?}", map);
}
```

### Nested Repetitions

Nested repetitions allow matching multi-dimensional structures like matrices or tables.

**When to use nested repetitions:**
- Generating multi-dimensional data structures
- Processing tables or grids
- Creating multiple related items with shared structure

```rust
//========================================
// Matrix creation with nested repetitions
//========================================
// Outer repetition: rows
//=======================================
// Inner repetition: elements in each row
//=======================================
macro_rules! matrix {
    ($([$($elem:expr),*]),* $(,)?) => {
        vec![
            $(
                vec![$($elem),*]  // Inner: elements in row
            ),*  // Outer: rows
        ]
    };
}

//==============================
// Multiple types of repetitions
//==============================
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
    // Create a 3x3 matrix
    let mat = matrix![
        [1, 2, 3],
        [4, 5, 6],
        [7, 8, 9],
    ];

    for row in &mat {
        println!("{:?}", row);
    }

    // Generate multiple functions at once
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

Counting elements in a macro requires recursive expansion—declarative macros don't have loops or counters.

**The counting trick:**
Use recursion to add 1 for each element until you hit the base case (empty).

```rust
//==========================================
// Count arguments using recursive expansion
//==========================================
// How it works:
//=========================================================================================================
// count!(a b c) → 1 + count!(b c) → 1 + 1 + count!(c) → 1 + 1 + 1 + count!() → 1 + 1 + 1 + 0 → 3
//=========================================================================================================
macro_rules! count {
    () => (0);  // Base case: no tokens = 0
    ($head:tt $($tail:tt)*) => (1 + count!($($tail)*));  // Recursive: 1 + count(rest)
}

//=======================
// Generate indexed names
//=======================
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

//=======================
// Tuple indexing pattern
//=======================
// Manually provides accessors for tuple elements
macro_rules! tuple_access {
    ($tuple:expr, 0) => { $tuple.0 };
    ($tuple:expr, 1) => { $tuple.1 };
    ($tuple:expr, 2) => { $tuple.2 };
    ($tuple:expr, 3) => { $tuple.3 };
}

fn counting_examples() {
    // Count tokens at compile time
    let count = count!(a b c d e);
    println!("Count: {}", count);  // Prints 5

    let tuple = (1, "hello", 3.14, true);
    println!("First: {}", tuple_access!(tuple, 0));
    println!("Second: {}", tuple_access!(tuple, 1));
}
```

### Pattern Matching with Guards

Pattern matching in macros can match specific literals or any syntax, giving you fine control over what invocations are valid.

**Pattern matching strategies:**
- Match specific literals to create keyword-based DSLs
- Match general patterns with fragment specifiers
- Combine both for flexible yet constrained syntax

```rust
//========================
// Match specific literals
//========================
// This creates a type-safe boolean matcher
macro_rules! match_literal {
    (true) => { "It's true!" };
    (false) => { "It's false!" };
    ($other:expr) => { "It's something else" };
}

//=====================================
// Match different types of expressions
//=====================================
// stringify! turns code into a string literal
macro_rules! describe_expr {
    ($e:expr) => {{
        println!("Expression: {}", stringify!($e));
        $e
    }};
}

//=========================
// Complex pattern matching
//=========================
// Match literal operators to create a calculator DSL
macro_rules! operation {
    // Match literal operators as tokens
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
    println!("{}", match_literal!(true));  // "It's true!"
    println!("{}", match_literal!(42));    // "It's something else"

    // operation! creates a mini calculator language
    let result = operation!(10, +, 5);
    println!("Result: {}", result);  // 15
}
```

---

## Hygiene and Scoping

Macro hygiene prevents name collisions between macro-generated code and the surrounding code. This is one of Rust's key innovations over C-style macros.

### Variable Hygiene

**The hygiene problem in C:**
```c
#define SWAP(a, b) { int temp = a; a = b; b = temp; }
int temp = 5;
SWAP(temp, x);  // BUG: 'temp' in macro collides with user's 'temp'
```

**Rust's solution:**
Variables created inside macros exist in a different "syntax context" and can't collide with user code.

```rust
//=======================================
// Macro-generated variables are hygienic
//=======================================
// The 'x' inside the macro is separate from the 'x' outside
macro_rules! hygienic_example {
    () => {
        let x = 42; // This x doesn't conflict with outer x
        println!("Inner x: {}", x);
    };
}

fn hygiene_test() {
    let x = 100;
    println!("Outer x: {}", x);  // Prints 100

    hygienic_example!();  // Prints "Inner x: 42"

    println!("Outer x again: {}", x);  // Still 100 (not affected by macro)
}
```

### Breaking Hygiene with $name:ident

Sometimes you *want* to create or modify variables in the caller's scope. Use `ident` fragment specifiers to intentionally break hygiene.

**When to break hygiene:**
- DSLs that create variables for the user
- Macros that modify existing variables
- Code generation patterns where the user expects side effects

```rust
//====================================================
// Intentionally capture variables from caller's scope
//====================================================
// The ident fragment specifier creates a variable in the outer scope
macro_rules! set_value {
    ($var:ident = $val:expr) => {
        let $var = $val;  // Creates $var in caller's scope
    };
}

macro_rules! increment {
    ($var:ident) => {
        $var += 1;  // Modifies caller's variable
    };
}

fn breaking_hygiene() {
    set_value!(counter = 0);
    println!("Counter: {}", counter);  // Works because we used ident

    increment!(counter);
    println!("Counter: {}", counter);  // 1
}
```

### Macro Scope and Ordering

Unlike functions, macros must be defined *before* they're used. Macros are expanded in a single pass through the file.

**Scoping rules:**
- Macros are visible from their definition point onward
- Macros can call other macros (including themselves recursively)
- `#[macro_export]` makes a macro available to other crates

```rust
//==================================
// Macros must be defined before use
//==================================
macro_rules! early_macro {
    () => {
        println!("Defined early");
    };
}

fn can_use_early() {
    early_macro!(); // Works - macro defined above
}

//================================================
// This won't compile if called before definition:
//================================================
// late_macro!();  // ERROR: macro not yet defined

macro_rules! late_macro {
    () => {
        println!("Defined late");
    };
}

fn can_use_late() {
    late_macro!(); // Works - macro defined above this function
}
```

### Module Visibility

Macros have special visibility rules compared to other items.

**Key differences:**
- Macros don't respect privacy boundaries by default
- `#[macro_export]` exports to the crate root, not the current module
- Importing macros requires special syntax in older Rust editions

```rust
//====================================
// Macros can be exported from modules
//====================================
mod macros {
    // #[macro_export] makes this available at crate root
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
        private_macro!();  // Module can use its own private macros
    }
}

//===========================================
// Can use public_macro anywhere in the crate
//===========================================
fn visibility_example() {
    public_macro!();
    // private_macro!(); // Error: not in scope
    macros::use_private(); // But can call function that uses it
}
```

### Context Capture

Macros can intentionally capture context to provide convenient DSLs.

**The context pattern:**
Create a scope with predefined variables that the user's code can access.

```rust
//==============================
// Capture context intentionally
//==============================
// Provides a 'context' variable to the user's block
macro_rules! with_context {
    ($name:ident, $body:block) => {
        {
            let context = "macro context";
            let $name = context;  // Bind to user's chosen name
            $body  // User code can access $name
        }
    };
}

fn context_example() {
    with_context!(ctx, {
        println!("Context: {}", ctx);  // ctx is provided by the macro
    });
}
```

---

## DSL Construction

Domain-Specific Languages (DSLs) are mini-languages embedded in Rust. Macros make DSLs compile to efficient Rust code without runtime interpretation.

### SQL-like DSL

This demonstrates how macros can create query-like syntax that compiles to iterator chains.

**Why build DSLs:**
- More readable than raw Rust for domain-specific tasks
- Compile-time validation of syntax
- Zero runtime overhead (compiles to normal Rust)

```rust
//==================================================
// SQL-like query syntax compiled to iterator chains
//==================================================
macro_rules! select {
    // select field1, field2 from table where condition
    ($($field:ident),+ from $table:ident where $condition:expr) => {
        {
            let results = $table
                .iter()
                .filter(|row| $condition(row))  // WHERE clause
                .map(|row| {
                    ($(row.$field,)+)  // SELECT clause
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

    // Looks like SQL, compiles to efficient iterator code
    let results = select!(name, age from users where |u: &User| u.age > 26);
    println!("Results: {:?}", results);
    // Output: [("Alice", 30), ("Carol", 35)]
}
```

### Configuration DSL

Create a structured configuration syntax that parses at compile time.

```rust
//========================================
// Configuration DSL with nested structure
//========================================
// section { key: value, key: value }
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
    // Produces a nested HashMap structure
}
```

### HTML-like DSL

Generate HTML strings with XML-like syntax (simplified version of real templating engines).

```rust
//===========================
// HTML-like DSL (simplified)
//===========================
macro_rules! html {
    // Self-closing tag: <br />
    (<$tag:ident />) => {
        format!("<{} />", stringify!($tag))
    };

    // Tag with content: <p>text</p>
    (<$tag:ident> $($content:tt)* </$close:ident>) => {
        format!("<{}>{}</{}>",
            stringify!($tag),
            html!($($content)*),  // Recursively process content
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
    // Produces: <html><body><h1>Hello, World!</h1><p>This is a paragraph.</p><br /></body></html>
}
```

### State Machine DSL

Define state machines declaratively, compiling to efficient match-based transitions.

```rust
//==================
// State machine DSL
//==================
// states: [State1, State2]
//===========================================
// transitions: { State1 -> State2 on Event }
//===========================================
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
                // Pattern match on (current_state, event) to find transitions
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
    println!("After start: {:?}", sm.current());  // Running

    sm.transition(Event::Pause).unwrap();
    println!("After pause: {:?}", sm.current());  // Paused
}
```

---

## Code Generation Patterns

Code generation with macros eliminates boilerplate by creating repetitive code automatically.

### Generating Struct Accessors

Automatically generate getters, setters, and mutable accessors for struct fields.

**Why generate accessors:**
- Encapsulation without manual boilerplate
- Consistent naming conventions
- Easy to add validation or logging later

```rust
//=======================================================================
// Note: This example uses the `paste` crate for identifier concatenation
//=======================================================================
// Add to Cargo.toml: paste = "1.0"

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
                    // _mut suffix for mutable accessor
                    pub fn [<$field _mut>](&mut self) -> &mut $type {
                        &mut self.$field
                    }

                    // set_ prefix for setter
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

Generate repetitive trait implementations automatically.

```rust
//================================================
// Generate From implementations for enum variants
//================================================
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

//=====================================
// Generate From impls for each variant
//=====================================
impl_from!(i64 => Value, Integer);
impl_from!(f64 => Value, Float);
impl_from!(String => Value, String);
impl_from!(bool => Value, Bool);

fn trait_impl_example() {
    let int_value: Value = 42i64.into();
    let string_value: Value = "hello".to_string().into();
    // Now you can use .into() to convert to Value
}
```

### Test Generation

Generate test cases from a compact specification.

```rust
//=================================================
// Generate multiple test functions from a template
//=================================================
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

//==========================
// Generate 3 test functions
//==========================
generate_tests! {
    add {
        test_add_positive: (2, 3) => 5,
        test_add_negative: (-2, -3) => -5,
        test_add_zero: (0, 5) => 5,
    }
}
//============
// Expands to:
//============
// #[test] fn test_add_positive() { assert_eq!(add(2, 3), 5); }
//================================================================
// #[test] fn test_add_negative() { assert_eq!(add(-2, -3), -5); }
//================================================================
// #[test] fn test_add_zero() { assert_eq!(add(0, 5), 5); }
```

### Bitflags Pattern

Generate bitflag types with operations (similar to the `bitflags` crate).

```rust
//===================================
// Simplified bitflags implementation
//===================================
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

        // Implement | operator for combining flags
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

---

## Macro Debugging

Debugging macros is challenging because you can't easily see the generated code. These tools and patterns help.

### Using cargo expand

`cargo expand` shows the fully expanded macro code—essential for understanding what your macros generate.

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

Force a compile error that shows the macro input—useful for understanding what the macro receives.

```rust
//====================================================
// Debug macro by showing its input as a compile error
//====================================================
macro_rules! debug_macro {
    ($($tt:tt)*) => {
        compile_error!(concat!("Macro input: ", stringify!($($tt)*)));
    };
}

//===============================================
// This will show the exact input at compile time
//===============================================
// debug_macro!(some input here);
//==============================================
// Compile error: "Macro input: some input here"
//==============================================
```

### Tracing Macro Expansion

Print macro inputs at runtime to trace execution.

```rust
//===================================
// Trace macro invocations at runtime
//===================================
macro_rules! trace {
    ($($arg:tt)*) => {
        {
            eprintln!("Macro trace: {}", stringify!($($arg)*));
            $($arg)*  // Still execute the code
        }
    };
}

fn tracing_example() {
    let x = trace!(5 + 3);
    println!("Result: {}", x);
    // Stderr: "Macro trace: 5 + 3"
    // Stdout: "Result: 8"
}
```

### Common Debugging Patterns

```rust
//==============================================
// 1. Echo pattern - see what the macro receives
//==============================================
macro_rules! echo {
    ($($tt:tt)*) => {
        {
            println!("Macro received: {}", stringify!($($tt)*));
            $($tt)*
        }
    };
}

//======================
// 2. Type introspection
//======================
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

//=====================
// 3. Count token trees
//=====================
macro_rules! count_tts {
    () => { 0 };
    ($odd:tt $($rest:tt)*) => { 1 + count_tts!($($rest)*) };
}

fn debugging_patterns() {
    echo!(println!("Hello"));

    let x = show_type!(vec![1, 2, 3]);
    // Prints: "Type of vec![1, 2, 3]: alloc::vec::Vec<i32>"

    const COUNT: usize = count_tts!(a b c d e);
    println!("Token count: {}", COUNT);  // 5
}
```

### Error Messages with Custom Diagnostics

Provide helpful error messages when macro invocation is invalid.

```rust
//================================================
// Validate input and provide clear error messages
//================================================
macro_rules! validate_input {
    (empty) => {
        compile_error!("Input cannot be empty!");
    };
    ($($valid:tt)*) => {
        // Process valid input
        $($valid)*
    };
}

//======================
// Better error messages
//======================
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

---

This comprehensive guide covers all essential patterns for declarative macros in Rust. Macros are powerful but can be complex—use them when the alternative is worse (repetitive boilerplate, impossible abstractions), and prefer simpler solutions (functions, traits) when possible. The patterns here—from basic repetitions to full DSLs—give you the tools to harness macro power effectively.
