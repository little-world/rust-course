# Declarative Macros
This chapter covers declarative macros (macro_rules!); pattern matching on syntax to generate code at compile-time. Macros enable variadic arguments, DSLs, and zero-cost abstractions impossible with functions. Pattern match input syntax, expand to template code.



## Pattern 1: Macro Patterns and Repetition

**Problem**: Functions have fixed signatures—can't accept variable number of arguments. println!("{} {}", a, b, c) needs different function for each arg count.

**Solution**: Use macro_rules! to pattern-match syntax and generate code.

**Why It Matters**: Zero-cost abstraction—macro expansion compiles to optimal code, no runtime overhead. Variadic macros without variadics—vec![1, 2, 3] expands to optimal Vec construction.

**Use Cases**: Collection literals (vec!, hashmap!), variadic functions (println!, format!, write!), testing macros (assert_eq!, assert!), DSL construction (sql!, html!), builders (setters for all fields), trait implementations (for all numeric types), derive-like custom macros.

### Example: Basic Pattern Matching

Create macros that accept different syntax patterns.

**The simplest patterns:**
- `()` matches empty invocation: `my_macro!()`
- `($name:expr)` matches an expression and binds it to `$name`
- Multiple patterns create different "overloads" for different invocation styles

### Example: Simple macro without arguments
The empty `()` pattern matches `say_hello!()` with no arguments; the body after `=>` is the generated code.
Invocations are replaced by expanded code at compile time with zero runtime overhead.

```rust
macro_rules! say_hello {
    () => {
        println!("Hello, World!");
    };
}
say_hello!(); // Prints "Hello, World!"
```

### Example: Macro with a single argument
The `$name:expr` syntax captures any Rust expression; inside the body, `$name` expands to the passed value.
The `:expr` fragment specifier tells the parser what syntax to expect, enabling type-safe code generation.

```rust
// $name:expr means "match any expression, call it $name"
macro_rules! greet {
    ($name:expr) => {
        println!("Hello, {}!", $name);
    };
}
greet!("Alice"); // Prints "Hello, Alice!"
```

### Example: Macro with multiple patterns
Multiple patterns separated by semicolons create overloaded behavior; patterns are tried in order until one matches.
Literal tokens like `add`, `sub`, `mul` must appear exactly as written, creating a mini DSL syntax.

```rust
macro_rules! calculate {
    (add $a:expr, $b:expr) => { $a + $b };
    (sub $a:expr, $b:expr) => { $a - $b };
    (mul $a:expr, $b:expr) => { $a * $b };
}

fn basic_examples() {
    say_hello!();
    greet!("Alice");
    let sum = calculate!(add 5, 3);     // → 5 + 3
    let product = calculate!(mul 4, 7); // → 4 * 7
    println!("Sum: {}, Product: {}", sum, product);
}
basic_examples(); // Demonstrates say_hello!, greet!, calculate! macros
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
macro_rules! fragment_examples {
    ($e:expr) => { println!("Expression: {}", $e); };         // expr: 5+3, vec![1,2]
    ($i:ident) => { let $i = 42; };                           // ident: x, my_var
    ($t:ty) => { std::mem::size_of::<$t>() };                // ty: i32, Vec<String>
    ($p:pat) => { match Some(42) { $p => println!("Matched!"), _ => () } }; // pat
    ($s:stmt) => { $s };                                      // stmt: let x = 5;
    ($b:block) => { $b };                                     // block: { ... }
    ($it:item) => { $it };                                    // item: fn, struct
    ($m:meta) => { #[$m] fn dummy() {} };                     // meta: derive(Debug)
    ($tt:tt) => { stringify!($tt) };                          // tt: any token
}

fn fragment_usage() {
    fragment_examples!(5 + 3);  // expr: prints "Expression: 8"
    fragment_examples!(x);  // ident: creates variable x = 42

    let size = fragment_examples!(i32);  // ty: returns 4 (size of i32)
    println!("Size of i32: {}", size);
}
fragment_usage(); // Shows expr, ident, ty fragment specifiers in action
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

### Example: Basic repetition with *
The `$(...)*` syntax matches zero or more comma-separated occurrences; in the expansion, code repeats for each element.
This is exactly how the standard library's `vec!` macro works internally.

```rust
macro_rules! create_vec {
    ($($elem:expr),*) => {{  // Match comma-separated exprs
        let mut v = Vec::new();
        $( v.push($elem); )*  // Repeat for each
        v
    }};
}
let v = create_vec![1, 2, 3]; // Creates Vec containing [1, 2, 3]
```

### Example: Repetition with + (one or more)
The `+` repetition requires at least one match, unlike `*` which accepts zero; `sum!()` becomes a compile error.
Choose `+` when an empty invocation doesn't make semantic sense, providing better error messages.

```rust
macro_rules! sum {
    ($($num:expr),+) => {{ // + requires at least one
        let mut total = 0;
        $( total += $num; )+
        total
    }};
}
let total = sum!(1, 2, 3, 4); // Returns 10
```

### Example: Optional repetition with ?
The `?` repetition matches zero or one occurrence; here `$(, $default:expr)?` allows an optional second argument.
In the expansion, `.or(Some($default))` only appears if provided, enabling different argument counts.

```rust
// Allows an optional second argument
macro_rules! optional_value {
    ($val:expr $(, $default:expr)?) => {
        Some($val) $(.or(Some($default)))?
    };
}
let v = optional_value!(42); // Some(42)
let v = optional_value!(42, 100); // Some(42).or(Some(100))
```

### Example: Multiple repetitions
Multiple metavariables in one repetition group like `$key:expr => $val:expr` capture key-value pairs together.
The `$(,)?` allows optional trailing commas; this pattern is the foundation for HashMap literal macros.

```rust
macro_rules! hash_map {
    ($($key:expr => $val:expr),* $(,)?) => {{ // $(,)? allows trailing comma
        let mut map = std::collections::HashMap::new();
        $( map.insert($key, $val); )*
        map
    }};
}

fn repetition_examples() {
    let v = create_vec![1, 2, 3, 4, 5];
    let total = sum!(1, 2, 3, 4, 5);
    let map = hash_map! { "name" => "Alice", "role" => "Developer", }; // Trailing comma ok
    println!("Vec: {:?}, Sum: {}, Map: {:?}", v, total, map);
}
repetition_examples(); // Demonstrates *, +, and HashMap macro patterns
```


### Example: Nested Repetitions

Nested repetitions allow matching multi-dimensional structures like matrices or tables.

**When to use nested repetitions:**
- Generating multi-dimensional data structures
- Processing tables or grids
- Creating multiple related items with shared structure

```rust
macro_rules! matrix {
    ($([$($elem:expr),*]),* $(,)?) => { // Outer: rows, Inner: elements
        vec![$( vec![$($elem),*] ),*]
    };
}
let m = matrix![[1,2,3], [4,5,6]]; // Creates Vec<Vec<i32>>
```

### Example: Multiple types of repetitions
Nested repetitions use multiple `$(...)*` patterns—outer for functions, inner for parameters with matching structure.
This technique generates multiple complete function definitions from a single macro invocation.

```rust
macro_rules! function_table {
    { $( fn $name:ident($($param:ident: $type:ty),*) -> $ret:ty $body:block )* } => {
        $( fn $name($($param: $type),*) -> $ret $body )*
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
nested_examples(); // Creates 3x3 matrix and generates add/multiply functions
```


### Counting and Indexing

Counting elements in a macro requires recursive expansion—declarative macros don't have loops or counters.

**The counting trick:**
Use recursion to add 1 for each element until you hit the base case (empty).

```rust
// count!(a b c) → 1 + 1 + 1 + 0 = 3
macro_rules! count {
    () => (0);
    ($head:tt $($tail:tt)*) => (1 + count!($($tail)*));
}
const N: usize = count!(a b c d e); // N = 5 at compile time
```

### Example: Generate indexed names
The `ident` fragment captures identifiers that become field names; each `$name` creates a separate `i32` field.
This pattern generates structs with user-specified field names, fully type-checked after expansion.

```rust
macro_rules! create_fields {
    ($($name:ident),*) => {
        struct GeneratedStruct { $( $name: i32, )* }
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
    let count = count!(a b c d e);  // 5 at compile time
    let tuple = (1, "hello", 3.14, true);
    println!("Count: {}, First: {}", count, tuple_access!(tuple, 0));
}
counting_examples(); // Demonstrates compile-time counting and tuple access
```


### Example: Pattern Matching with Guards

Pattern matching in macros can match specific literals or any syntax, giving you fine control over what invocations are valid.

**Pattern matching strategies:**
- Match specific literals to create keyword-based DSLs
- Match general patterns with fragment specifiers
- Combine both for flexible yet constrained syntax

### Example: Match specific literals
Literal patterns like `(true)` and `(false)` match exact tokens, not expressions evaluating to those values.
Specific literals should come before general `$other:expr` catch-alls since patterns are tested in order.

```rust
macro_rules! match_literal {
    (true) => { "It's true!" };
    (false) => { "It's false!" };
    ($other:expr) => { "It's something else" };
}
match_literal!(true); // Returns "It's true!"
```

### Example: Match different types of expressions
The `stringify!` macro converts code to a string literal at compile time, invaluable for debugging.
Double braces `{{ }}` create a block expression, letting you add logging around any expression.

```rust
// stringify! turns code into a string literal
macro_rules! describe_expr {
    ($e:expr) => {{
        println!("Expression: {}", stringify!($e));
        $e
    }};
}
let x = describe_expr!(2 + 2); // Prints "Expression: 2 + 2", returns 4
```

### Example: Complex pattern matching
Operators like `+`, `-`, `*`, `/` are matched as literal tokens, creating natural syntax: `operation!(10, +, 5)`.
Each pattern generates different code; this is the foundation for expression DSLs and calculator languages.

```rust
macro_rules! operation {
    ($a:expr, +, $b:expr) => { $a + $b };
    ($a:expr, -, $b:expr) => { $a - $b };
    ($a:expr, *, $b:expr) => { $a * $b };
    ($a:expr, /, $b:expr) => { $a / $b };
}

fn pattern_matching_examples() {
    println!("{}", match_literal!(true));  // "It's true!"
    println!("{}", match_literal!(42));    // "It's something else"

    // operation! creates a mini calculator language
    let result = operation!(10, +, 5);
    println!("Result: {}", result);  // 15
}
pattern_matching_examples(); // Shows literal matching and operator DSL
```



## Pattern 2: Hygiene and Scoping

**Problem**: Macro-generated variables collide with caller's variables—C's #define SWAP uses temp, but caller has temp variable, conflict! Macro introduces $x but user has x—which wins?

**Solution**: Rust macros are hygienic—compiler renames macro-generated variables to avoid collisions. Variables from macro and caller exist in different "syntax contexts".

**Why It Matters**: Prevents subtle name collision bugs—user's x won't conflict with macro's internal x. Makes macros composable: nested macro calls work without name clashes.

**Use Cases**: All macros (hygiene is default behavior), library macros using $crate for paths, nested macro invocations, macros generating helper functions/structs, temporary variables in macros, composable macro systems.

### Example: Hygienic Variables Pattern

Generate temporary variables without colliding with user code.

### Example: Macro-generated variables are hygienic
Rust's macro hygiene means variables inside macros exist in a separate "syntax context"—no collisions with outer scope.
The compiler renames macro-generated identifiers internally, a major improvement over C's `#define` bugs.

```rust
macro_rules! hygienic_example {
    () => {
        let x = 42; // Separate from outer x
        println!("Inner x: {}", x);
    };
}

fn hygiene_test() {
    let x = 100;
    hygienic_example!();  // Inner x: 42
    println!("Outer x: {}", x);  // Still 100
}
hygiene_test(); // Demonstrates macro hygiene - inner x doesn't shadow outer x
```


### Example: Breaking Hygiene with $name:ident

Sometimes you *want* to create or modify variables in the caller's scope. Use `ident` fragment specifiers to intentionally break hygiene.

**When to break hygiene:**
- DSLs that create variables for the user
- Macros that modify existing variables
- Code generation patterns where the user expects side effects

### Example: Intentionally capture variables from caller's scope
The `ident` fragment passes identifiers without hygiene protection, creating variables in the caller's scope.
Use this pattern for DSLs that need to define variables, like `let_mut!` or test setup macros.

```rust
macro_rules! set_value {
    ($var:ident = $val:expr) => { let $var = $val; }; // In caller's scope
}

macro_rules! increment {
    ($var:ident) => { $var += 1; }; // Modifies caller's var
}

fn breaking_hygiene() {
    set_value!(counter = 0);
    increment!(counter);
    println!("Counter: {}", counter); // 1
}
breaking_hygiene(); // Creates and modifies 'counter' in caller's scope
```


### Example: Macro Scope and Ordering

Unlike functions, macros must be defined *before* they're used. Macros are expanded in a single pass through the file.

**Scoping rules:**
- Macros are visible from their definition point onward
- Macros can call other macros (including themselves recursively)
- `#[macro_export]` makes a macro available to other crates

### Example: Macros must be defined before use
Macros are expanded during parsing in a single pass—they must be defined textually before invocation.
This is why macros are placed at the top of files; use `#[macro_use]` to import from other modules.

```rust
macro_rules! early_macro {
    () => {
        println!("Defined early");
    };
}

fn can_use_early() {
    early_macro!(); // Works - macro defined above
}

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


### Example: Module Visibility

Macros have special visibility rules compared to other items.

**Key differences:**
- Macros don't respect privacy boundaries by default
- `#[macro_export]` exports to the crate root, not the current module
- Importing macros requires special syntax in older Rust editions

### Example: Macros can be exported from modules
The `#[macro_export]` attribute makes a macro available at the crate root, not the defining module.
Non-exported macros are private to their module; functions can use private macros for encapsulation.

```rust
mod macros {
    #[macro_export]  // Available at crate root
    macro_rules! public_macro {
        () => { println!("Public macro"); };
    }

    macro_rules! private_macro {  // Private to module
        () => { println!("Private macro"); };
    }

    pub fn use_private() { private_macro!(); }
}

```

### Example: Can use public_macro anywhere in the crate
Exported macros work anywhere without path qualification; private macros error outside their module.
Wrap private macro usage in public functions to expose functionality without exposing the macro itself.

```rust
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
macro_rules! with_context {
    ($name:ident, $body:block) => {{
        let context = "macro context";
        let $name = context;  // Bind to user's name
        $body
    }};
}

fn context_example() {
    with_context!(ctx, {
        println!("Context: {}", ctx);  // ctx is provided by the macro
    });
}
context_example(); // Macro provides 'ctx' variable to the block
```


## Pattern 3: DSL Construction

**Problem**: Domain-specific code in Rust verbose—writing SQL queries as strings loses compile-time checking. HTML templates as strings have no type safety.

**Solution**: Build DSLs with macros that parse custom syntax at compile-time. sql!

**Why It Matters**: Compile-time type safety—SQL column typos become compile errors, not runtime. Domain code readable: select!(user.name from users where |u| u.active) vs manual iterator chains.

**Use Cases**: SQL query builders (type-safe at compile-time), HTML templates (yew, maud), test DSLs (assert_matches!, mock!), configuration DSLs, state machine definitions, parser combinators, JSON builders, regex DSLs, markup languages.

### Example: SQL-Style DSL Pattern

Create readable query syntax that compiles to efficient iterator code.

### Example: SQL-like query syntax compiled to iterator chains
Literal tokens `from` and `where` create SQL-like keywords; `$($field:ident),+` captures SELECT fields.
The expansion generates `filter()` for WHERE and `map()` for SELECT—efficient, type-checked iterator code.

```rust
macro_rules! select {
    ($($field:ident),+ from $table:ident where $condition:expr) => {{
        $table.iter()
            .filter(|row| $condition(row))    // WHERE
            .map(|row| ($(row.$field,)+))     // SELECT
            .collect::<Vec<_>>()
    }};
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
    println!("Results: {:?}", results); // [("Alice", 30), ("Carol", 35)]
}
sql_dsl_example(); // SQL-like syntax: select!(field from table where cond)
```


### Example: Configuration DSL

Create a structured configuration syntax that parses at compile time.

```rust
macro_rules! config {
    { $( $section:ident { $( $key:ident: $value:expr ),* $(,)? } )* } => {{
        use std::collections::HashMap;
        let mut config = HashMap::new();
        $({
            let mut section = HashMap::new();
            $( section.insert(stringify!($key), $value.to_string()); )*
            config.insert(stringify!($section), section);
        })*
        config
    }};
}

fn config_dsl_example() {
    let settings = config! {
        database { host: "localhost", port: 5432, name: "mydb", }
        server { host: "0.0.0.0", port: 8080, workers: 4, }
    };
    println!("Config: {:?}", settings);
}
config_dsl_example(); // Nested config syntax → HashMap<&str, HashMap<&str, String>>
```


### Example: HTML-like DSL

Generate HTML strings with XML-like syntax (simplified version of real templating engines).

### Example: HTML-like DSL (simplified)
Multiple patterns handle self-closing tags, tags with content, and text; `<$tag:ident>` matches `<div>` or `<p>`.
Recursive calls process nested elements; `$($element:tt)*` handles sibling elements concatenated together.

```rust
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
html_dsl_example(); // XML-like syntax compiles to HTML string
```


### Example: State Machine DSL

Define state machines declaratively, compiling to efficient match-based transitions.

```rust
// State machine DSL
// states: [State1, State2]
// transitions: { State1 -> State2 on Event }
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
state_machine_example(); // Declarative state machine with type-safe transitions
```


---

## Pattern 4: Code Generation Patterns

**Problem**: Implementing trait for all numeric types (u8, u16, u32, u64, u128, i8, ...) is 10+ identical impls. Tuple trait impls for (T1), (T1, T2), ..., (T1...T12) exponential boilerplate.

**Solution**: Macros generate impl blocks via repetition. Define trait impl template, list types, macro generates all.

**Why It Matters**: DRY—define once, generate many. Adding u256 type?

**Use Cases**: Trait impls for primitives (all numeric types), tuple trait impls (arity 1-12), enum From/Into/Display, struct getters/setters/builders, newtype pattern automation, format string wrappers, test case generation.

### Example: Trait Implementation Generation

Implement same trait for many types without copy-paste.

### Example: Note: This example uses the `paste` crate for identifier concatenation
The `accessors!` macro generates struct definition plus getter/setter methods; `paste::paste!` enables identifier manipulation.
For each field, three methods are generated: getter, mutable getter (`_mut`), and setter (`set_`).

```rust
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
accessor_example(); // Auto-generated getters/setters via accessors! macro
```



### Example: Generate From implementations for enum variants
This macro generates `From<$from> for $to` trait implementations; `$variant:ident` captures the enum variant.
Each invocation expands to a complete impl block—without it, you'd write identical code for every conversion.

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
impl_from!(i64 => Value, Integer); // Generates From<i64> for Value
```

### Example: Generate From impls for each variant
Four invocations generate four `From` impls; after expansion `42i64.into()` creates `Value::Integer(42)`.
Adding a new variant requires one `impl_from!` line instead of 10+ lines of boilerplate.

```rust
impl_from!(i64 => Value, Integer);
impl_from!(f64 => Value, Float);
impl_from!(String => Value, String);
impl_from!(bool => Value, Bool);

fn trait_impl_example() {
    let int_value: Value = 42i64.into();
    let string_value: Value = "hello".to_string().into();
    // Now you can use .into() to convert to Value
}
trait_impl_example(); // Uses generated From impls: 42i64.into() → Value::Integer
```


### Example: Test Generation

Generate test cases from a compact specification.

### Example: Generate multiple test functions from a template
This macro captures a function name and test cases; `$test_name: ($inputs) => $expected` matches each spec.
The outer pattern groups tests for one function; `$(,)?` allows optional trailing commas.

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

```

### Example: Generate 3 test functions
Three lines of test specification expand into three complete `#[test]` functions with assert_eq! calls.
Without this macro, you'd write 15+ lines of repetitive boilerplate—widely used to reduce test code.

```rust
generate_tests! {
    add {
        test_add_positive: (2, 3) => 5,
        test_add_negative: (-2, -3) => -5,
        test_add_zero: (0, 5) => 5,
    }
}
// Expands to 3 #[test] functions with assert_eq! calls
```


### Example: Bitflags Pattern

Generate bitflag types with operations (similar to the `bitflags` crate).

### Example: Simplified bitflags implementation
This macro captures attributes, visibility, name, backing type, and flags; `$(#[$attr:meta])*` preserves doc comments.
Each `const $flag` becomes an associated constant; `impl BitOr` enables `|` operator syntax.

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
bitflags_example(); // Combine flags with |, check with .contains()
```


---

## Pattern 5: Macro Debugging

**Problem**: Macro errors cryptic—"no rules expected token `ident`" doesn't show which rule failed. Expansion invisible—can't see generated code.

**Solution**: Use cargo expand to view full expansion (cargo install cargo-expand, then cargo expand). trace_macros!(true) logs which rules matched.

**Why It Matters**: Debug 10x faster by seeing actual generated code. cargo expand reveals what macro produces—often obvious bugs.

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

### Example: Debug macro by showing its input as a compile error
The `$($tt:tt)*` pattern captures all input as token trees; `compile_error!` with `stringify!` shows them at compile time.
This technique reveals mismatches between what you intended and what the macro actually captured.

```rust
macro_rules! debug_macro {
    ($($tt:tt)*) => {
        compile_error!(concat!("Macro input: ", stringify!($($tt)*)));
    };
}

// debug_macro!(some input here);
// → Compile error: "Macro input: some input here"
```


### Example: Tracing Macro Expansion

Print macro inputs at runtime to trace execution.

### Example: Trace macro invocations at runtime
Unlike `compile_error!`, this logs input at runtime via `eprintln!` to stderr while still executing the code.
The block wraps everything so the macro returns the expression's value—debug without stopping compilation.

```rust
macro_rules! trace {
    ($($arg:tt)*) => {{
        eprintln!("Macro trace: {}", stringify!($($arg)*));
        $($arg)*
    }};
}

fn tracing_example() {
    let x = trace!(5 + 3); // Logs "5 + 3" to stderr, returns 8
    println!("Result: {}", x);
}
tracing_example(); // Logs macro input to stderr, then executes
```


### Example: 1. Echo pattern - see what the macro receives
The echo pattern prints exactly what tokens the macro receives via `stringify!`, then executes them unchanged.
Using `$($tt:tt)*` captures any valid Rust syntax—the simplest debugging macro for any code.

```rust
macro_rules! echo {
    ($($tt:tt)*) => {{ println!("Macro received: {}", stringify!($($tt)*)); $($tt)* }};
}
echo!(let x = 5); // Prints input, then executes it
```

### Example: 2. Type introspection
This macro reveals the concrete type at runtime using `std::any::type_name` with generic type inference.
The macro stores, prints the type, and returns the value—usable anywhere an expression is expected.

```rust
macro_rules! show_type {
    ($expr:expr) => {{
        fn type_of<T>(_: &T) -> &'static str { std::any::type_name::<T>() }
        let value = $expr;
        println!("Type of {}: {}", stringify!($expr), type_of(&value));
        value
    }};
}
let v = show_type!(vec![1,2,3]); // Prints type, returns value
```

### Example: 3. Count token trees
This recursive macro peels off one `$odd:tt` at a time, adding 1 and recursing; base case `() => { 0 }`.
The count evaluates at compile time as a `const` expression: `count_tts!(a b c d e)` becomes 5.

```rust
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
debugging_patterns(); // Demonstrates echo, show_type, count_tts macros
```


### Example: Error Messages with Custom Diagnostics

Provide helpful error messages when macro invocation is invalid.

### Example: Validate input and provide clear error messages
Pattern order matters—the first match wins; put specific cases like `(empty)` before general catch-alls.
This creates "guard clauses" that reject invalid input with helpful compile-time error messages.

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

```

### Example: Better error messages
The `$lit:literal` pattern only matches compile-time literals; other expressions fall through.
The catch-all uses `compile_error!` with `stringify!` to show clear "expected literal, got X" messages.

```rust
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
