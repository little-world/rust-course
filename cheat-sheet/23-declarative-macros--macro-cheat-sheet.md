
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