# Procedural Macros
This chapter covers procedural macros—full Rust functions that manipulate TokenStreams. Three types: derive (#[derive(Trait)]), attribute (#[my_attr]), function-like (sql!). Must be separate crate. Use syn to parse, quote to generate. Powers serde, tokio, clap ecosystem.


Procedural macros require separate crate with `proc-macro = true`:

```toml
# Cargo.toml for the main crate
[dependencies]
my_macros = { path = "my_macros" }

# my_macros/Cargo.toml
[package]
name = "my_macros"
version = "0.1.0"
edition = "2021"

[lib]
proc-macro = true

[dependencies]
syn = { version = "2.0", features = ["full", "extra-traits"] }
quote = "1.0"
proc-macro2 = "1.0"
```

---

## Pattern 1: Derive Macros

**Problem**: Auto-implementing traits for many types tedious—manual Debug impl for 50 structs, forget to add new fields. Serde needs Serialize/Deserialize for every type—manual impls error-prone.

**Solution**: Write proc_macro_derive function that receives TokenStream (struct definition). Parse with syn::parse into DeriveInput (AST).

**Why It Matters**: Powers entire derive ecosystem—serde, clap, thiserror all use proc_macro_derive. Type-safe codegen: inspects actual struct definition.

**Use Cases**: serde Serialize/Deserialize (JSON/binary), Clone/Debug/PartialEq derives, builder patterns (derive_builder crate), ORM models (Diesel, SeaORM), command-line parsers (clap), error types (thiserror), validation (validator), getters/setters.

### Example: Basic Derive Pattern
This demonstrates the fundamental derive macro workflow. The `parse_macro_input!` macro converts the raw TokenStream into a structured `DeriveInput` AST. Extract the type name via `input.ident` and use `quote!` to generate the trait implementation with `stringify!` for compile-time reflection.

```rust
// my_macros/examples/lib.rs
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(HelloWorld)]
pub fn hello_world_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);  // Parse to AST
    let name = &input.ident;  // Get struct/enum name

    let expanded = quote! {  // Generate impl
        impl HelloWorld for #name {
            fn hello_world() {
                println!("Hello, World! My name is {}", stringify!(#name));
            }
        }
    };

    TokenStream::from(expanded)
}

```

```rust
// main.rs - using the derive macro
trait HelloWorld {
    fn hello_world();
}

#[derive(HelloWorld)]
struct MyStruct;

#[derive(HelloWorld)]
struct AnotherStruct;

fn main() {
    MyStruct::hello_world();
    AnotherStruct::hello_world();
}
MyStruct::hello_world(); // Prints "Hello, World! My name is MyStruct"
```

### Example: Derive Macro with Field Access
This shows how to inspect struct fields at compile time. Pattern-match on `Data::Struct` to extract fields, then handle `Fields::Named`, `Fields::Unnamed`, and `Fields::Unit` variants. Use `#(...)*` repetition in `quote!` for iteration and `syn::Index::from` for tuple struct indices.

```rust
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Data, Fields};

#[proc_macro_derive(Describe)]
pub fn describe_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let description = match &input.data {  // Handle struct/enum/union
        Data::Struct(data_struct) => {
            match &data_struct.fields {
                Fields::Named(fields) => {
                    let field_names: Vec<_> = fields.named
                        .iter()
                        .map(|f| &f.ident)
                        .collect();

                    quote! {
                        impl Describe for #name {
                            fn describe(&self) -> String {
                                format!(
                                    "{} {{ {} }}",
                                    stringify!(#name),
                                    vec![
                                        #(
                                            format!("{}: {:?}", stringify!(#field_names), self.#field_names)
                                        ),*
                                    ].join(", ")
                                )
                            }
                        }
                    }
                }
                Fields::Unnamed(fields) => {
                    let field_indices: Vec<_> = (0..fields.unnamed.len())
                        .map(syn::Index::from)
                        .collect();

                    quote! {
                        impl Describe for #name {
                            fn describe(&self) -> String {
                                format!(
                                    "{}({:?})",
                                    stringify!(#name),
                                    (#(self.#field_indices,)*)
                                )
                            }
                        }
                    }
                }
                Fields::Unit => {
                    quote! {
                        impl Describe for #name {
                            fn describe(&self) -> String {
                                stringify!(#name).to_string()
                            }
                        }
                    }
                }
            }
        }
        Data::Enum(_) => {
            quote! {
                impl Describe for #name {
                    fn describe(&self) -> String {
                        format!("{:?}", self)
                    }
                }
            }
        }
        Data::Union(_) => {
            panic!("Unions are not supported");
        }
    };

    TokenStream::from(description)
}
```

```rust
// Usage
trait Describe {
    fn describe(&self) -> String;
}

#[derive(Describe)]
struct Person {
    name: String,
    age: u32,
}

#[derive(Describe)]
struct Point(i32, i32);

fn main() {
    let person = Person {
        name: "Alice".to_string(),
        age: 30,
    };
    println!("{}", person.describe());

    let point = Point(10, 20);
    println!("{}", point.describe());
}
person.describe(); // Returns "Person { name: "Alice", age: 30 }"
```

### Example: Derive Macro with Attributes
This implements the builder pattern via derive macro. Register helper attributes using `attributes(builder)` in proc_macro_derive. The generated builder struct wraps fields in `Option<T>` for incremental construction, with fluent setters returning `Self` and `build()` validating all required fields.

```rust
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Data, Fields, Attribute};

#[proc_macro_derive(Builder, attributes(builder))]
pub fn builder_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let builder_name = syn::Ident::new(&format!("{}Builder", name), name.span());

    let fields = match &input.data {
        Data::Struct(data) => {
            match &data.fields {
                Fields::Named(fields) => &fields.named,
                _ => panic!("Builder only supports named fields"),
            }
        }
        _ => panic!("Builder only supports structs"),
    };

    let builder_fields = fields.iter().map(|f| {  // Option<T> wrappers
        let name = &f.ident;
        let ty = &f.ty;
        quote! {
            #name: Option<#ty>
        }
    });

    let setters = fields.iter().map(|f| {  // Fluent setters
        let name = &f.ident;
        let ty = &f.ty;
        quote! {
            pub fn #name(mut self, value: #ty) -> Self {
                self.#name = Some(value);
                self
            }
        }
    });

    let build_fields = fields.iter().map(|f| {  // Build with validation
        let name = &f.ident;
        quote! {
            #name: self.#name.ok_or(concat!("Field not set: ", stringify!(#name)))?
        }
    });

    let expanded = quote! {
        pub struct #builder_name {
            #(#builder_fields,)*
        }

        impl #builder_name {
            pub fn new() -> Self {
                Self {
                    #(#name: None,)*
                }
            }

            #(#setters)*

            pub fn build(self) -> Result<#name, Box<dyn std::error::Error>> {
                Ok(#name {
                    #(#build_fields,)*
                })
            }
        }

        impl #name {
            pub fn builder() -> #builder_name {
                #builder_name::new()
            }
        }
    };

    let field_names = fields.iter().map(|f| &f.ident);  // For initialization

    let expanded = quote! {
        pub struct #builder_name {
            #(#builder_fields,)*
        }

        impl #builder_name {
            pub fn new() -> Self {
                Self {
                    #(#field_names: None,)*
                }
            }

            #(#setters)*

            pub fn build(self) -> Result<#name, Box<dyn std::error::Error>> {
                Ok(#name {
                    #(#build_fields,)*
                })
            }
        }

        impl #name {
            pub fn builder() -> #builder_name {
                #builder_name::new()
            }
        }
    };

    TokenStream::from(expanded)
}
```

```rust
// Usage
#[derive(Builder)]
struct User {
    username: String,
    email: String,
    age: u32,
}

fn main() {
    let user = User::builder()
        .username("alice".to_string())
        .email("alice@example.com".to_string())
        .age(30)
        .build()
        .unwrap();
}
User::builder().username("alice").email("a@b.com").age(30).build()?
```

### Example: Derive Macro for Enums
This demonstrates enum introspection at compile time. Extract variant information from `Data::Enum` via `DataEnum` destructuring, then collect identifiers into a vector. The generated `variants()` returns a static slice of values, while `variant_names()` provides strings. This pattern powers strum and enum-iterator.

```rust
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Data, DataEnum};

#[proc_macro_derive(EnumIter)]
pub fn enum_iter_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let variants = match &input.data {
        Data::Enum(DataEnum { variants, .. }) => variants,
        _ => panic!("EnumIter only works on enums"),
    };

    let variant_idents: Vec<_> = variants  // Unit variants only
        .iter()
        .map(|variant| &variant.ident)
        .collect();

    let expanded = quote! {
        impl #name {
            pub fn variants() -> &'static [#name] {
                &[
                    #(#name::#variant_idents,)*
                ]
            }

            pub fn variant_names() -> &'static [&'static str] {
                &[
                    #(stringify!(#variant_idents),)*
                ]
            }
        }
    };

    TokenStream::from(expanded)
}
```

```rust
// Usage
#[derive(Debug, Copy, Clone, EnumIter)]
enum Color {
    Red,
    Green,
    Blue,
}

fn main() {
    for color in Color::variants() {
        println!("{:?}", color);
    }

    for name in Color::variant_names() {
        println!("{}", name);
    }
}
Color::variants() returns &[Color::Red, Color::Green, Color::Blue]
```

## Pattern 2: Attribute Macros

**Problem**: Need to modify/wrap functions without changing their code—add timing, logging, tracing. Want aspect-oriented programming (cross-cutting concerns).

**Solution**: Write proc_macro_attribute function receiving two TokenStreams: attribute args and item being annotated. Parse item (function, struct, impl) with syn.

**Why It Matters**: Enables tokio::main and tokio::test (essential for async). tracing::instrument adds automatic logging (non-invasive observability).

**Use Cases**: tokio::main/test (async runtime), tracing::instrument (automatic logging), web framework routes (actix, axum, rocket), test fixtures, memoization/caching, error handling wrappers, performance timing, authorization checks.

### Example: Attribute Macro Pattern
This shows how attribute macros wrap functions with instrumentation. The macro receives two TokenStreams: attribute arguments and the annotated item. Parse using `ItemFn`, preserving signature, visibility, and attributes. The closure pattern `(|| #fn_block)()` captures returns for before/after measurement.

```rust
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn};

#[proc_macro_attribute]
pub fn timing(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);

    let fn_name = &input.sig.ident;
    let fn_block = &input.block;
    let fn_sig = &input.sig;
    let fn_vis = &input.vis;
    let fn_attrs = &input.attrs;

    let expanded = quote! {
        #(#fn_attrs)*
        #fn_vis #fn_sig {
            let start = std::time::Instant::now();
            let result = (|| #fn_block)();
            let elapsed = start.elapsed();
            println!("Function {} took {:?}", stringify!(#fn_name), elapsed);
            result
        }
    };

    TokenStream::from(expanded)
}
```

```rust
// Usage
#[timing]
fn slow_function() {
    std::thread::sleep(std::time::Duration::from_millis(100));
}

fn main() {
    slow_function(); // Prints: Function slow_function took ~100ms
}
slow_function(); // Auto-prints elapsed time after execution
```

### Example: Attribute Macro with Parameters
This demonstrates parameterized attribute macros with custom argument parsing. Implement the `Parse` trait for a `LogArgs` struct to extract the `LitStr` prefix. The parsed value interpolates via `#prefix` in `quote!`, enabling configurable per-call-site behavior used extensively in logging crates.

```rust
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn, parse::{Parse, ParseStream}, LitStr};

struct LogArgs {
    prefix: LitStr,
}

impl Parse for LogArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(LogArgs {
            prefix: input.parse()?,
        })
    }
}

#[proc_macro_attribute]
pub fn log(attr: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(attr as LogArgs);
    let input = parse_macro_input!(item as ItemFn);

    let prefix = args.prefix.value();
    let fn_name = &input.sig.ident;
    let fn_block = &input.block;
    let fn_sig = &input.sig;
    let fn_vis = &input.vis;

    let expanded = quote! {
        #fn_vis #fn_sig {
            println!("{} Entering {}", #prefix, stringify!(#fn_name));
            let result = (|| #fn_block)();
            println!("{} Exiting {}", #prefix, stringify!(#fn_name));
            result
        }
    };

    TokenStream::from(expanded)
}
```

```rust
// Usage
#[log("[INFO]")]
fn my_function() {
    println!("Doing work...");
}

fn main() {
    my_function();
    // Output:
    // [INFO] Entering my_function
    // Doing work...
    // [INFO] Exiting my_function
}
my_function(); // Logs entry/exit with [INFO] prefix
```

### Example: Method Attribute Macro
This shows attribute macros applied to methods within impl blocks. Parse using `ImplItemFn` instead of `ItemFn` for method-specific syntax. The generated code wraps the method body with a `lazy_static!` cache for memoization using `Mutex<HashMap>`. This pattern powers cached and lru_cache crates.

```rust
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ImplItemFn};

#[proc_macro_attribute]
pub fn cache(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ImplItemFn);
    let fn_name = &input.sig.ident;
    let fn_block = &input.block;
    let fn_sig = &input.sig;
    let fn_vis = &input.vis;
    let cache_name = syn::Ident::new(&format!("{}_cache", fn_name), fn_name.span());

    let expanded = quote! {
        #fn_vis #fn_sig {
            use std::sync::Mutex;
            use std::collections::HashMap;

            lazy_static::lazy_static! {
                static ref #cache_name: Mutex<HashMap<String, _>> = Mutex::new(HashMap::new());
            }

            #fn_block  // In production, hash arguments properly
        }
    };

    TokenStream::from(expanded)
}
```

### Example: Struct Attribute Macro
This demonstrates modifying struct definitions with attribute macros. Parse using `ItemStruct` then mutate the fields collection to inject a new `_debug_info: String` field. The technique `Field::parse_named.parse2(quote!{...})` creates AST nodes from tokens for programmatic field insertion and accessor generation.

```rust
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemStruct};

#[proc_macro_attribute]
pub fn add_debug_info(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(item as ItemStruct);

    if let syn::Fields::Named(ref mut fields) = input.fields {  // Inject field
        fields.named.push(
            syn::Field::parse_named
                .parse2(quote! { _debug_info: String })
                .unwrap()
        );
    }

    let name = &input.ident;

    let expanded = quote! {
        #input

        impl #name {
            pub fn debug_info(&self) -> &str {
                &self._debug_info
            }
        }
    };

    TokenStream::from(expanded)
}
```

```rust
// Usage
#[add_debug_info]
struct MyStruct {
    value: i32,
}

fn main() {
    let s = MyStruct {
        value: 42,
        _debug_info: "Created at startup".to_string(),
    };
    println!("{}", s.debug_info());
}
s.debug_info(); // Returns the _debug_info string
```

## Pattern 3: Function-like Macros

**Problem**: Need custom syntax beyond derive/attribute—sql! needs full SQL parsing with compile-time checking.

**Solution**: Write proc_macro function receiving TokenStream. Implement syn::parse::Parse for custom syntax structs.

**Why It Matters**: Enables sophisticated DSLs—sqlx::query! validates SQL against database schema at compile-time.

**Use Cases**: SQL DSLs (sqlx, diesel—type-checked queries), HTML templates (yew html!, maud), configuration DSLs, query builders, compile-time regex validation, format string checking, JSON literals with validation, embedded language DSLs.

### Example: Function-like Macro Pattern
This demonstrates function-like macros that accept arbitrary syntax. Unlike derive and attribute macros, input tokens need not be valid Rust. Implement `Parse` to define custom grammar rules for your DSL, enabling compile-time SQL validation in sqlx and type-safe HTML in yew.

```rust
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse::{Parse, ParseStream}, Expr, Token};

struct SqlQuery {
    query: Expr,
}

impl Parse for SqlQuery {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(SqlQuery {
            query: input.parse()?,
        })
    }
}

#[proc_macro]
pub fn sql(input: TokenStream) -> TokenStream {
    let SqlQuery { query } = parse_macro_input!(input as SqlQuery);

    let expanded = quote! {
        {
            let query_str = #query;  // Validate SQL at compile time
            println!("Executing SQL: {}", query_str);
            query_str
        }
    };

    TokenStream::from(expanded)
}
```

```rust
// Usage
fn main() {
    let query = sql!("SELECT * FROM users WHERE age > 18");
    println!("Query: {}", query);
}
let q = sql!("SELECT * FROM users"); // Compile-time SQL validation
```

### Example: Complex Function-like Macro
This creates a hashmap literal macro with custom `key => value` syntax. Parse the fat arrow using `Token![=>]` from syn. The `Punctuated` type with `parse_terminated` handles comma-separated pairs. Define structs mirroring grammar structure and implement `Parse` for each component.

```rust
use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    Ident, Token, Expr,
};

struct HashMapLiteral {
    entries: Punctuated<KeyValue, Token![,]>,
}

struct KeyValue {
    key: Expr,
    _arrow: Token![=>],
    value: Expr,
}

impl Parse for KeyValue {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(KeyValue {
            key: input.parse()?,
            _arrow: input.parse()?,
            value: input.parse()?,
        })
    }
}

impl Parse for HashMapLiteral {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(HashMapLiteral {
            entries: input.parse_terminated(KeyValue::parse, Token![,])?,
        })
    }
}

#[proc_macro]
pub fn hashmap(input: TokenStream) -> TokenStream {
    let HashMapLiteral { entries } = parse_macro_input!(input as HashMapLiteral);

    let insertions = entries.iter().map(|kv| {
        let key = &kv.key;
        let value = &kv.value;
        quote! {
            map.insert(#key, #value);
        }
    });

    let expanded = quote! {
        {
            let mut map = std::collections::HashMap::new();
            #(#insertions)*
            map
        }
    };

    TokenStream::from(expanded)
}
hashmap! { "key" => "value" } 
```

```rust
// Usage
fn main() {
    let map = hashmap! {
        "name" => "Alice",
        "role" => "Developer",
        "level" => "Senior"
    };

    println!("{:?}", map);
}
let m = hashmap!{ "a" => 1, "b" => 2 }; // HashMap with 2 entries
```

### Example: DSL Function-like Macro
This builds a route definition DSL similar to web frameworks. Parse `METHOD, "/path" => handler` syntax with semicolon-separated routes. The macro generates a closure that pattern-matches requests against registered routes, powering actix-web, axum, and rocket endpoint definitions.

```rust
use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    Ident, Token, Expr, braced,
};

struct RouteDefinition {
    method: Ident,
    _comma: Token![,],
    path: Expr,
    _arrow: Token![=>],
    handler: Expr,
}

impl Parse for RouteDefinition {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(RouteDefinition {
            method: input.parse()?,
            _comma: input.parse()?,
            path: input.parse()?,
            _arrow: input.parse()?,
            handler: input.parse()?,
        })
    }
}

struct Routes {
    routes: Vec<RouteDefinition>,
}

impl Parse for Routes {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut routes = Vec::new();

        while !input.is_empty() {
            routes.push(input.parse()?);

            if !input.is_empty() {
                input.parse::<Token![;]>()?;
            }
        }

        Ok(Routes { routes })
    }
}

#[proc_macro]
pub fn routes(input: TokenStream) -> TokenStream {
    let Routes { routes } = parse_macro_input!(input as Routes);

    let route_matches = routes.iter().map(|route| {
        let method = &route.method;
        let path = &route.path;
        let handler = &route.handler;

        quote! {
            if request.method == stringify!(#method) && request.path == #path {
                return #handler(request);
            }
        }
    });

    let expanded = quote! {
        |request: Request| -> Response {
            #(#route_matches)*
            Response::not_found()
        }
    };

    TokenStream::from(expanded)
}
routes!(GET, "/users" => handle_users; POST, "/users" => create_user)
```

## Pattern 4: Token Stream Manipulation

**Problem**: Need fine-grained token control beyond syn's abstractions. syn doesn't support your custom syntax.

**Solution**: Work with TokenStream and TokenTree directly. Iterate tokens, match on TokenTree variants (Group, Ident, Punct, Literal).

**Why It Matters**: Maximum flexibility—handle any syntax, even invalid Rust. Performance: direct token manipulation faster than full syn parse.

**Use Cases**: Custom DSL parsers (syntax not in syn), performance-critical macros (skip syn overhead), advanced derive scenarios, token filters/transformers, macro composition, debugging token structure, embedded language parsers.

### Example: Direct TokenStream Pattern
This demonstrates low-level token manipulation without syn parsing. Collect input into `Vec<TokenTree>` for direct manipulation, then collect back into TokenStream. The four `TokenTree` variants are `Group`, `Ident`, `Punct`, and `Literal`. Useful for simple transforms or unsupported syntax.

```rust
use proc_macro::{TokenStream, TokenTree, Delimiter, Group, Punct, Spacing};
use quote::quote;

#[proc_macro]
pub fn reverse_tokens(input: TokenStream) -> TokenStream {
    let tokens: Vec<TokenTree> = input.into_iter().collect();
    let reversed: TokenStream = tokens.into_iter().rev().collect();
    reversed
}
reverse_tokens!(a b c)
```

### Example: Token Inspection
This shows how to debug macro inputs by inspecting token structure. Use `eprintln!` for compile-time debug output. Groups contain delimiters; Idents are identifiers; Puncts have spacing (Joint or Alone); Literals hold strings and numbers. Clone TokenStream before iteration to preserve the original.

```rust
use proc_macro::TokenStream;

#[proc_macro]
pub fn inspect_tokens(input: TokenStream) -> TokenStream {
    eprintln!("Token count: {}", input.clone().into_iter().count());

    for token in input.clone() {
        match token {
            proc_macro::TokenTree::Group(g) => {
                eprintln!("Group: delimiter={:?}", g.delimiter());
            }
            proc_macro::TokenTree::Ident(i) => {
                eprintln!("Ident: {}", i);
            }
            proc_macro::TokenTree::Punct(p) => {
                eprintln!("Punct: {} (spacing={:?})", p.as_char(), p.spacing());
            }
            proc_macro::TokenTree::Literal(l) => {
                eprintln!("Literal: {}", l);
            }
        }
    }

    input
}
inspect_tokens!(my code here) 
```

### Example: Building TokenStream from Scratch
This demonstrates constructing token streams programmatically without quote. Create identifiers using `Ident::new` with `Span::call_site()` for error reporting. Groups wrap tokens in delimiters like Brace. While verbose, this enables fully dynamic code generation and precise span manipulation.

```rust
use proc_macro::{TokenStream, TokenTree, Ident, Span, Literal};

#[proc_macro]
pub fn build_struct(_input: TokenStream) -> TokenStream {
    let mut tokens = TokenStream::new();

    tokens.extend(Some(TokenTree::Ident(Ident::new("struct", Span::call_site()))));
    tokens.extend(Some(TokenTree::Ident(Ident::new("MyStruct", Span::call_site()))));

    let mut fields = TokenStream::new();  // { ... }
    fields.extend(Some(TokenTree::Ident(Ident::new("value", Span::call_site()))));
    fields.extend(Some(TokenTree::Punct(proc_macro::Punct::new(':', proc_macro::Spacing::Alone))));
    fields.extend(Some(TokenTree::Ident(Ident::new("i32", Span::call_site()))));

    tokens.extend(Some(TokenTree::Group(
        proc_macro::Group::new(proc_macro::Delimiter::Brace, fields)
    )));

    tokens
}
build_struct!() 
struct MyStruct { value: i32 }
```

### Example: Span Manipulation
This shows how to preserve spans for accurate compiler error messages. When creating derived identifiers, copy the span from original tokens to maintain error positioning. The `call_site()` points to macro invocation, while `mixed_site()` provides limited hygiene. Propagate spans for better debugging.

```rust
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Ident, parse::{Parse, ParseStream}};

struct SpanExample {
    name: Ident,
}

impl Parse for SpanExample {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(SpanExample {
            name: input.parse()?,
        })
    }
}

#[proc_macro]
pub fn with_span(input: TokenStream) -> TokenStream {
    let SpanExample { name } = parse_macro_input!(input as SpanExample);
    let span = name.span();  // Preserve original span
    let prefixed = Ident::new(&format!("prefixed_{}", name), span);  // Same span

    let expanded = quote! {
        let #prefixed = stringify!(#name);
    };

    TokenStream::from(expanded)
}
with_span!(foo) 
let prefixed_foo = "foo";
```

## Pattern 5: Macro Helper Crates (syn, quote)

**Problem**: Parsing TokenStream manually is tedious—80% of macro is parsing boilerplate. Generating code with string concatenation error-prone (unbalanced braces, hygiene bugs).

**Solution**: Use syn to parse TokenStream into AST (DeriveInput, ItemFn, etc.). quote!

**Why It Matters**: syn eliminates 90% of parsing code—DeriveInput has all struct info. quote!

**Use Cases**: All proc macros (syn+quote ubiquitous), derive macros (syn::DeriveInput), attribute parsing (darling), testing macros (proc_macro2), custom parsing (syn::parse::Parse), code generation (quote!), field iteration, type inspection.

### Example: syn Parsing Pattern
This demonstrates custom parsing with syn by defining types mirroring syntax structure. The `braced!` macro extracts brace content into a ParseStream. Use `parse_terminated` for comma-separated lists. Syn provides `DeriveInput`, `ItemFn`, `ItemStruct` for common constructs; custom Parse enables arbitrary grammars.

```rust
use syn::{
    parse::{Parse, ParseStream},
    Ident, Token, Type, Visibility,
    braced, punctuated::Punctuated,
};

struct StructDef {  // Custom struct parser
    vis: Visibility,
    _struct_token: Token![struct],
    name: Ident,
    fields: Punctuated<Field, Token![,]>,
}

struct Field {
    name: Ident,
    _colon: Token![:],
    ty: Type,
}

impl Parse for Field {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Field {
            name: input.parse()?,
            _colon: input.parse()?,
            ty: input.parse()?,
        })
    }
}

impl Parse for StructDef {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        Ok(StructDef {
            vis: input.parse()?,
            _struct_token: input.parse()?,
            name: input.parse()?,
            fields: {
                braced!(content in input);
                content.parse_terminated(Field::parse, Token![,])?
            },
        })
    }
}
pub struct Foo { x: i32, y: String } // into StructDef AST
```

### Example: Using quote for Code Generation
This demonstrates the quote crate for template-based code generation. Use `#variable` to interpolate values and `#(...)*` to repeat patterns for iterators. The `format_ident!` macro creates dynamic identifiers from strings. Quote produces TokenStreams with correct spans and hygiene automatically.

```rust
use quote::{quote, format_ident};
use syn::Ident;

fn generate_getter(struct_name: &Ident, field_name: &Ident, field_type: &syn::Type) -> proc_macro2::TokenStream {
    quote! {
        impl #struct_name {
            pub fn #field_name(&self) -> &#field_type {
                &self.#field_name
            }
        }
    }
}

fn generate_multiple_methods(name: &str, count: usize) -> proc_macro2::TokenStream {
    let methods = (0..count).map(|i| {
        let method_name = format_ident!("method_{}", i);
        quote! {
            pub fn #method_name(&self) -> i32 {
                #i
            }
        }
    });

    quote! {
        impl MyStruct {
            #(#methods)*
        }
    }
}
generate_multiple_methods("MyStruct", 3)
```

### Example: Advanced syn Features
This shows advanced attribute parsing using syn's meta types. Match on `Meta::NameValue` for key-value attributes, then destructure through `Expr::Lit` and `Lit::Str` for string values. Doc comments become `#[doc = "..."]` internally. Essential for serde-style field attributes and clap configuration.

```rust
use syn::{
    Attribute, Expr, ExprLit, Lit, Meta, MetaNameValue,
};

fn parse_custom_attribute(attr: &Attribute) -> Option<String> {  // Extract doc strings
    if attr.path().is_ident("doc") {
        if let Meta::NameValue(MetaNameValue {
            value: Expr::Lit(ExprLit {
                lit: Lit::Str(s),
                ..
            }),
            ..
        }) = &attr.meta {
            return Some(s.value());
        }
    }
    None
}

fn get_doc_comments(attrs: &[Attribute]) -> Vec<String> {  // Collect doc comments
    attrs.iter()
        .filter_map(parse_custom_attribute)
        .collect()
}
get_doc_comments(&item.attrs) 
```

### Example: Combining syn and quote
This demonstrates the standard procedural macro workflow combining syn and quote. Parse input with syn to extract structured data, transform by iterating fields with `.map()`, then generate code using quote interpolation. The `format_ident!` creates dynamic identifiers like `set_field` for setters.

```rust
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Data, Fields};

#[proc_macro_derive(GetterSetter)]
pub fn getter_setter_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let fields = match &input.data {
        Data::Struct(data) => {
            match &data.fields {
                Fields::Named(fields) => &fields.named,
                _ => panic!("Only named fields supported"),
            }
        }
        _ => panic!("Only structs supported"),
    };

    let getters = fields.iter().map(|f| {  // Generate getters
        let field_name = &f.ident;
        let field_type = &f.ty;
        quote! {
            pub fn #field_name(&self) -> &#field_type {
                &self.#field_name
            }
        }
    });

    let setters = fields.iter().map(|f| {  // Generate setters
        let field_name = &f.ident;
        let field_type = &f.ty;
        let setter_name = quote::format_ident!("set_{}", field_name.as_ref().unwrap());
        quote! {
            pub fn #setter_name(&mut self, value: #field_type) {
                self.#field_name = value;
            }
        }
    });

    let expanded = quote! {
        impl #name {
            #(#getters)*
            #(#setters)*
        }
    };

    TokenStream::from(expanded)
}
#[derive(GetterSetter)] 
```

### Example: Error Handling in Procedural Macros
This shows proper error handling for user-friendly compiler messages. Use `Error::new_spanned` to create errors pointing to problematic code with accurate spans. Call `.to_compile_error()` to convert errors into displayable TokenStreams. Use `Error::combine()` for reporting multiple validation failures at once.

```rust
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Error};

#[proc_macro_derive(Validated)]
pub fn validated_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    match &input.data {  // Validate struct-only
        syn::Data::Struct(_) => {}
        _ => {
            return Error::new_spanned(
                &input,
                "Validated can only be derived for structs"
            )
            .to_compile_error()
            .into();
        }
    }

    let name = &input.ident;

    let expanded = quote! {
        impl Validated for #name {
            fn validate(&self) -> Result<(), String> {
                Ok(())
            }
        }
    };

    TokenStream::from(expanded)
}
#[derive(Validated)]
```

### Example: Custom Parse Implementation
This demonstrates recursive parsing for tree-structured configuration DSLs. Use `peek()` for lookahead without consuming tokens. The `braced!` macro extracts nested content for recursion. The `ConfigValue` enum represents either `LitStr` terminals or nested items, enabling hierarchical configurations with arbitrary depth.

```rust
use syn::{
    parse::{Parse, ParseStream},
    Ident, Token, LitStr,
    Result,
};

enum ConfigValue {
    String(LitStr),
    Nested(Vec<ConfigItem>),
}

struct ConfigItem {
    key: Ident,
    _eq: Token![=],
    value: ConfigValue,
}

impl Parse for ConfigValue {
    fn parse(input: ParseStream) -> Result<Self> {
        if input.peek(syn::token::Brace) {
            let content;
            syn::braced!(content in input);
            let mut items = Vec::new();
            while !content.is_empty() {
                items.push(content.parse()?);
            }
            Ok(ConfigValue::Nested(items))
        } else {
            Ok(ConfigValue::String(input.parse()?))
        }
    }
}

impl Parse for ConfigItem {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(ConfigItem {
            key: input.parse()?,
            _eq: input.parse()?,
            value: input.parse()?,
        })
    }
}

```

### Example: Using proc_macro2
This shows proc_macro2, which provides an identical API to proc_macro but works outside proc-macro crates. This enables unit testing macro logic in regular test modules. Quote generates proc_macro2 TokenStreams by default, converting to proc_macro via `.into()` at macro boundaries.

```rust
use proc_macro2::{TokenStream, Span, Ident};
use quote::quote;

fn generate_with_proc_macro2() -> TokenStream {
    let struct_name = Ident::new("GeneratedStruct", Span::call_site());
    let field_name = Ident::new("value", Span::call_site());

    quote! {
        struct #struct_name {
            #field_name: i32,
        }

        impl #struct_name {
            fn new(#field_name: i32) -> Self {
                Self { #field_name }
            }
        }
    }
}

// Convert between proc_macro and proc_macro2
use proc_macro::TokenStream as TokenStream1;
use proc_macro2::TokenStream as TokenStream2;

fn convert_token_streams(input: TokenStream1) -> TokenStream1 {
    let tokens: TokenStream2 = input.into();  // Convert to proc_macro2
    let result = quote! { #tokens };          // Process
    result.into()                             // Convert back
}
```

### Example: Complete Example: JSON Serialization Macro
This implements a JSON serialization derive macro similar to serde. Iterate over named fields to extract identifiers for access and strings for JSON keys. The generated `to_json()` formats fields using debug formatting, demonstrating compile-time struct inspection with zero runtime overhead.

```rust
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Data, Fields};

#[proc_macro_derive(ToJson)]
pub fn to_json_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let json_impl = match &input.data {
        Data::Struct(data) => {
            match &data.fields {
                Fields::Named(fields) => {
                    let field_serializations = fields.named.iter().map(|f| {
                        let field_name = &f.ident;
                        let field_name_str = field_name.as_ref().unwrap().to_string();
                        quote! {
                            format!("\"{}\": {:?}", #field_name_str, self.#field_name)
                        }
                    });

                    quote! {
                        impl #name {
                            pub fn to_json(&self) -> String {
                                format!(
                                    "{{ {} }}",
                                    vec![#(#field_serializations),*].join(", ")
                                )
                            }
                        }
                    }
                }
                _ => panic!("Only named fields supported"),
            }
        }
        _ => panic!("Only structs supported"),
    };

    TokenStream::from(json_impl)
}
#[derive(ToJson)]
```

```rust
// Usage
#[derive(ToJson)]
struct Person {
    name: String,
    age: u32,
}

fn main() {
    let person = Person {
        name: "Alice".to_string(),
        age: 30,
    };
    println!("{}", person.to_json());
}
person.to_json(); // Returns { "name": "Alice", "age": 30 }
```

### Example: Testing Procedural Macros
This demonstrates testing strategies for procedural macros. Integration tests apply macros to sample code and verify behavior via assertions. Use trybuild for compile-fail tests verifying error messages. Unit test with proc_macro2 types. Run `cargo expand` to inspect generated code during development.

```rust
#[test]
fn test_derive_macro() {
    #[derive(HelloWorld)]
    struct TestStruct;

    TestStruct::hello_world();
}

#[test]
fn test_attribute_macro() {
    #[timing]
    fn test_fn() -> i32 {
        42
    }

    assert_eq!(test_fn(), 42);
}

#[test]
fn test_function_macro() {
    let map = hashmap! {
        "key1" => "value1",
        "key2" => "value2"
    };

    assert_eq!(map.get("key1"), Some(&"value1"));
}
```



### Summary

This chapter covered procedural macros:

1. **Derive Macros**: #[derive(Trait)] auto-implements traits, powers serde/clap, parses struct with syn
2. **Attribute Macros**: #[my_attr] wraps/modifies items, enables tokio::main, tracing::instrument, web routes
3. **Function-like Macros**: my_macro!() custom syntax DSLs, SQL/HTML builders, full parsing control
4. **Token Stream Manipulation**: Direct token access, maximum flexibility, performance-critical macros
5. **Helper Crates**: syn for parsing, quote! for codegen, darling for attributes, proc_macro2 for testing

**Key Takeaways**:
- Proc macros are Rust functions that manipulate TokenStreams at compile-time
- Must be in separate crate with proc-macro = true
- Three types: derive, attribute, function-like
- syn parses tokens → AST, quote! generates code
- Powers entire ecosystem: serde, tokio, clap, diesel, actix-web

**Macro Types Comparison**:
- **Derive**: Auto-implement traits (#[derive(Serialize)])
- **Attribute**: Modify items (#[tokio::main], #[get("/users")])
- **Function-like**: Custom syntax (sql!(), html!())

**Essential Crates**:
- **syn**: Parse TokenStream to AST (DeriveInput, ItemFn, etc.)
- **quote**: Generate code with interpolation (quote! { impl #name })
- **proc_macro2**: Testing (works outside proc-macro crate)
- **darling**: Declarative attribute parsing (#[my_attr(skip)])

**Common Patterns**:
- Derive for trait impls: #[derive(Serialize, Clone)]
- Attribute for wrappers: #[tokio::main], #[tracing::instrument]
- Function-like for DSLs: sql!("SELECT *"), html! { <div/> }

**When to Use Each Type**:
- Use **derive** when: Auto-implementing traits (serde, builder)
- Use **attribute** when: Wrapping/modifying items (async, logging)
- Use **function-like** when: Custom DSL syntax (SQL, HTML)

**Best Practices**:
- Always use syn+quote (don't parse manually)
- Provide clear compile errors
- Test with proc_macro2 (unit tests)
- Document with examples
- Handle edge cases (empty structs, unit enums)
- Use darling for attribute parsing

**Common Use Cases**:
- serde: Serialize/Deserialize derives
- tokio: async runtime setup (#[tokio::main])
- clap: Command-line parsing derives
- tracing: Automatic instrumentation
- Web frameworks: Route registration
- ORMs: Model derives (Diesel, SeaORM)
- Builders: Auto-generate builder pattern

**Debugging Tips**:
- cargo expand shows macro output
- proc_macro2 enables unit testing
- eprintln! in macro for debug output
- Test incrementally—start simple
- Check syn docs for AST structure
