use proc_macro::TokenStream;
use quote::{quote, format_ident};
use syn::{parse_macro_input, DeriveInput, Data, Fields, ItemFn, ItemStruct};
use syn::parse::Parser;

// ===========================================
// Pattern 1: Derive Macros
// ===========================================

/// Basic derive macro that implements HelloWorld trait
#[proc_macro_derive(HelloWorld)]
pub fn hello_world_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let expanded = quote! {
        impl HelloWorld for #name {
            fn hello_world() {
                println!("Hello, World! My name is {}", stringify!(#name));
            }
        }
    };

    TokenStream::from(expanded)
}

/// Derive macro that accesses fields to generate descriptions
#[proc_macro_derive(Describe)]
pub fn describe_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

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
                                    (#(&self.#field_indices,)*)
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

/// Builder derive macro that generates a builder pattern
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

    let builder_fields = fields.iter().map(|f| {
        let name = &f.ident;
        let ty = &f.ty;
        quote! {
            #name: Option<#ty>
        }
    });

    let builder_fields_init = fields.iter().map(|f| {
        let name = &f.ident;
        quote! {
            #name: None
        }
    });

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
                    #(#builder_fields_init,)*
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

/// EnumIter derive macro for iterating over enum variants
#[proc_macro_derive(EnumIter)]
pub fn enum_iter_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let variants = match &input.data {
        Data::Enum(data_enum) => &data_enum.variants,
        _ => panic!("EnumIter only works on enums"),
    };

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

// ===========================================
// Pattern 2: Attribute Macros
// ===========================================

/// Timing attribute macro that wraps functions with timing measurement
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

/// Log attribute macro with custom prefix
#[proc_macro_attribute]
pub fn log(attr: TokenStream, item: TokenStream) -> TokenStream {
    let prefix = attr.to_string().trim_matches('"').to_string();
    let input = parse_macro_input!(item as ItemFn);

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

/// Struct attribute macro that adds debug info field
#[proc_macro_attribute]
pub fn add_debug_info(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(item as ItemStruct);

    if let syn::Fields::Named(ref mut fields) = input.fields {
        fields.named.push(
            syn::Field::parse_named
                .parse2(quote! { pub _debug_info: String })
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

// ===========================================
// Pattern 3: Function-like Macros
// ===========================================

/// SQL-like function macro
#[proc_macro]
pub fn sql(input: TokenStream) -> TokenStream {
    let query = input.to_string();

    let expanded = quote! {
        {
            let query_str = #query;
            println!("Executing SQL: {}", query_str);
            query_str
        }
    };

    TokenStream::from(expanded)
}

/// HashMap literal macro
#[proc_macro]
pub fn hashmap(input: TokenStream) -> TokenStream {
    use syn::parse::{Parse, ParseStream};
    use syn::{Expr, Token};
    use syn::punctuated::Punctuated;

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

    struct HashMapLiteral {
        entries: Punctuated<KeyValue, Token![,]>,
    }

    impl Parse for HashMapLiteral {
        fn parse(input: ParseStream) -> syn::Result<Self> {
            Ok(HashMapLiteral {
                entries: input.parse_terminated(KeyValue::parse, Token![,])?,
            })
        }
    }

    let parsed = parse_macro_input!(input as HashMapLiteral);

    let insertions = parsed.entries.iter().map(|kv| {
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

// ===========================================
// Pattern 5: Combining syn and quote
// ===========================================

/// GetterSetter derive macro
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

    let getters = fields.iter().map(|f| {
        let field_name = &f.ident;
        let field_type = &f.ty;
        quote! {
            pub fn #field_name(&self) -> &#field_type {
                &self.#field_name
            }
        }
    });

    let setters = fields.iter().map(|f| {
        let field_name = &f.ident;
        let field_type = &f.ty;
        let setter_name = format_ident!("set_{}", field_name.as_ref().unwrap());
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

/// ToJson derive macro for simple JSON serialization
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

/// Validated derive macro with error handling example
#[proc_macro_derive(Validated)]
pub fn validated_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    match &input.data {
        syn::Data::Struct(_) => {}
        _ => {
            return syn::Error::new_spanned(
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
