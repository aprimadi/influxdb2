use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::{parse_macro_input, Field, Fields, Ident, ItemStruct, Lit, Meta, MetaNameValue, Type};

const INFLUX_TAG: &str = "influxdb";

pub fn impl_writeable(tokens: TokenStream) -> TokenStream {
    let krate = krate();
    let writable_krate = writable_krate();
    let input = parse_macro_input!(tokens as ItemStruct); // only struct is supported now.
    let ident = input.ident;
    let generics = input.generics;
    let measurement = input.attrs.into_iter().find_map(|a| {
        let is_outer = match a.style {
            syn::AttrStyle::Outer => true,
            syn::AttrStyle::Inner(_) => false,
        };
        let is_measurement = a.path.is_ident("measurement");
        if is_outer && is_measurement {
            Some(a)
        } else {
            None
        }
    });
    let measure_value = measurement.and_then(|attr| match attr.parse_meta() {
        Ok(Meta::NameValue(MetaNameValue {
            lit: Lit::Str(lit_str),
            ..
        })) => Some(lit_str.value()),
        _ => None,
    });

    let measure = match measure_value {
        Some(v) => format_ident!("{}", v).to_string(),
        None => ident.clone().to_string(),
    };

    let fields: Vec<FieldWritable> = match input.fields {
        Fields::Named(fields) => fields
            .named
            .into_iter()
            .filter_map(FieldWritable::from)
            .filter(|field| match field.field_type {
                FieldType::Ignore => false,
                _ => true,
            })
            .collect(),
        _ => panic!("a struct without named fields is not supported"),
    };
    let tag_writes: Vec<TokenStream2> = fields
        .iter()
        .filter_map(|f| match f.field_type {
            FieldType::Tag => {
                let ident = f.ident.clone();
                let ident_str = ident.to_string();
                let kind = f.kind.clone();
                Some(quote! {
                    w.write_all(format!("{}", #ident_str).as_bytes())?;
                    w.write_all(b"=")?;
                    w.write_all(<#kind as #writable_krate::KeyWritable>::encode_key(&self.#ident).into_bytes().as_slice())?;
                })
            }
            _ => None,
        })
        .collect();

    let fields_writes: Vec<TokenStream2> = fields
        .iter()
        .filter_map(|f| match f.field_type {
            FieldType::Field => {
                let ident = f.ident.clone();
                let ident_str = ident.to_string();
                let kind = f.kind.clone();
                Some(quote! {
                    w.write_all(format!("{}", #ident_str).as_bytes())?;
                    w.write_all(b"=")?;
                    w.write_all(<#kind as #writable_krate::ValueWritable>::encode_value(&self.#ident).into_bytes().as_slice())?;
                })
            }
            _ => None,
        })
        .collect();

    let timestamp_writes: Vec<TokenStream2>  = fields
        .iter()
        .filter_map(|f| match f.field_type {
            FieldType::Timestamp => {
                let ident = f.ident.clone();
                let kind = f.kind.clone();
                Some(quote! {
                    w.write_all(<#kind as #writable_krate::TimestampWritable>::encode_timestamp(&self.#ident).into_bytes().as_slice())?;
                })
            }
            _ => None,
        })
        .collect();

    if tag_writes.len() < 1 {
        panic!("You have to specify at least one #[tag] field.")
    }
    if timestamp_writes.len() != 1 {
        panic!("You have to specify at exact one #[timestamp] field.")
    }
    if fields_writes.len() < 1 {
        panic!("You have to specify at least one #[field] field.")
    }

    let mut combined_tag_writes = vec![];
    for (index, tag_write) in tag_writes.iter().enumerate() {
        if index > 0 {
            combined_tag_writes.push(quote!(w.write_all(b",")?;));
        }
        combined_tag_writes.push(tag_write.clone());
    }

    let mut combined_fields_writes = vec![];
    for (index, fields_write) in fields_writes.iter().enumerate() {
        if index > 0 {
            combined_fields_writes.push(quote!(w.write_all(b",")?;));
        }
        combined_fields_writes.push(fields_write.clone());
    }

    let output = quote! {
        impl #generics #krate::models::WriteDataPoint for #ident #generics
        {
            fn write_data_point_to<W>(&self,mut w: W) -> std::io::Result<()>
            where
                W: std::io::Write{
                w.write_all(format!("{},", #measure).as_bytes())?;

                #(
                    #combined_tag_writes
                )*
                w.write_all(b" ")?;
                #(
                    #combined_fields_writes
                )*
                w.write_all(b" ")?;
                #(
                    #timestamp_writes
                )*
                w.write_all(b"\n")?;

                Ok(())
            }
        }
    };
    output.into()
}

fn krate() -> TokenStream2 {
    quote!(::influxdb2)
}

fn writable_krate() -> TokenStream2 {
    let root = krate();
    quote!(#root::writable)
}

#[derive(Debug)]
struct FieldWritable {
    field_type: FieldType,
    kind: Type,
    ident: Ident,
}

impl FieldWritable {
    fn from(value: Field) -> Option<Self> {
        let ident = value.ident.unwrap();

        let field_type = value.attrs.iter().find_map(|a| {
            if a.path.is_ident(INFLUX_TAG) {
                a.parse_meta().ok().and_then(|meta| match meta {
                    Meta::List(list) => {
                        if list.path.is_ident(INFLUX_TAG) {
                            list.nested.iter().find_map(|m| match m {
                                syn::NestedMeta::Meta(Meta::Path(p)) => {
                                    if p.is_ident("tag") {
                                        Some(FieldType::Tag)
                                    } else if p.is_ident("ignore") {
                                        Some(FieldType::Ignore)
                                    } else if p.is_ident("field") {
                                        Some(FieldType::Field)
                                    } else if p.is_ident("timestamp") {
                                        Some(FieldType::Timestamp)
                                    } else {
                                        None
                                    }
                                }
                                _ => None,
                            })
                        } else {
                            None
                        }
                    }
                    _ => None,
                })
            } else {
                None
            }
        });
        Some(Self {
            field_type: field_type.unwrap_or(FieldType::Field),
            kind: value.ty,
            ident: ident,
        })
    }
}

#[derive(Eq, PartialEq, Ord, PartialOrd, Debug)]
enum FieldType {
    Tag,
    Field,
    Timestamp,
    Ignore,
}
