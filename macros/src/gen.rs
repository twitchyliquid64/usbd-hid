use proc_macro::TokenStream as TokenStream1;
use std::collections::{HashMap, HashSet};
use std::iter::FromIterator;

use proc_macro2::{Ident, Span};
use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::{Attribute, AttrStyle, Fields, ItemEnum, ItemStruct, parse_str, Type, TypeTuple, Variant, Result, parse};
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;

pub(crate) fn generate_type(
    template: &ItemStruct,
    ident: Ident,
    mut fields: HashMap<Option<u8>, Fields>
) -> TokenStream {
    let template = template.clone();
    if let Some(fields) = fields.remove(&None) {
        ItemStruct{
            ident,
            fields,
            ..template
        }.to_token_stream()
    } else {
        let mut variants_iter = Vec::<Variant>::new();
        for i in 0u8..=255u8 {
            variants_iter.push(Variant {
                attrs: Vec::new(),
                ident: parse_str(format!("X{}", i).as_str()).unwrap(),
                fields: fields.remove(&Some(i)).unwrap_or(Fields::Unit),
                discriminant: None
            });

            if fields.is_empty() {
                break
            }
        }

        ItemEnum {
            attrs: template.attrs,
            vis: template.vis,
            generics: template.generics,
            brace_token: Default::default(),
            ident,
            enum_token: syn::token::Enum{ span: Span::call_site() },
            variants: Punctuated::from_iter(variants_iter)
        }.to_token_stream()
    }
}

pub(crate) fn extract_and_format_derive(empty_struct: &mut ItemStruct) -> Result<Attribute> {
    let pos = empty_struct.attrs.iter().position(|a| a.path.is_ident("derive") && a.style == AttrStyle::Outer);

    let drives = {
        let mut d: HashSet<Type> = if let Some(i) = pos {
            let existing_drive = empty_struct.attrs.remove(i);
            match parse::<syn::Type>(TokenStream1::from(existing_drive.tokens))? {
                Type::Paren(p) => {
                    let mut h = HashSet::new();
                    h.insert(*p.elem);
                    h
                }
                Type::Tuple(t) => {
                    t.elems.into_iter().collect()
                }
                t => {
                    return Err(
                        parse::Error::new(t.span(), "`#[gen_hid_descriptor]` unrecognized derive")
                    )
                }
            }
        } else {
            HashSet::new()
        };

        d.insert(parse_str("Deserialize").unwrap());
        d.insert(parse_str("Serialize").unwrap());

        d
    };

    let mut attrs_ts = TokenStream::new();

    TypeTuple {
        paren_token: Default::default(),
        elems: Punctuated::from_iter(drives)
    }.to_tokens(&mut attrs_ts);

    Ok(Attribute {
        pound_token: syn::token::Pound { spans: [Span::call_site()] },
        style: AttrStyle::Outer,
        bracket_token: Default::default(),
        path: parse_str("derive").unwrap(),
        tokens: attrs_ts
    })
}