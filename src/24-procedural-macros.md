# Procedural Macros

[Derive Macros](#pattern-1-derive-macros)

- Problem: Auto-implementing traits tedious (Debug, Serialize, Clone for 50 types); manual impls forget fields; code generation limited in macro_rules!
- Solution: #[derive(MyTrait)] with proc_macro_derive; parse struct with syn; generate impl with quote; full AST access; TokenStream → parse → manipulate → generate
- Why It Matters: Powers serde, clap, derive ecosystem; type-safe codegen; adds fields → derive updates automatically; impossible with declarative macros
- Use Cases: serde Serialize/Deserialize, Clone/Debug derives, builder patterns, ORM derives (Diesel), clap command parsing, validation derives

[Attribute Macros](#pattern-2-attribute-macros)

- Problem: Need to modify/wrap functions; add tracing/logging; inject code before/after; declarative macros can't modify items; aspect-oriented programming in Rust
- Solution: #[my_attr] attribute macros; receive item + attr args; modify function/struct/impl; wrap with additional code; full AST manipulation
- Why It Matters: tokio::main wraps async fn; tracing instruments functions; route macros (actix-web); test frameworks; non-invasive code injection
- Use Cases: tokio::test/main (async runtime), tracing::instrument (logging), actix-web routes, test fixtures, memoization, error handling wrappers

[Function-like Macros](#pattern-3-function-like-macros)

- Problem: Custom syntax beyond derive/attribute; sql! needs full syntax parsing; html! templates; declarative macros limited; need complex parsing
- Solution: my_macro!(...) function-like proc macros; full parsing control; syn::parse for custom syntax; build DSLs; return arbitrary code
- Why It Matters: Enables complex DSLs—sql! with type checking, html! templates; more flexible than macro_rules!; full compiler integration
- Use Cases: SQL DSLs (type-safe queries), HTML templates, config DSLs, query builders, compile-time regex, format string validation

[Token Stream Manipulation](#pattern-4-token-stream-manipulation)

- Problem: Need fine-grained token control; syn too high-level; custom parsing; build syntax not in syn; low-level macro construction
- Solution: Direct TokenStream manipulation; proc_macro2::TokenTree; syn::parse for custom types; manual token construction; Punctuated for lists
- Why It Matters: Maximum flexibility—handle any syntax; syn doesn't support your grammar; custom DSL syntax; performance-critical parsing
- Use Cases: Custom DSL parsers, performance-critical macros, syntax not in syn, advanced derive scenarios, macro composition

[Macro Helper Crates (syn, quote)](pattern-5-macro-helper-crates)

- Problem: Parsing TokenStream manually tedious; constructing code error-prone; quote interpolation complex; field iteration boilerplate
- Solution: syn parses Rust syntax to AST; quote! generates code with interpolation; proc_macro2 for testing; darling for derive helpers
- Why It Matters: syn eliminates 90% parsing code; quote! readable codegen (#field_name interpolation); darling handles attributes; essential tooling
- Use Cases: All proc macros (syn+quote standard), derive macros (darling), testing (proc_macro2), attribute parsing, custom derives

[Procedural Macros Cheat Sheet](#procedural-macros-cheat-sheet)
- common patterns in procedural macros

### Overview

This chapter covers procedural macros—full Rust functions that manipulate TokenStreams. Three types: derive (#[derive(Trait)]), attribute (#[my_attr]), function-like (sql!). Must be separate crate. Use syn to parse, quote to generate. Powers serde, tokio, clap ecosystem.

## Setup

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

**Problem**: Auto-implementing traits for many types tedious—manual Debug impl for 50 structs, forget to add new fields. Serde needs Serialize/Deserialize for every type—manual impls error-prone. Builder pattern requires repetitive setter methods. Clone/PartialEq boilerplate. Declarative macros can't inspect struct fields or types. Need full AST access to generate correct impls.

**Solution**: Write proc_macro_derive function that receives TokenStream (struct definition). Parse with syn::parse into DeriveInput (AST). Inspect struct name, fields, generics. Generate trait impl using quote! macro. Return TokenStream. Compiler inserts generated impl. Can add helper attributes (#[my_attr(skip)]) for configuration. Works with structs, enums, unions.

**Why It Matters**: Powers entire derive ecosystem—serde, clap, thiserror all use proc_macro_derive. Type-safe codegen: inspects actual struct definition. Adding field automatically updates generated impl—no manual sync. Impossible with declarative macros (no AST access). Essential for library ergonomics: users just #[derive(Trait)]. One derive impl → thousands of types.

**Use Cases**: serde Serialize/Deserialize (JSON/binary), Clone/Debug/PartialEq derives, builder patterns (derive_builder crate), ORM models (Diesel, SeaORM), command-line parsers (clap), error types (thiserror), validation (validator), getters/setters.

### Example: Basic Derive Pattern
 Create simple derive macro for custom trait.

```rust
//=====================
// my_macros/src/lib.rs
//=====================
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(HelloWorld)]
pub fn hello_world_derive(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let input = parse_macro_input!(input as DeriveInput);

    // Get the name of the struct/enum
    let name = &input.ident;

    // Generate the implementation
    let expanded = quote! {
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
//=================================
// main.rs - using the derive macro
//=================================
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
```

### Example: Derive Macro with Field Access

```rust
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Data, Fields};

#[proc_macro_derive(Describe)]
pub fn describe_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    // Handle different data types
    let description = match &input.data {
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
//======
// Usage
//======
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
```

### Example: Derive Macro with Attributes

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

    // Generate builder fields
    let builder_fields = fields.iter().map(|f| {
        let name = &f.ident;
        let ty = &f.ty;
        quote! {
            #name: Option<#ty>
        }
    });

    // Generate setter methods
    let setters = fields.iter().map(|f| {
        let name = &f.ident;
        let ty = &f.ty;
        quote! {
            pub fn #name(mut self, value: #ty) -> Self {
                self.#name = Some(value);
                self
            }
        }
    });

    // Generate build method
    let build_fields = fields.iter().map(|f| {
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

    // Need to fix field names in builder initialization
    let field_names = fields.iter().map(|f| &f.ident);

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
//======
// Usage
//======
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
```

### Example: Derive Macro for Enums

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

    // Only support unit variants for simplicity
    let variant_idents: Vec<_> = variants
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
//======
// Usage
//======
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
```

## Pattern 2: Attribute Macros

**Problem**: Need to modify/wrap functions without changing their code—add timing, logging, tracing. Want aspect-oriented programming (cross-cutting concerns). tokio::main converts fn main() to async runtime setup. Can't add code before/after function with derive macros. Need to inject behavior non-invasively. Web frameworks need route registration (#[get("/users")]).

**Solution**: Write proc_macro_attribute function receiving two TokenStreams: attribute args and item being annotated. Parse item (function, struct, impl) with syn. Modify/wrap with additional code. Use quote! to generate enhanced version. Return modified TokenStream. Can parse attribute parameters. Works on functions, structs, impls, modules.

**Why It Matters**: Enables tokio::main and tokio::test (essential for async). tracing::instrument adds automatic logging (non-invasive observability). actix-web/axum route macros register handlers. Test frameworks use attributes for fixtures. Aspect-oriented: logging, metrics, caching without touching business logic. Critical for framework ergonomics.

**Use Cases**: tokio::main/test (async runtime), tracing::instrument (automatic logging), web framework routes (actix, axum, rocket), test fixtures, memoization/caching, error handling wrappers, performance timing, authorization checks.

### Example: Attribute Macro Pattern

Wrap function with timing/logging without modifying its body.

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
//======
// Usage
//======
#[timing]
fn slow_function() {
    std::thread::sleep(std::time::Duration::from_millis(100));
}

fn main() {
    slow_function(); // Prints: Function slow_function took ~100ms
}
```

### Example: Attribute Macro with Parameters

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
//======
// Usage
//======
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
```

### Example: Method Attribute Macro

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

    let cache_name = syn::Ident::new(
        &format!("{}_cache", fn_name),
        fn_name.span()
    );

    let expanded = quote! {
        #fn_vis #fn_sig {
            use std::sync::Mutex;
            use std::collections::HashMap;

            lazy_static::lazy_static! {
                static ref #cache_name: Mutex<HashMap<String, _>> = Mutex::new(HashMap::new());
            }

            // Simple cache implementation
            // In real code, you'd want to hash the arguments properly
            #fn_block
        }
    };

    TokenStream::from(expanded)
}
```

### Example: Struct Attribute Macro

```rust
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemStruct};

#[proc_macro_attribute]
pub fn add_debug_info(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(item as ItemStruct);

    // Add a field to the struct
    if let syn::Fields::Named(ref mut fields) = input.fields {
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
//======
// Usage
//======
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
```

## Pattern 3: Function-like Macros

**Problem**: Need custom syntax beyond derive/attribute—sql! needs full SQL parsing with compile-time checking. html! templates require complex syntax. Declarative macro_rules! limited (no lookahead, clunky parsing). Want my_macro!(custom syntax here) with arbitrary grammar. Configuration DSLs need validation at compile-time.

**Solution**: Write proc_macro function receiving TokenStream. Implement syn::parse::Parse for custom syntax structs. Parse with syn::parse_macro_input. Build arbitrary DSL. Use quote! to generate output code. Return TokenStream. Full parsing control—can implement any grammar. More flexible than macro_rules! with better error messages.

**Why It Matters**: Enables sophisticated DSLs—sqlx::query! validates SQL against database schema at compile-time. html! macros (yew, maud) provide type-safe templates. More flexible than declarative macros (full parsing control). Better errors: point to exact syntax issues. Essential for complex domain-specific syntax. Compile-time validation catches bugs early.

**Use Cases**: SQL DSLs (sqlx, diesel—type-checked queries), HTML templates (yew html!, maud), configuration DSLs, query builders, compile-time regex validation, format string checking, JSON literals with validation, embedded language DSLs.

### Example: Function-like Macro Pattern
Create SQL-like macro with custom parsing.

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
            let query_str = #query;
            // Validate SQL at compile time
            println!("Executing SQL: {}", query_str);
            query_str
        }
    };

    TokenStream::from(expanded)
}
```

```rust
//======
// Usage
//======
fn main() {
    let query = sql!("SELECT * FROM users WHERE age > 18");
    println!("Query: {}", query);
}
```

### Example: Complex Function-like Macro

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
```

```rust
//======
// Usage
//======
fn main() {
    let map = hashmap! {
        "name" => "Alice",
        "role" => "Developer",
        "level" => "Senior"
    };

    println!("{:?}", map);
}
```

### Example: DSL Function-like Macro

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
```

## Pattern 4: Token Stream Manipulation

**Problem**: Need fine-grained token control beyond syn's abstractions. syn doesn't support your custom syntax. Want to manipulate tokens directly (reverse, filter, transform). Performance-critical: syn parsing overhead matters. Building syntax tree manually. Need access to token spacing/delimiters.

**Solution**: Work with TokenStream and TokenTree directly. Iterate tokens, match on TokenTree variants (Group, Ident, Punct, Literal). Use proc_macro2 for testable code. Manually construct tokens. syn::parse for custom Parse impls. Punctuated for comma-separated lists. Direct token manipulation gives maximum control.

**Why It Matters**: Maximum flexibility—handle any syntax, even invalid Rust. Performance: direct token manipulation faster than full syn parse. When syn doesn't fit (custom DSL syntax, token games). Low-level building block for advanced macros. Essential for macro composition (macro calling macro). Learning tool: understand token representation.

**Use Cases**: Custom DSL parsers (syntax not in syn), performance-critical macros (skip syn overhead), advanced derive scenarios, token filters/transformers, macro composition, debugging token structure, embedded language parsers.

### Example: Direct TokenStream Pattern

 Manipulate tokens directly without syn parsing.

```rust
use proc_macro::{TokenStream, TokenTree, Delimiter, Group, Punct, Spacing};
use quote::quote;

#[proc_macro]
pub fn reverse_tokens(input: TokenStream) -> TokenStream {
    let tokens: Vec<TokenTree> = input.into_iter().collect();
    let reversed: TokenStream = tokens.into_iter().rev().collect();
    reversed
}
```

### Example: Token Inspection

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
```

### Example: Building TokenStream from Scratch

```rust
use proc_macro::{TokenStream, TokenTree, Ident, Span, Literal};

#[proc_macro]
pub fn build_struct(_input: TokenStream) -> TokenStream {
    let mut tokens = TokenStream::new();

    // struct
    tokens.extend(Some(TokenTree::Ident(Ident::new("struct", Span::call_site()))));

    // MyStruct
    tokens.extend(Some(TokenTree::Ident(Ident::new("MyStruct", Span::call_site()))));

    // { ... }
    let mut fields = TokenStream::new();
    fields.extend(Some(TokenTree::Ident(Ident::new("value", Span::call_site()))));
    fields.extend(Some(TokenTree::Punct(proc_macro::Punct::new(':', proc_macro::Spacing::Alone))));
    fields.extend(Some(TokenTree::Ident(Ident::new("i32", Span::call_site()))));

    tokens.extend(Some(TokenTree::Group(
        proc_macro::Group::new(proc_macro::Delimiter::Brace, fields)
    )));

    tokens
}
```

### Example: Span Manipulation

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

    // Get the span of the input identifier
    let span = name.span();

    // Create a new identifier with the same span
    let prefixed = Ident::new(&format!("prefixed_{}", name), span);

    let expanded = quote! {
        let #prefixed = stringify!(#name);
    };

    TokenStream::from(expanded)
}
```

## Pattern 5: Macro Helper Crates (syn, quote)

**Problem**: Parsing TokenStream manually is tedious—80% of macro is parsing boilerplate. Generating code with string concatenation error-prone (unbalanced braces, hygiene bugs). Testing proc macros requires separate crate. Attribute parsing repetitive. Iterating struct fields common but verbose. Need readable codegen with interpolation.

**Solution**: Use syn to parse TokenStream into AST (DeriveInput, ItemFn, etc.). quote! macro generates code with #{interpolation}. proc_macro2 for testable macros (works outside proc-macro crate). darling for declarative attribute parsing. Punctuated<T, P> for comma-separated lists. syn handles all Rust syntax—you work with typed AST.

**Why It Matters**: syn eliminates 90% of parsing code—DeriveInput has all struct info. quote! readable: quote! { impl #name { fn #method } } vs string building. Essential tooling: virtually all proc macros use syn+quote. darling reduces attribute boilerplate. proc_macro2 enables unit testing. Without these: writing proc macros impractical. Industry standard tools.

**Use Cases**: All proc macros (syn+quote ubiquitous), derive macros (syn::DeriveInput), attribute parsing (darling), testing macros (proc_macro2), custom parsing (syn::parse::Parse), code generation (quote!), field iteration, type inspection.

### Example: syn Parsing Pattern

 Parse Rust syntax from TokenStream into typed AST.

```rust
use syn::{
    parse::{Parse, ParseStream},
    Ident, Token, Type, Visibility,
    braced, punctuated::Punctuated,
};

//==========================
// Parse a struct definition
//==========================
struct StructDef {
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
```

### Example: Using quote for Code Generation

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
```

### Example: Advanced syn Features

```rust
use syn::{
    Attribute, Expr, ExprLit, Lit, Meta, MetaNameValue,
};

//========================
// Parse custom attributes
//========================
fn parse_custom_attribute(attr: &Attribute) -> Option<String> {
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

//===============================
// Extract documentation comments
//===============================
fn get_doc_comments(attrs: &[Attribute]) -> Vec<String> {
    attrs.iter()
        .filter_map(parse_custom_attribute)
        .collect()
}
```

### Example: Combining syn and quote

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

    // Generate getters
    let getters = fields.iter().map(|f| {
        let field_name = &f.ident;
        let field_type = &f.ty;
        quote! {
            pub fn #field_name(&self) -> &#field_type {
                &self.#field_name
            }
        }
    });

    // Generate setters
    let setters = fields.iter().map(|f| {
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
```

### Example: Error Handling in Procedural Macros

```rust
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Error};

#[proc_macro_derive(Validated)]
pub fn validated_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    // Validate that it's a struct
    match &input.data {
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
```

### Example: Custom Parse Implementation

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

//===========================================
// Convert between proc_macro and proc_macro2
//===========================================
use proc_macro::TokenStream as TokenStream1;
use proc_macro2::TokenStream as TokenStream2;

fn convert_token_streams(input: TokenStream1) -> TokenStream1 {
    let tokens: TokenStream2 = input.into();
    // Process with proc_macro2
    let result = quote! { #tokens };
    result.into()
}
```

### Example: Complete Example: JSON Serialization Macro

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
```

```rust
//======
// Usage
//======
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
```

### Example: Testing Procedural Macros

```rust
//===========================
// tests/integration_tests.rs
//===========================
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

**Setup Requirements**:
```toml
[lib]
proc-macro = true

[dependencies]
syn = { version = "2", features = ["full"] }
quote = "1"
proc_macro2 = "1"
```

### Procedural Macros Cheat Sheet
```rust
// ===== PROCEDURAL MACROS SETUP =====
// Cargo.toml for proc-macro crate:
/*
[lib]
proc-macro = true

[dependencies]
syn = { version = "2.0", features = ["full", "extra-traits"] }
quote = "1.0"
proc-macro2 = "1.0"
*/

// ===== DERIVE MACROS =====
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

// Basic derive macro
#[proc_macro_derive(HelloMacro)]
pub fn hello_macro_derive(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let input = parse_macro_input!(input as DeriveInput);
    
    // Get the name of the struct/enum
    let name = &input.ident;
    
    // Generate the implementation
    let expanded = quote! {
        impl HelloMacro for #name {
            fn hello_macro() {
                println!("Hello, Macro! My name is {}!", stringify!(#name));
            }
        }
    };
    
    TokenStream::from(expanded)
}

// Usage:
/*
#[derive(HelloMacro)]
struct Pancakes;

Pancakes::hello_macro();  // Prints: Hello, Macro! My name is Pancakes!
*/

// ===== DERIVE WITH ATTRIBUTES =====
#[proc_macro_derive(Builder, attributes(builder))]
pub fn derive_builder(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let builder_name = format!("{}Builder", name);
    let builder_ident = syn::Ident::new(&builder_name, name.span());
    
    // Extract struct fields
    let fields = match input.data {
        syn::Data::Struct(syn::DataStruct {
            fields: syn::Fields::Named(syn::FieldsNamed { ref named, .. }),
            ..
        }) => named,
        _ => panic!("Builder only works on structs with named fields"),
    };
    
    // Generate builder fields (all Option<T>)
    let builder_fields = fields.iter().map(|f| {
        let name = &f.ident;
        let ty = &f.ty;
        quote! { #name: Option<#ty> }
    });
    
    // Generate setter methods
    let setters = fields.iter().map(|f| {
        let name = &f.ident;
        let ty = &f.ty;
        quote! {
            pub fn #name(&mut self, #name: #ty) -> &mut Self {
                self.#name = Some(#name);
                self
            }
        }
    });
    
    // Generate build method
    let build_fields = fields.iter().map(|f| {
        let name = &f.ident;
        quote! {
            #name: self.#name.clone()
                .ok_or(concat!("Field ", stringify!(#name), " is not set"))?
        }
    });
    
    let expanded = quote! {
        pub struct #builder_ident {
            #(#builder_fields,)*
        }
        
        impl #builder_ident {
            #(#setters)*
            
            pub fn build(&self) -> Result<#name, Box<dyn std::error::Error>> {
                Ok(#name {
                    #(#build_fields,)*
                })
            }
        }
        
        impl #name {
            pub fn builder() -> #builder_ident {
                #builder_ident {
                    #(#(fields.iter().map(|f| {
                        let name = &f.ident;
                        quote! { #name: None }
                    })),*)*
                }
            }
        }
    };
    
    TokenStream::from(expanded)
}

// Usage:
/*
#[derive(Builder)]
struct User {
    name: String,
    age: u32,
    email: String,
}

let user = User::builder()
    .name("Alice".to_string())
    .age(30)
    .email("alice@example.com".to_string())
    .build()?;
*/

// ===== ATTRIBUTE MACROS =====
#[proc_macro_attribute]
pub fn route(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as syn::ItemFn);
    let attr = parse_macro_input!(attr as syn::LitStr);
    
    let fn_name = &input.sig.ident;
    let fn_block = &input.block;
    let fn_inputs = &input.sig.inputs;
    let fn_output = &input.sig.output;
    let route_path = attr.value();
    
    let expanded = quote! {
        pub fn #fn_name(#fn_inputs) #fn_output {
            println!("Route: {}", #route_path);
            #fn_block
        }
    };
    
    TokenStream::from(expanded)
}

// Usage:
/*
#[route("/users/:id")]
fn get_user(id: u32) -> String {
    format!("User {}", id)
}
*/

// ===== FUNCTION-LIKE MACROS =====
#[proc_macro]
pub fn sql(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as syn::LitStr);
    let query = input.value();
    
    // Parse and validate SQL at compile time
    // Generate code to execute query
    let expanded = quote! {
        {
            let query = #query;
            // Generated query execution code
            execute_query(query)
        }
    };
    
    TokenStream::from(expanded)
}

// Usage:
/*
let result = sql!("SELECT * FROM users WHERE id = ?");
*/

// ===== PARSING CUSTOM SYNTAX =====
use syn::parse::{Parse, ParseStream};

// Custom syntax: make_struct!(MyStruct { field: i32, other: String })
struct MakeStructInput {
    name: syn::Ident,
    fields: syn::punctuated::Punctuated<syn::Field, syn::Token![,]>,
}

impl Parse for MakeStructInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let name: syn::Ident = input.parse()?;
        
        let content;
        syn::braced!(content in input);
        let fields = content.parse_terminated(syn::Field::parse_named, syn::Token![,])?;
        
        Ok(MakeStructInput { name, fields })
    }
}

#[proc_macro]
pub fn make_struct(input: TokenStream) -> TokenStream {
    let MakeStructInput { name, fields } = parse_macro_input!(input as MakeStructInput);
    
    let expanded = quote! {
        struct #name {
            #fields
        }
    };
    
    TokenStream::from(expanded)
}

// ===== WORKING WITH ATTRIBUTES =====
use syn::{Attribute, Meta};

fn parse_attributes(attrs: &[Attribute]) -> Vec<String> {
    attrs.iter()
        .filter_map(|attr| {
            if attr.path().is_ident("builder") {
                match &attr.meta {
                    Meta::List(meta_list) => {
                        // Parse #[builder(arg1, arg2)]
                        Some(meta_list.tokens.to_string())
                    }
                    _ => None
                }
            } else {
                None
            }
        })
        .collect()
}

// ===== DERIVE MACRO WITH GENERICS =====
#[proc_macro_derive(Debug)]
pub fn derive_debug(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let generics = &input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    
    let expanded = quote! {
        impl #impl_generics std::fmt::Debug for #name #ty_generics #where_clause {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", stringify!(#name))
            }
        }
    };
    
    TokenStream::from(expanded)
}

// ===== ENUM VARIANTS ITERATION =====
#[proc_macro_derive(EnumIter)]
pub fn derive_enum_iter(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    
    let variants = match input.data {
        syn::Data::Enum(ref data) => &data.variants,
        _ => panic!("EnumIter only works on enums"),
    };
    
    let variant_names = variants.iter().map(|v| {
        let variant = &v.ident;
        quote! { #name::#variant }
    });
    
    let expanded = quote! {
        impl #name {
            pub fn iter() -> impl Iterator<Item = Self> {
                [#(#variant_names),*].iter().copied()
            }
        }
    };
    
    TokenStream::from(expanded)
}

// Usage:
/*
#[derive(EnumIter)]
enum Color {
    Red,
    Green,
    Blue,
}

for color in Color::iter() {
    println!("{:?}", color);
}
*/

// ===== SERIALIZATION DERIVE =====
#[proc_macro_derive(Serialize)]
pub fn derive_serialize(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    
    let fields = match input.data {
        syn::Data::Struct(syn::DataStruct {
            fields: syn::Fields::Named(ref fields),
            ..
        }) => &fields.named,
        _ => panic!("Serialize only works on structs with named fields"),
    };
    
    let field_serializers = fields.iter().map(|f| {
        let field_name = &f.ident;
        let field_str = field_name.as_ref().unwrap().to_string();
        quote! {
            serializer.serialize_field(#field_str, &self.#field_name)?;
        }
    });
    
    let field_count = fields.len();
    
    let expanded = quote! {
        impl Serialize for #name {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                use serde::ser::SerializeStruct;
                let mut serializer = serializer.serialize_struct(
                    stringify!(#name),
                    #field_count
                )?;
                #(#field_serializers)*
                serializer.end()
            }
        }
    };
    
    TokenStream::from(expanded)
}

// ===== ERROR HANDLING IN PROC MACROS =====
#[proc_macro_derive(Validated)]
pub fn derive_validated(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    
    // Return compile errors with syn::Error
    let fields = match input.data {
        syn::Data::Struct(syn::DataStruct {
            fields: syn::Fields::Named(ref fields),
            ..
        }) => &fields.named,
        _ => {
            return syn::Error::new(
                input.ident.span(),
                "Validated can only be derived for structs with named fields"
            )
            .to_compile_error()
            .into();
        }
    };
    
    // Generate code...
    TokenStream::new()
}

// ===== HELPER ATTRIBUTE MACRO =====
#[proc_macro_attribute]
pub fn log_function_call(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as syn::ItemFn);
    let fn_name = &input.sig.ident;
    let fn_block = &input.block;
    let fn_sig = &input.sig;
    
    let expanded = quote! {
        #fn_sig {
            println!("Calling function: {}", stringify!(#fn_name));
            let result = #fn_block;
            println!("Function {} completed", stringify!(#fn_name));
            result
        }
    };
    
    TokenStream::from(expanded)
}

// Usage:
/*
#[log_function_call]
fn add(a: i32, b: i32) -> i32 {
    a + b
}
*/

// ===== CONDITIONAL COMPILATION =====
#[proc_macro_derive(ConditionalImpl)]
pub fn derive_conditional(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    
    let expanded = quote! {
        #[cfg(feature = "special")]
        impl SpecialTrait for #name {
            fn special_method(&self) {
                println!("Special implementation");
            }
        }
        
        #[cfg(not(feature = "special"))]
        impl SpecialTrait for #name {
            fn special_method(&self) {
                println!("Default implementation");
            }
        }
    };
    
    TokenStream::from(expanded)
}

// ===== HELPER FUNCTIONS =====
use proc_macro2::Span;
use syn::Ident;

// Create identifier
fn make_ident(name: &str) -> Ident {
    Ident::new(name, Span::call_site())
}

// Check if type is Option
fn is_option(ty: &syn::Type) -> bool {
    if let syn::Type::Path(type_path) = ty {
        if let Some(segment) = type_path.path.segments.last() {
            return segment.ident == "Option";
        }
    }
    false
}

// Extract inner type from Option<T>
fn extract_option_inner(ty: &syn::Type) -> Option<&syn::Type> {
    if let syn::Type::Path(type_path) = ty {
        if let Some(segment) = type_path.path.segments.last() {
            if segment.ident == "Option" {
                if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                    if let Some(syn::GenericArgument::Type(inner)) = args.args.first() {
                        return Some(inner);
                    }
                }
            }
        }
    }
    None
}

// ===== PROC MACRO EXAMPLES =====

// Example 1: Getters/Setters
#[proc_macro_derive(Getters)]
pub fn derive_getters(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    
    let fields = match input.data {
        syn::Data::Struct(syn::DataStruct {
            fields: syn::Fields::Named(ref fields),
            ..
        }) => &fields.named,
        _ => return TokenStream::new(),
    };
    
    let getters = fields.iter().map(|f| {
        let field_name = &f.ident;
        let field_ty = &f.ty;
        let getter_name = make_ident(&format!("get_{}", field_name.as_ref().unwrap()));
        
        quote! {
            pub fn #getter_name(&self) -> &#field_ty {
                &self.#field_name
            }
        }
    });
    
    let expanded = quote! {
        impl #name {
            #(#getters)*
        }
    };
    
    TokenStream::from(expanded)
}

// Example 2: FromStr derive
#[proc_macro_derive(FromStr)]
pub fn derive_from_str(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    
    let variants = match input.data {
        syn::Data::Enum(ref data) => &data.variants,
        _ => panic!("FromStr only works on enums"),
    };
    
    let match_arms = variants.iter().map(|v| {
        let variant = &v.ident;
        let variant_str = variant.to_string();
        quote! {
            #variant_str => Ok(#name::#variant),
        }
    });
    
    let expanded = quote! {
        impl std::str::FromStr for #name {
            type Err = String;
            
            fn from_str(s: &str) -> Result<Self, Self::Err> {
                match s {
                    #(#match_arms)*
                    _ => Err(format!("Unknown variant: {}", s)),
                }
            }
        }
    };
    
    TokenStream::from(expanded)
}

// Example 3: Display derive for enums
#[proc_macro_derive(Display)]
pub fn derive_display(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    
    let variants = match input.data {
        syn::Data::Enum(ref data) => &data.variants,
        _ => return TokenStream::new(),
    };
    
    let match_arms = variants.iter().map(|v| {
        let variant = &v.ident;
        let variant_str = variant.to_string();
        quote! {
            #name::#variant => write!(f, #variant_str),
        }
    });
    
    let expanded = quote! {
        impl std::fmt::Display for #name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    #(#match_arms)*
                }
            }
        }
    };
    
    TokenStream::from(expanded)
}

// Example 4: Default with field attributes
#[proc_macro_derive(DefaultWithAttrs, attributes(default))]
pub fn derive_default_with_attrs(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    
    let fields = match input.data {
        syn::Data::Struct(syn::DataStruct {
            fields: syn::Fields::Named(ref fields),
            ..
        }) => &fields.named,
        _ => return TokenStream::new(),
    };
    
    let field_defaults = fields.iter().map(|f| {
        let field_name = &f.ident;
        
        // Check for #[default = "value"] attribute
        let default_value = f.attrs.iter()
            .find(|attr| attr.path().is_ident("default"))
            .and_then(|attr| {
                if let Meta::NameValue(meta) = &attr.meta {
                    Some(&meta.value)
                } else {
                    None
                }
            });
        
        if let Some(value) = default_value {
            quote! { #field_name: #value }
        } else {
            quote! { #field_name: Default::default() }
        }
    });
    
    let expanded = quote! {
        impl Default for #name {
            fn default() -> Self {
                Self {
                    #(#field_defaults,)*
                }
            }
        }
    };
    
    TokenStream::from(expanded)
}

// Usage:
/*
#[derive(DefaultWithAttrs)]
struct Config {
    #[default = "8080"]
    port: u16,
    #[default = "\"localhost\".to_string()"]
    host: String,
    verbose: bool,  // Uses Default::default()
}
*/

// ===== TESTING PROC MACROS =====
/*
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_derive_macro() {
        let input = quote! {
            struct MyStruct {
                field: i32,
            }
        };
        
        let output = derive_my_macro(input.into());
        // Assert on output
    }
}
*/
```