//! Implements the functionality to enable conversion between a struct type a
//! map container type in Rust through the use of a procedural macros.
#![recursion_limit = "128"]

extern crate proc_macro;
mod expand_tuple;
mod expand_writable;

use expand_tuple::{make_tuple_fields, make_tuple_tags};
use expand_writable::impl_writeable;
use itertools::izip;
use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Ident};

/// Implements the functionality for converting entries in a BTreeMap into
/// attributes and values of a struct. It will consume a tokenized version of
/// the initial struct declaration, and use code generation to implement the
/// `FromMap` trait for instantiating the contents of the struct.
#[proc_macro_derive(FromDataPoint)]
pub fn from_data_point(input: TokenStream) -> TokenStream {
    let ast = syn::parse_macro_input!(input as DeriveInput);

    // parse out all the field names in the struct as `Ident`s
    let fields = match ast.data {
        Data::Struct(st) => st.fields,
        _ => panic!("Implementation must be a struct"),
    };
    let idents: Vec<&Ident> = fields
        .iter()
        .filter_map(|field| field.ident.as_ref())
        .collect::<Vec<&Ident>>();

    // This is struct fields. For example:
    //
    // ```
    // struct StockQuote {
    //     ticker: String,
    //     close: f64,
    //     time: i64,
    // }
    // ```
    //
    // keys will be ["ticker", "close", "time"]
    //
    // convert all the field names into strings
    let keys: Vec<String> = idents
        .clone()
        .iter()
        .map(|ident| ident.to_string())
        .collect::<Vec<String>>();

    // Typenames: i.e. "String", "f64", "i64"
    let typenames = fields
        .iter()
        .map(|field| {
            let t = field.ty.clone();
            let s = quote! {#t}.to_string();
            s
        })
        .collect::<Vec<String>>();

    // get the name identifier of the struct input AST
    let name: &Ident = &ast.ident;
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();

    let datetime_re = regex::Regex::new(r"DateTime").unwrap();
    let duration_re = regex::Regex::new(r"Duration").unwrap();
    let base64_re = regex::Regex::new(r"Vec").unwrap();
    let mut assignments = Vec::new();
    for (key, typename, ident) in izip!(keys, typenames, idents) {
        match &typename[..] {
            "f64" => {
                assignments.push(quote! {
                    let mut key = String::from(#key);
                    if !hashmap.contains_key(&key) {
                        key = format!("_{}", key);
                    }
                    match hashmap.entry(key.clone()) {
                        ::std::collections::btree_map::Entry::Occupied(entry) => {
                            if let influxdb2_structmap::value::Value::Double(v) = entry.get() {
                                settings.#ident = (v as &::num_traits::cast::ToPrimitive).to_f64().unwrap();
                            }
                        },
                        _ => panic!("Cannot parse out map entry, key: {}", key),
                    }
                })
            }
            "i64" => {
                assignments.push(quote! {
                    let mut key = String::from(#key);
                    if !hashmap.contains_key(&key) {
                        key = format!("_{}", key);
                    }
                    match hashmap.entry(key.clone()) {
                        ::std::collections::btree_map::Entry::Occupied(entry) => {
                            if let influxdb2_structmap::value::Value::Long(v) = entry.get() {
                                settings.#ident = *v;
                            }
                        },
                        _ => panic!("Cannot parse out map entry, key: {}", key),
                    }
                })
            }
            "u64" => {
                assignments.push(quote! {
                    let mut key = String::from(#key);
                    if !hashmap.contains_key(&key) {
                        key = format!("_{}", key);
                    }
                    match hashmap.entry(key.clone()) {
                        ::std::collections::btree_map::Entry::Occupied(entry) => {
                            if let influxdb2_structmap::value::Value::UnsignedLong(v) = entry.get() {
                                settings.#ident = *v;
                            }
                        },
                        _ => panic!("Cannot parse out map entry, key: {}", key),
                    }
                })
            }
            "bool" => {
                assignments.push(quote! {
                    let mut key = String::from(#key);
                    if !hashmap.contains_key(&key) {
                        key = format!("_{}", key);
                    }
                    match hashmap.entry(key.clone()) {
                        ::std::collections::btree_map::Entry::Occupied(entry) => {
                            if let influxdb2_structmap::value::Value::Bool(v) = entry.get() {
                                settings.#ident = *v;
                            }
                        },
                        _ => panic!("Cannot parse out map entry, key: {}", key),
                    }
                })
            }
            "String" => {
                assignments.push(quote! {
                    let mut key = String::from(#key);
                    if !hashmap.contains_key(&key) {
                        key = format!("_{}", key);
                    }
                    match hashmap.entry(key.clone()) {
                        ::std::collections::btree_map::Entry::Occupied(entry) => {
                            if let influxdb2_structmap::value::Value::String(v) = entry.get() {
                                settings.#ident = v.clone();
                            }
                        },
                        _ => panic!("Cannot parse out map entry, key: {}", key),
                    }
                })
            }
            x if duration_re.is_match(x) => {
                assignments.push(quote! {
                    let mut key = String::from(#key);
                    if !hashmap.contains_key(&key) {
                        key = format!("_{}", key);
                    }
                    match hashmap.entry(key.clone()) {
                        ::std::collections::btree_map::Entry::Occupied(entry) => {
                            if let influxdb2_structmap::value::Value::Duration(v) = entry.get() {
                                settings.#ident = *v;
                            }
                        },
                        _ => panic!("Cannot parse out map entry, key: {}", key),
                    }
                })
            }
            x if datetime_re.is_match(x) => {
                assignments.push(quote! {
                    let mut key = String::from(#key);
                    if !hashmap.contains_key(&key) {
                        key = format!("_{}", key);
                    }
                    match hashmap.entry(key.clone()) {
                        ::std::collections::btree_map::Entry::Occupied(entry) => {
                            if let influxdb2_structmap::value::Value::TimeRFC(v) = entry.get() {
                                settings.#ident = *v;
                            }
                        },
                        _ => panic!("Cannot parse out map entry, key: {}", key),
                    }
                })
            }
            x if base64_re.is_match(x) => {
                assignments.push(quote! {
                    let mut key = String::from(#key);
                    if !hashmap.contains_key(&key) {
                        key = format!("_{}", key);
                    }
                    match hashmap.entry(key.clone()) {
                        ::std::collections::btree_map::Entry::Occupied(entry) => {
                            if let influxdb2_structmap::value::Value::Base64Binary(v) = entry.get() {
                                settings.#ident = *v;
                            }
                        },
                        _ => panic!("Cannot parse out map entry, key: {}", key),
                    }
                })
            }
            x => {
                panic!("{} is not handled", x);
            }
        }
    }

    // start codegen of a generic or non-generic impl for the given struct using quasi-quoting
    let tokens = quote! {
        impl #impl_generics influxdb2_structmap::FromMap for #name #ty_generics #where_clause {

            fn from_genericmap(mut hashmap: influxdb2_structmap::GenericMap) -> #name {
                let mut settings = #name::default();

                #(
                    #assignments
                )*

                settings
            }

        }
    };
    TokenStream::from(tokens)
}

#[proc_macro]
pub fn impl_tuple_tags(tokens: TokenStream) -> TokenStream {
    make_tuple_tags(tokens)
}

#[proc_macro]
pub fn impl_tuple_fields(tokens: TokenStream) -> TokenStream {
    make_tuple_fields(tokens)
}

#[proc_macro_derive(WriteDataPoint, attributes(measurement, influxdb))]
pub fn impl_influx_writable(tokens: TokenStream) -> TokenStream {
    impl_writeable(tokens)
}

#[cfg(test)]
mod tests {
    #[test]
    fn ui() {
        let t = trybuild::TestCases::new();
        t.pass("tests/struct.rs");
        t.pass("tests/multistruct.rs");
        t.pass("tests/writable.rs")
    }
}
