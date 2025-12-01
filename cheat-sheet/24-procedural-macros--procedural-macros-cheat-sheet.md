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