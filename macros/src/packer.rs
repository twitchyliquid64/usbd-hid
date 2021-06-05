extern crate usbd_hid_descriptors;
use usbd_hid_descriptors::*;

use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse, Ident, Index, Result};

use crate::spec::*;
use crate::item::*;

use core::iter::Extend;

pub fn uses_report_ids(spec: &Spec) -> bool {
    match spec {
        Spec::MainItem(_) => false,
        Spec::Collection(c) => {
            for (_, s) in &c.fields {
                if uses_report_ids(&s) {
                    return true;
                }
            }
            c.report_id.is_some()
        },
    }
}

fn make_unary_serialize_invocation(bits: usize, ident: Ident, signed: bool) -> TokenStream {
    match (bits, signed) {
        (8, false) => quote!({ s.serialize_element(&(self.#ident as u8))?; }),
        (16, false) => quote!({ s.serialize_element(&(self.#ident as u16))?; }),
        (32, false) => quote!({ s.serialize_element(&(self.#ident as u32))?; }),
        (8, true) => quote!({ s.serialize_element(&(self.#ident as i8))?; }),
        (16, true) => quote!({ s.serialize_element(&(self.#ident as i16))?; }),
        (32, true) => quote!({ s.serialize_element(&(self.#ident as i32))?; }),
        _ => quote!(),
    }
}

pub fn gen_serializer(fields: Vec<ReportUnaryField>, typ: MainItemKind) -> Result<TokenStream> {
    let mut elems = Vec::new();

    for field in fields {
        if field.descriptor_item.kind != typ {
            continue;
        }
        let signed = field.descriptor_item.logical_minimum < 0;

        let rc = match field.descriptor_item.report_size {
            1 => {
                if field.descriptor_item.report_count == 1 {
                    elems.push(make_unary_serialize_invocation(field.bit_width, field.ident.clone(), signed));
                } else {
                    let ident = field.ident.clone();
                    elems.push(quote!({ s.serialize_element(&self.#ident)?; }));
                }
                Ok(())
            },
            8 => { // u8 / i8
                if field.descriptor_item.report_count == 1 {
                    elems.push(make_unary_serialize_invocation(8, field.ident.clone(), signed));
                } else if field.descriptor_item.report_count <= 32 {
                    let ident = field.ident.clone();
                    elems.push(quote!({ s.serialize_element(&self.#ident)?; }));
                } else {
                    // XXX - don't attempt to serialize arrays larger than 32
                    //       (not supported by serde, yet)
                }
                Ok(())
            },
            16 | 32 => { // u16 / i16 / u32 / i32
                if field.descriptor_item.report_count == 1 {
                    elems.push(make_unary_serialize_invocation(field.descriptor_item.report_size as usize, field.ident.clone(), signed));
                    Ok(())
                } else {
                    Err(parse::Error::new(field.ident.span(),"Arrays of 16/32bit fields not supported"))
                }
            },
            _ => Err(
                parse::Error::new(field.ident.span(),"Unsupported report size for serialization")
            )
        };

        if let Err(e) = rc {
            return Err(e);
        }
    }

    let mut out = TokenStream::new();
    let idx = Index::from(elems.len());
    out.extend(elems);
    Ok(quote!({
        let mut s = serializer.serialize_tuple(#idx)?;
        #out
        s.end()
    }))
}
