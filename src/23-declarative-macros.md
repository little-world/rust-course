# Declarative Macros

[Macro Patterns and Repetition](#pattern-1-macro-patterns-and-repetition)

- Problem: Functions have fixed signatures; can't accept variable arguments; println! needs N args; vec![1,2,3] variable length; boilerplate for every type
- Solution: macro_rules! with pattern matching; $(...)* for repetition; fragment specifiers (expr, ident, ty); match syntax, expand to code
- Why It Matters: Zero-cost abstraction—compiles to optimal code; variadic without runtime overhead; reduces boilerplate 10x; DRY principle at compile time
- Use Cases: vec!/hashmap! (collections), println!/format! (variadics), assert_eq! (testing), builders, DSLs, derive-like custom macros

[Hygiene and Scoping](#pattern-2-hygiene-and-scoping)

- Problem: Macro-generated variables collide with caller's variables; $x shadows user's x; unhygienic macros break; need fresh identifiers
- Solution: Hygienic macros—compiler renames macro vars to avoid collisions; $crate for absolute paths; $crate::module works across crates
- Why It Matters: Prevents subtle bugs—user's x won't conflict with macro's x; macros composable (no name clashes); $crate enables library macros
- Use Cases: All macros (hygiene default), library macros ($crate for paths), nested macro calls, macro-generated structs/functions

[DSL Construction](#pattern-3-dsl-construction)

- Problem: Want Rust-like syntax for domain logic; SQL/HTML in strings error-prone; type-safe query builders verbose; domain code unreadable
- Solution: Macros parse custom syntax at compile-time; sql! macro for type-safe queries; html! for templates; match complex patterns
- Why It Matters: Type safety at compile-time (SQL typos → compile error); readable domain code; zero runtime overhead vs string parsing
- Use Cases: SQL builders (compile-time checked), HTML templates, test DSLs, config DSLs, state machines, parser combinators

[Code Generation Patterns](#pattern-4-code-generation-patterns)

- Problem: Implementing trait for 50 types is tedious; tuple impls for (T1), (T1,T2), ...; enum boilerplate; getter/setter repetition
- Solution: Macros generate impl blocks; repeat patterns for tuples; auto-generate From/Into; builder pattern automation
- Why It Matters: DRY—define once, generate many; adding type doesn't need 50 manual impls; reduces human error (forgot impl for u128)
- Use Cases: Trait impls for primitives, tuple trait impls (1-12 elements), enum helpers (from_str, to_string), builders, newtype patterns

[Macro Debugging](#pattern-5-macro-debugging)

- Problem: Macro errors cryptic—"no rules expected token"; expansion invisible; hygiene confusing; recursion limits hit
- Solution: cargo expand shows expansion; trace_macros!(true) logs matching; rust-analyzer inline expansion; #[macro_export] for visibility
- Why It Matters: Debug faster—see actual generated code; understand errors (token in wrong place); iterate on macro design
- Use Cases: All macro development, debugging expansion errors, understanding library macros, teaching, code review

[Macro Cheat Sheet](#macro-cheat-sheet)
- common patterns for declarative macros

### Overview
This chapter covers declarative macros (macro_rules!)—pattern matching on syntax to generate code at compile-time. Macros enable variadic arguments, DSLs, and zero-cost abstractions impossible with functions. Pattern match input syntax, expand to template code.



## Pattern 1: Macro Patterns and Repetition

**Problem**: Functions have fixed signatures—can't accept variable number of arguments. println!("{} {}", a, b, c) needs different function for each arg count. vec![1, 2, 3] vs vec![1, 2, ..., 1000] requires different code. Implementing trait for u8, u16, u32, ..., u128 is 10x boilerplate. Can't have foo(expr, expr, ...) with N expressions. Type system can't express "any number of arguments of any types".

**Solution**: Use macro_rules! to pattern-match syntax and generate code. $(...)*  for repetition—matches 0+ times, separated by delimiter. Fragment specifiers: $e:expr (expression), $i:ident (identifier), $t:ty (type). Multiple rules for different patterns. Macros expand at compile-time before type checking. Can generate any Rust code: expressions, statements, items (structs, functions).

**Why It Matters**: Zero-cost abstraction—macro expansion compiles to optimal code, no runtime overhead. Variadic macros without variadics—vec![1, 2, 3] expands to optimal Vec construction. Reduces boilerplate 10x—one macro replaces 10 implementations. DRY principle at compile-time. Essential for vec!, println!, assert_eq!, custom collection literals. Without macros, would need either C-style variadics (unsafe) or builder pattern (verbose).

**Use Cases**: Collection literals (vec!, hashmap!), variadic functions (println!, format!, write!), testing macros (assert_eq!, assert!), DSL construction (sql!, html!), builders (setters for all fields), trait implementations (for all numeric types), derive-like custom macros.

### Example: Basic Pattern Matching

Create macros that accept different syntax patterns.

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

### Example: Fragment Specifiers

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

### Example: Repetition Patterns

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

### Example: Nested Repetitions

Nested repetitions allow matching multi-dimensional structures like matrices or tables.

**When to use nested repetitions:**
- Generating multi-dimensional data structures
- Processing tables or grids
- Creating multiple related items with shared structure

```rust
//========================================
// Matrix creation with nested repetitions
// Outer repetition: rows
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
// How it works:
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
// Manually provides accessors for tuple elements
//=======================
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

### Example: Pattern Matching with Guards

Pattern matching in macros can match specific literals or any syntax, giving you fine control over what invocations are valid.

**Pattern matching strategies:**
- Match specific literals to create keyword-based DSLs
- Match general patterns with fragment specifiers
- Combine both for flexible yet constrained syntax

```rust
//========================
// Match specific literals
//========================
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


## Pattern 2: Hygiene and Scoping

**Problem**: Macro-generated variables collide with caller's variables—C's #define SWAP uses temp, but caller has temp variable, conflict! Macro introduces $x but user has x—which wins? Unhygienic macros have name capture bugs. Need fresh identifiers that won't conflict. Macros in library crates reference other modules—absolute paths break across crates. Without hygiene, composing macros fails.

**Solution**: Rust macros are hygienic—compiler renames macro-generated variables to avoid collisions. Variables from macro and caller exist in different "syntax contexts". Use $crate::module for absolute paths within crate—works even when macro exported. Hygiene automatic for let bindings. Can deliberately break hygiene when needed (macro parameters). Multiple invocations get independent scopes.

**Why It Matters**: Prevents subtle name collision bugs—user's x won't conflict with macro's internal x. Makes macros composable: nested macro calls work without name clashes. Essential for library macros: $crate enables safe cross-crate usage. Without hygiene, macros unreliable—works in testing, breaks in production when user happens to have same var name. C macros notorious for this; Rust fixes it.

**Use Cases**: All macros (hygiene is default behavior), library macros using $crate for paths, nested macro invocations, macros generating helper functions/structs, temporary variables in macros, composable macro systems.

### Example: Hygienic Variables Pattern

Generate temporary variables without colliding with user code.

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

### Example: Breaking Hygiene with $name:ident

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

### Example: Macro Scope and Ordering

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
// late_macro!();  // ERROR: macro not yet defined
//================================================
macro_rules! late_macro {
    () => {
        println!("Defined late");
    };
}

fn can_use_late() {
    late_macro!(); // Works - macro defined above this function
}
```

### Example: Module Visibility

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

### Example: Context Capture

Macros can intentionally capture context to provide convenient DSLs.

**The context pattern:**
Create a scope with predefined variables that the user's code can access.

```rust
//==================================================
// Capture context intentionally
// Provides a 'context' variable to the user's block
//==================================================
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

## Pattern 3: DSL Construction

**Problem**: Domain-specific code in Rust verbose—writing SQL queries as strings loses compile-time checking. HTML templates as strings have no type safety. Handwritten query builders (data.iter().filter(...).map(...)) unreadable for complex queries. Want domain-natural syntax but Rust's semantics. Parsing strings at runtime slow. Test assertion DSLs verbose.

**Solution**: Build DSLs with macros that parse custom syntax at compile-time. sql! macro parses SQL-like syntax, generates type-safe Rust. html! macro parses HTML, validates at compile-time. State machine DSLs. Match complex patterns to extract structure. Macros translate domain syntax to efficient Rust code. Zero runtime parsing overhead.

**Why It Matters**: Compile-time type safety—SQL column typos become compile errors, not runtime. Domain code readable: select!(user.name from users where |u| u.active) vs manual iterator chains. Zero runtime overhead: DSL compiles to optimal Rust, no interpretation. Safer than strings: html! validates structure. Essential for readable domain-heavy code. Testing DSLs (assert_eq!) more ergonomic.

**Use Cases**: SQL query builders (type-safe at compile-time), HTML templates (yew, maud), test DSLs (assert_matches!, mock!), configuration DSLs, state machine definitions, parser combinators, JSON builders, regex DSLs, markup languages.

### Example: SQL-Style DSL Pattern

Create readable query syntax that compiles to efficient iterator code.

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

### Example 2: Configuration DSL

Create a structured configuration syntax that parses at compile time.

```rust
//========================================
// Configuration DSL with nested structure
// section { key: value, key: value }
//========================================
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

### Example: HTML-like DSL

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

### Example 3: State Machine DSL

Define state machines declaratively, compiling to efficient match-based transitions.

```rust
//===========================================
// State machine DSL
// states: [State1, State2]
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

## Pattern 4: Code Generation Patterns

**Problem**: Implementing trait for all numeric types (u8, u16, u32, u64, u128, i8, ...) is 10+ identical impls. Tuple trait impls for (T1), (T1, T2), ..., (T1...T12) exponential boilerplate. Enum From/Into conversions manual for each variant. Getters/setters for 20 struct fields—120 lines of boilerplate. Adding new type means copying impl. Human error: forgot impl for u128.

**Solution**: Macros generate impl blocks via repetition. Define trait impl template, list types, macro generates all. impl_for_primitives!(MyTrait for u8, u16, ...). Generate tuple impls with nested repetitions. Auto-generate enum helpers (from_str, to_string, is_variant). Builder pattern: generate setters from field list. One macro invocation → hundreds of lines of code.

**Why It Matters**: DRY—define once, generate many. Adding u256 type? One entry in macro call, all impls generated. Eliminates human error: can't forget impl. Reduces code review burden—review macro once, not 50 impls. Consistent behavior across types. Essential for libraries (std does this for tuples). Maintainability: change one template, updates all impls.

**Use Cases**: Trait impls for primitives (all numeric types), tuple trait impls (arity 1-12), enum From/Into/Display, struct getters/setters/builders, newtype pattern automation, format string wrappers, test case generation.

### Example: Trait Implementation Generation

Implement same trait for many types without copy-paste.

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

### Example: Trait Implementation Generation

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

### Example: Test Generation

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
//================================================
// Expands to:
// #[test] fn test_add_positive() { assert_eq!(add(2, 3), 5); }
// #[test] fn test_add_negative() { assert_eq!(add(-2, -3), -5); }
// #[test] fn test_add_zero() { assert_eq!(add(0, 5), 5); }
//================================================
```

### Example: Bitflags Pattern

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

## Pattern 5: Macro Debugging

**Problem**: Macro errors cryptic—"no rules expected token `ident`" doesn't show which rule failed. Expansion invisible—can't see generated code. Pattern doesn't match but why? Hygiene confusing: which x is which? Recursion limit hit (default 128). Macro compiles but generates wrong code—how to inspect? Error in expansion points to macro call site, not generated code.

**Solution**: Use cargo expand to view full expansion (cargo install cargo-expand, then cargo expand). trace_macros!(true) logs which rules matched. rust-analyzer shows inline expansion. compile_error! for debug printing during expansion. #[macro_export] makes macros visible. Incremental debugging: simplify macro input until works. Check fragment specifier types (expr vs ty vs ident).

**Why It Matters**: Debug 10x faster by seeing actual generated code. cargo expand reveals what macro produces—often obvious bugs. trace_macros shows pattern matching flow. Without tools, debugging macros like blackbox. Essential for learning: see how vec! expands. Code review: expand to verify correctness. Teaching: show students actual code generated.

**Use Cases**: All macro development (cargo expand essential), debugging "no rules expected" errors, understanding library macros (expand tokio::main!), teaching macros, code review of macro-heavy code, performance analysis (see if macro optimal), verifying hygiene.

### Example: cargo expand Tool

View expanded macro output to understand what code is generated.

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

### Example: Debug Printing with compile_error!

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
// debug_macro!(some input here);
// Compile error: "Macro input: some input here"
//===============================================
```

### Example: Tracing Macro Expansion

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

### Example: Common Debugging Patterns

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

### Example: Error Messages with Custom Diagnostics

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



### Summary

This chapter covered declarative macros (macro_rules!):

1. **Macro Patterns and Repetition**: Pattern matching syntax, $(...)* for repetition, fragment specifiers, variadic arguments
2. **Hygiene and Scoping**: Automatic variable renaming prevents collisions, $crate for cross-crate paths
3. **DSL Construction**: Custom syntax for domain logic, compile-time validation, zero runtime overhead
4. **Code Generation Patterns**: Generate trait impls, tuple impls, builders, eliminate boilerplate
5. **Macro Debugging**: cargo expand, trace_macros!, rust-analyzer, compile_error! debugging

**Key Takeaways**:
- Macros operate on syntax at compile-time—generate code before type checking
- Zero-cost abstraction: macro expansion compiles to optimal code
- Variadic macros via $(...)* repetition—vec![1, 2, 3, ..., N]
- Hygiene prevents name collisions automatically
- DSLs provide type safety at compile-time vs runtime string parsing
- Code generation eliminates boilerplate: one macro → 50 impls

**Fragment Specifiers**:
- `expr`: expressions (1 + 2, foo())
- `ident`: identifiers (variable names)
- `ty`: types (u32, Vec<T>)
- `stmt`: statements
- `pat`: patterns (match arms)
- `tt`: token tree (any token)
- `item`: items (fn, struct, impl)

**When to Use Macros**:
- Variadic functions (println!, vec!)
- DSLs with compile-time validation
- Eliminating boilerplate (trait impls for many types)
- Zero-cost abstractions impossible with functions
- Custom syntax that compiles to efficient Rust

**When NOT to Use Macros**:
- Functions work—prefer functions (simpler, better errors)
- Trait system suffices—traits more composable
- Procedural macros better fit—more power, cleaner code
- One-off code—not worth macro complexity

**Debugging Tips**:
- cargo expand to see generated code
- trace_macros!(true) to log pattern matching
- Start simple, add complexity incrementally
- Test macro with various inputs
- compile_error! for debug output

**Common Patterns**:
- Collection literals: vec![1, 2, 3]
- Variadic println!("{} {}", a, b)
- DSLs: sql!(SELECT * FROM users)
- Trait impl generation for primitives
- Builder pattern automation

**Best Practices**:
- Document macro patterns and examples
- Provide clear compile errors
- Use $crate for library macros
- Test edge cases (empty, single, many elements)
- Keep macros simple—complexity hurts maintainability

### Macro Cheat Sheet
```rust
// ===== DECLARATIVE MACROS (macro_rules!) =====

// Basic macro - no arguments
macro_rules! say_hello {
    () => {
        println!("Hello!");
    };
}

say_hello!();                                       // Prints: Hello!

// Macro with single argument
macro_rules! print_value {
    ($val:expr) => {
        println!("Value: {}", $val);
    };
}

print_value!(42);                                   // Prints: Value: 42
print_value!(1 + 2);                                // Prints: Value: 3

// ===== MACRO FRAGMENT SPECIFIERS =====
// expr - expression
macro_rules! eval {
    ($e:expr) => { $e };
}

// ident - identifier (variable/function name)
macro_rules! create_var {
    ($name:ident) => {
        let $name = 42;
    };
}

// ty - type
macro_rules! create_struct {
    ($name:ident, $field_type:ty) => {
        struct $name {
            value: $field_type,
        }
    };
}

// pat - pattern
macro_rules! match_option {
    ($opt:expr, $pattern:pat => $result:expr) => {
        match $opt {
            $pattern => $result,
            _ => panic!("Pattern didn't match"),
        }
    };
}

// stmt - statement
macro_rules! execute {
    ($s:stmt) => {
        $s
    };
}

// block - block of code
macro_rules! run_block {
    ($b:block) => {
        $b
    };
}

// item - item (function, struct, etc.)
macro_rules! create_function {
    ($i:item) => {
        $i
    };
}

// meta - attribute metadata
// path - path (::std::vec::Vec)
// tt - token tree (any valid token)
// literal - literal value

// ===== MULTIPLE PATTERNS =====
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

calculate!(add 5, 3);                               // 8
calculate!(sub 10, 4);                              // 6
calculate!(mul 2, 7);                               // 14

// ===== REPETITION =====
// Zero or more repetitions: $(...)*
macro_rules! vec_of_strings {
    ($($x:expr),*) => {
        {
            let mut temp_vec = Vec::new();
            $(
                temp_vec.push($x.to_string());
            )*
            temp_vec
        }
    };
}

let v = vec_of_strings!("a", "b", "c");            // vec!["a", "b", "c"]

// One or more repetitions: $(...)+ 
macro_rules! sum {
    ($($x:expr),+) => {
        {
            let mut total = 0;
            $(
                total += $x;
            )+
            total
        }
    };
}

sum!(1, 2, 3, 4);                                   // 10

// Zero or one repetition: $(...)?
macro_rules! optional_print {
    ($val:expr $(, $msg:expr)?) => {
        print!("{}", $val);
        $(
            print!(" - {}", $msg);
        )?
        println!();
    };
}

optional_print!(42);                                // "42"
optional_print!(42, "answer");                      // "42 - answer"

// ===== NESTED REPETITIONS =====
macro_rules! matrix {
    ($([$($x:expr),*]),*) => {
        vec![
            $(
                vec![$($x),*],
            )*
        ]
    };
}

let m = matrix!([1, 2], [3, 4], [5, 6]);           // [[1,2], [3,4], [5,6]]

// ===== RECURSIVE MACROS =====
// Count elements
macro_rules! count {
    () => { 0 };
    ($head:expr) => { 1 };
    ($head:expr, $($tail:expr),+) => {
        1 + count!($($tail),+)
    };
}

count!(1, 2, 3, 4);                                // 4

// Reverse list (compile-time)
macro_rules! reverse {
    ([] $($rev:expr),*) => {
        [$($rev),*]
    };
    ([$head:expr $(, $tail:expr)*] $($rev:expr),*) => {
        reverse!([$($tail),*] $head $(, $rev)*)
    };
}

reverse!([1, 2, 3, 4]);                            // [4, 3, 2, 1]

// ===== MACRO HYGIENE =====
// Macros are hygienic - variables don't leak
macro_rules! declare_var {
    () => {
        let x = 42;
    };
}

declare_var!();
// println!("{}", x);                               // ERROR: x not in scope

// Using $name to expose variable
macro_rules! declare_named {
    ($name:ident) => {
        let $name = 42;
    };
}

declare_named!(my_var);
println!("{}", my_var);                             // OK: 42

// ===== COMMON MACRO PATTERNS =====

// Pattern 1: HashMap creation
macro_rules! hashmap {
    ($($key:expr => $val:expr),* $(,)?) => {
        {
            let mut map = ::std::collections::HashMap::new();
            $(
                map.insert($key, $val);
            )*
            map
        }
    };
}

let map = hashmap! {
    "a" => 1,
    "b" => 2,
    "c" => 3,
};

// Pattern 2: Implement trait for multiple types
macro_rules! impl_trait {
    ($trait_name:ident for $($type:ty),+) => {
        $(
            impl $trait_name for $type {
                fn method(&self) {
                    println!("Called on {}", stringify!($type));
                }
            }
        )+
    };
}

trait MyTrait {
    fn method(&self);
}

impl_trait!(MyTrait for i32, f64, String);

// Pattern 3: Generate getter/setter methods
macro_rules! getter_setter {
    ($field:ident: $type:ty) => {
        paste::paste! {
            pub fn [<get_ $field>](&self) -> &$type {
                &self.$field
            }
            
            pub fn [<set_ $field>](&mut self, value: $type) {
                self.$field = value;
            }
        }
    };
}

// Pattern 4: Assert with custom message
macro_rules! assert_custom {
    ($condition:expr, $($arg:tt)*) => {
        if !$condition {
            panic!("Assertion failed: {}", format!($($arg)*));
        }
    };
}

assert_custom!(2 + 2 == 4, "Math is broken!");

// Pattern 5: Logging macro
macro_rules! log {
    ($level:expr, $($arg:tt)*) => {
        println!("[{}] {}", $level, format!($($arg)*));
    };
}

log!("INFO", "User {} logged in", "alice");
log!("ERROR", "Failed to connect");

// Pattern 6: Match with automatic arms
macro_rules! match_enum {
    ($val:expr, $enum:ident :: { $($variant:ident),* }) => {
        match $val {
            $(
                $enum::$variant => stringify!($variant),
            )*
        }
    };
}

// Pattern 7: Lazy static-like macro
macro_rules! lazy_static {
    ($name:ident: $type:ty = $init:expr) => {
        static $name: std::sync::OnceLock<$type> = std::sync::OnceLock::new();
        impl $name {
            fn get() -> &'static $type {
                $name.get_or_init(|| $init)
            }
        }
    };
}

// ===== DEBUGGING MACROS =====
// dbg! - debug print and return value
let x = dbg!(1 + 2);                               // Prints: [src/main.rs:42] 1 + 2 = 3

// stringify! - convert to string literal
let s = stringify!(1 + 2);                         // "1 + 2"

// concat! - concatenate literals
let s = concat!("Hello", " ", "World");            // "Hello World"

// file! - current file name
let file = file!();                                // "src/main.rs"

// line! - current line number
let line = line!();                                // line number

// column! - current column number
let col = column!();

// module_path! - current module path
let path = module_path!();                         // "myapp::mymodule"

// cfg! - check configuration
let is_unix = cfg!(unix);
let is_debug = cfg!(debug_assertions);

// env! - get environment variable at compile time
let user = env!("USER");                           // Compile error if not set

// option_env! - optional environment variable
let user = option_env!("USER");                    // Option<&str>

// include! - include file content as code
// include!("other_file.rs");

// include_str! - include file as string
let content = include_str!("data.txt");

// include_bytes! - include file as byte array
let bytes = include_bytes!("image.png");

// ===== PROCEDURAL MACROS (OVERVIEW) =====
// Note: Requires separate crate with proc-macro = true

// Derive macro example (in proc-macro crate)
/*
use proc_macro::TokenStream;

#[proc_macro_derive(MyTrait)]
pub fn derive_my_trait(input: TokenStream) -> TokenStream {
    // Parse and generate code
}
*/

// Usage:
/*
#[derive(MyTrait)]
struct MyStruct {
    field: i32,
}
*/

// Attribute macro example
/*
#[proc_macro_attribute]
pub fn my_attribute(attr: TokenStream, item: TokenStream) -> TokenStream {
    // Transform the item
}
*/

// Usage:
/*
#[my_attribute]
fn my_function() {
    // ...
}
*/

// Function-like procedural macro
/*
#[proc_macro]
pub fn my_macro(input: TokenStream) -> TokenStream {
    // Generate code
}
*/

// Usage:
/*
my_macro! {
    // input
}
*/

// ===== MACRO EXAMPLES =====

// Example 1: Min/max macro
macro_rules! min {
    ($x:expr) => { $x };
    ($x:expr, $($rest:expr),+) => {
        std::cmp::min($x, min!($($rest),+))
    };
}

let minimum = min!(3, 1, 4, 1, 5);                 // 1

// Example 2: Timing macro
macro_rules! time_it {
    ($label:expr, $block:block) => {
        {
            let start = std::time::Instant::now();
            let result = $block;
            let duration = start.elapsed();
            println!("{} took: {:?}", $label, duration);
            result
        }
    };
}

time_it!("Computation", {
    std::thread::sleep(std::time::Duration::from_millis(100));
    42
});

// Example 3: Repeat expression
macro_rules! repeat_expr {
    ($expr:expr; $count:expr) => {
        {
            let mut v = Vec::new();
            for _ in 0..$count {
                v.push($expr);
            }
            v
        }
    };
}

let v = repeat_expr!(rand::random::<i32>(); 5);   // 5 random numbers

// Example 4: Unwrap or return
macro_rules! try_or_return {
    ($expr:expr) => {
        match $expr {
            Some(val) => val,
            None => return None,
        }
    };
}

fn example(opt: Option<i32>) -> Option<i32> {
    let value = try_or_return!(opt);
    Some(value * 2)
}

// Example 5: Debug print with context
macro_rules! debug {
    ($val:expr) => {
        println!("[{}:{}] {} = {:?}", 
            file!(), line!(), stringify!($val), $val);
    };
}

let x = 42;
debug!(x);                                          // [main.rs:123] x = 42

// Example 6: Builder pattern
macro_rules! builder {
    ($name:ident { $($field:ident: $type:ty),* }) => {
        struct $name {
            $($field: $type),*
        }
        
        paste::paste! {
            impl $name {
                $(
                    fn [<with_ $field>](mut self, $field: $type) -> Self {
                        self.$field = $field;
                        self
                    }
                )*
            }
        }
    };
}

// Example 7: Test generation
macro_rules! test_cases {
    ($fn_name:ident: $($input:expr => $expected:expr),+ $(,)?) => {
        $(
            #[test]
            fn [<test_ $fn_name _ $input>]() {
                assert_eq!($fn_name($input), $expected);
            }
        )+
    };
}

// Example 8: Bitflags-like macro
macro_rules! bitflags {
    ($name:ident: $type:ty { $($flag:ident = $val:expr),* $(,)? }) => {
        struct $name($type);
        
        impl $name {
            $(
                const $flag: Self = Self($val);
            )*
            
            fn contains(&self, other: Self) -> bool {
                self.0 & other.0 == other.0
            }
        }
    };
}

bitflags! {
    Flags: u32 {
        READ = 0b001,
        WRITE = 0b010,
        EXECUTE = 0b100,
    }
}

// Example 9: Enum with string conversion
macro_rules! enum_str {
    ($name:ident { $($variant:ident),* $(,)? }) => {
        enum $name {
            $($variant),*
        }
        
        impl $name {
            fn as_str(&self) -> &'static str {
                match self {
                    $(Self::$variant => stringify!($variant)),*
                }
            }
        }
    };
}

enum_str! {
    Color {
        Red,
        Green,
        Blue,
    }
}

// Example 10: Compile-time assertions
macro_rules! const_assert {
    ($condition:expr) => {
        const _: () = assert!($condition);
    };
}

const_assert!(std::mem::size_of::<usize>() == 8);  // Compile-time check

// ===== MACRO BEST PRACTICES =====

// 1. Use $(,)? for optional trailing comma
macro_rules! my_vec {
    ($($x:expr),* $(,)?) => {
        vec![$($x),*]
    };
}

// 2. Wrap in block for hygiene
macro_rules! swap {
    ($a:expr, $b:expr) => {
        {
            let temp = $a;
            $a = $b;
            $b = temp;
        }
    };
}

// 3. Use fully qualified paths
macro_rules! new_vec {
    () => {
        ::std::vec::Vec::new()                     // Avoid name conflicts
    };
}

// 4. Document macros
/// Creates a HashMap from key-value pairs
/// 
/// # Examples
/// ```
/// let map = hashmap!("a" => 1, "b" => 2);
/// ```
macro_rules! documented_macro {
    // ...
}

// 5. Use vis for visibility
macro_rules! create_struct {
    ($vis:vis $name:ident) => {
        $vis struct $name;
    };
}

create_struct!(pub MyStruct);                      // Public struct
```