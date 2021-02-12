use proc_macro2::{TokenStream};
use quote::quote;
use syn::{Fields, FieldsNamed, ItemStruct, parse, Result};

pub(crate) fn wrap_struct(item: ItemStruct) -> TokenStream {
    quote! {
        #[derive(Debug, Clone, Copy, Serialize, Deserialize)]
        #[repr(C, packed)]
        #item
    }
}

pub(crate) fn read_struct(orig: &ItemStruct) -> Result<(FieldsNamed, ItemStruct)> {
    let out = orig.clone();
    if let Fields::Named(named_fields) = out.fields {
        Ok((named_fields, ItemStruct {
            fields: Fields::Unit,
            ..out
        }))
    } else {
        Err(parse::Error::new(
            out.ident.span(),
            "`#[gen_hid_descriptor]` internal error when generating type",
        ))
    }
}