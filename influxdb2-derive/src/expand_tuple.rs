extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{parse_macro_input, ExprTuple, Ident};

fn ident_check(input: ExprTuple) -> Vec<Ident> {
    let generic_idents = input
        .elems
        .iter()
        .map(|e| match e {
            syn::Expr::Path(p) => {
                if p.path.segments.len() != 1 {
                    panic!("Support only generic type.")
                };
                let f = &p.path.segments[0];
                f.ident.clone()
            }
            _ => panic!("Support only generic type."),
        })
        .collect::<Vec<_>>();
    if generic_idents.len() % 2 != 0 {
        panic!("Must got even number of generic type")
    }
    generic_idents
}

pub fn make_tuple_tags(tokens: TokenStream) -> TokenStream {
    let input = parse_macro_input!(tokens as ExprTuple);
    let generic_idents = ident_check(input);

    let mut concate_str: Vec<TokenStream2> = Vec::new();
    let enumerate_generic = generic_idents.iter().enumerate().collect::<Vec<_>>();
    let mut chunks = enumerate_generic.as_slice().chunks(2).peekable();
    loop {
        match chunks.next() {
            Some(c) => {
                let first_index: syn::Index = c[0].0.into();
                let second_index: syn::Index = c[1].0.into();
                concate_str.push(quote!(res.push_str(&self.#first_index.encode_key())));
                concate_str.push(quote!(res.push_str("=")));
                concate_str.push(quote!(res.push_str(&self.#second_index.encode_key())));

                if chunks.peek().is_some() {
                    concate_str.push(quote!(res.push_str(",")))
                }
            }
            None => {
                break;
            }
        };
    }
    let generic_annotate = generic_idents
        .iter()
        .map(|i| quote!(#i: KeyWritable))
        .collect::<Vec<_>>();
    let output = quote! {
        impl <#(#generic_annotate),*> TagsWritable for (#(#generic_idents),*){
            fn encode_tags(&self) -> String {
                let mut res = String::new();
                #(
                    #concate_str;
                )*
                res
            }
        }
    };
    output.into()
}

pub fn make_tuple_fields(tokens: TokenStream) -> TokenStream {
    let input = parse_macro_input!(tokens as ExprTuple);
    let generic_idents = ident_check(input);

    let mut concate_str: Vec<TokenStream2> = Vec::new();
    let enumerate_generic = generic_idents.iter().enumerate().collect::<Vec<_>>();
    let mut chunks = enumerate_generic.as_slice().chunks(2).peekable();
    loop {
        match chunks.next() {
            Some(c) => {
                let first_index: syn::Index = c[0].0.into();
                let second_index: syn::Index = c[1].0.into();
                concate_str.push(quote!(res.push_str(&self.#first_index.encode_key())));
                concate_str.push(quote!(res.push_str("=")));
                concate_str.push(quote!(res.push_str(&self.#second_index.encode_value())));

                if chunks.peek().is_some() {
                    concate_str.push(quote!(res.push_str(",")))
                }
            }
            None => {
                break;
            }
        };
    }
    let generic_annotate = &generic_idents
        .chunks(2)
        .map(|i| {
            let first = i[0].clone();
            let second = i[1].clone();
            quote!(#first: KeyWritable, #second: ValueWritable)
        })
        .collect::<Vec<_>>();
    let output = quote! {
        impl <#(#generic_annotate),*> FieldsWritable for (#(#generic_idents),*){
            fn encode_fields(&self) -> String {
                let mut res = String::new();
                #(
                    #concate_str;
                )*
                res
            }
        }
    };
    output.into()
}
