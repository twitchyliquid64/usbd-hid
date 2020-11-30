use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::{Fields, FieldsNamed, ItemStruct, parse, Result};
use syn::punctuated::Punctuated;
use usbd_hid_descriptors::MainItemKind;

use crate::item::ReportUnaryField;

pub(crate) fn wrap_struct(item: ItemStruct) -> TokenStream {
    quote! {
        #[derive(Debug, Clone, Copy, Serialize, Deserialize)]
        #[repr(C, packed)]
        #item
    }
}

pub(crate) fn filter_struct_fields(
    orig: &ItemStruct,
    fields: &Vec<ReportUnaryField>,
    keep: MainItemKind,
) -> Result<Option<ItemStruct>> {
    let mut out = orig.clone();
    if let Fields::Named(fns) = out.fields {
        let FieldsNamed { brace_token, named } = fns;

        let mut filtered = Punctuated::new();

        let keeps: Vec<Ident> = fields
            .iter()
            .filter_map(|f| {
                if f.descriptor_item.kind == keep {
                    Some(f.ident.clone())
                } else {
                    None
                }
            })
            .collect();

        for f in named {
            if f.ident
                .as_ref()
                .map(|i| keeps.contains(&i))
                .unwrap_or(false)
            {
                filtered.push(f);
            }
        }

        return if filtered.is_empty() {
            Ok(None)
        } else {
            out.fields = Fields::Named(FieldsNamed {
                brace_token,
                named: filtered,
            });
            Ok(Some(out))
        };
    }
    Err(parse::Error::new(
        out.ident.span(),
        "`#[gen_hid_descriptor]` internal error when generating type",
    ))
}
