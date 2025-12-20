extern crate usbd_hid_descriptors;
use usbd_hid_descriptors::*;

use alloc::vec::Vec;

use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse, Result};

use crate::item::*;
use crate::spec::*;

pub fn uses_report_ids(spec: &Spec) -> bool {
    match spec {
        Spec::MainItem(_) => false,
        Spec::Collection(c) => {
            for s in c.fields.values() {
                if uses_report_ids(s) {
                    return true;
                }
            }
            c.report_id.is_some()
        }
    }
}

pub fn gen_serializer(fields: Vec<ReportUnaryField>, typ: MainItemKind) -> Result<TokenStream> {
    let mut elems: Vec<TokenStream> = Vec::new();

    let mut offset = 0;
    for field in fields {
        if field.descriptor_item.kind != typ {
            continue;
        }
        let ident = &field.ident;

        let report_size = field.descriptor_item.report_size;
        let report_count = field.descriptor_item.report_count;
        let rc = match report_size {
            1 => {
                // bitfield, stored in a u8 / u16 / u32
                elems.push(write_value(field.bit_width, offset, quote!(#ident)));
                offset += field.bit_width / 8;
                Ok(())
            }
            8 => {
                // u8 / i8
                if report_count == 1 {
                    elems.push(write_value(8, offset, quote!(#ident)));
                    offset += 1;
                } else {
                    // byte array
                    for i in 0..report_count as usize {
                        elems.push(write_value(8, offset, quote!(#ident[#i])));
                        offset += 1;
                    }
                }
                Ok(())
            }
            16 | 32 => {
                // u16 / i16 / u32 / i32
                if report_count == 1 {
                    elems.push(write_value(report_size as usize, offset, quote!(#ident)));
                    offset += report_size as usize / 8;
                    Ok(())
                } else {
                    Err(parse::Error::new(
                        field.ident.span(),
                        "Arrays of 16/32bit fields not supported",
                    ))
                }
            }
            _ => Err(parse::Error::new(
                field.ident.span(),
                "Unsupported report size for serialization",
            )),
        };

        rc?;
    }

    Ok(quote!({
        if #offset > buffer.len() {
            return Err(usbd_hid::descriptor::BufferOverflow);
        }
        #( #elems; )*
        Ok(#offset)
    }))
}

fn write_value(bit_width: usize, offset: usize, ident: TokenStream) -> TokenStream {
    if bit_width == 8 {
        quote!(buffer[#offset] = self.#ident as u8)
    } else if bit_width == 16 {
        quote!(
            let [v1, v2] = self.#ident.to_le_bytes();
            buffer[#offset] = v1;
            buffer[#offset + 1] = v2;
        )
    } else if bit_width == 32 {
        quote!(
            let [v1, v2, v3, v4] = (self.#ident as u32).to_le_bytes();
            buffer[#offset] = v1;
            buffer[#offset + 1] = v2;
            buffer[#offset + 2] = v3;
            buffer[#offset + 3] = v4;
        )
    } else {
        unreachable!("Earlier diagnostic check catches this")
    }
}
