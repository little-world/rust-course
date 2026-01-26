// Pattern 2: Hygiene and Scoping
// Rust macros are hygienic - compiler renames macro-generated variables to avoid collisions

// The 'x' inside the macro is separate from the 'x' outside
macro_rules! hygienic_example {
    () => {
        let x = 42; // This x doesn't conflict with outer x
        println!("Inner x: {}", x);
    };
}

fn hygiene_test() {
    println!("=== Hygienic Variables ===\n");

    let x = 100;
    println!("Outer x: {}", x);

    hygienic_example!();

    println!("Outer x again: {}", x); // Still 100
}

// The ident fragment specifier creates a variable in the outer scope
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
    println!("\n=== Breaking Hygiene with ident ===\n");

    set_value!(counter = 0);
    println!("Counter: {}", counter);

    let mut mutable_counter = 0;
    increment!(mutable_counter);
    println!("Incremented counter: {}", mutable_counter);
}

// Macros must be defined before use
macro_rules! early_macro {
    () => {
        println!("Defined early");
    };
}

fn can_use_early() {
    early_macro!();
}

macro_rules! late_macro {
    () => {
        println!("Defined late");
    };
}

fn can_use_late() {
    late_macro!();
}

fn scope_ordering_demo() {
    println!("\n=== Macro Scope and Ordering ===\n");

    can_use_early();
    can_use_late();
}

// Module visibility demo
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
        private_macro!();
    }
}

fn visibility_example() {
    println!("\n=== Module Visibility ===\n");

    public_macro!();
    // private_macro!(); // Error: not in scope
    macros::use_private();
}

// Capture context intentionally
macro_rules! with_context {
    ($name:ident, $body:block) => {{
        let context = "macro context";
        let $name = context;
        $body
    }};
}

fn context_example() {
    println!("\n=== Context Capture ===\n");

    with_context!(ctx, {
        println!("Context: {}", ctx);
    });
}

// Create variables with specific types
macro_rules! declare {
    ($name:ident: $ty:ty = $val:expr) => {
        let $name: $ty = $val;
    };
}

// Mutable variable creation
macro_rules! declare_mut {
    ($name:ident = $val:expr) => {
        let mut $name = $val;
    };
}

fn variable_creation_demo() {
    println!("\n=== Variable Creation Patterns ===\n");

    declare!(x: i32 = 42);
    println!("Declared x: {}", x);

    declare_mut!(counter = 0);
    counter += 10;
    println!("Mutable counter: {}", counter);
}

// Nested macro calls
macro_rules! outer {
    ($inner:tt) => {
        println!("Outer before");
        inner!($inner);
        println!("Outer after");
    };
}

macro_rules! inner {
    ($msg:expr) => {
        println!("Inner: {}", $msg);
    };
}

fn nested_macro_demo() {
    println!("\n=== Nested Macro Calls ===\n");

    outer!("hello from nested");
}

fn main() {
    println!("=== Declarative Macros: Hygiene and Scoping ===\n");

    hygiene_test();
    breaking_hygiene();
    scope_ordering_demo();
    visibility_example();
    context_example();
    variable_creation_demo();
    nested_macro_demo();

    println!("\nHygiene and scoping demo completed");
}
