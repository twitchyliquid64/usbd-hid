extern crate usbd_hid_descriptors;

use syn::{parse, Field, Fields, Type, Expr, Result, Ident, ExprLit, Lit};
use usbd_hid_descriptors::*;
use crate::spec::*;

// MainItem describes all the mandatory data points of a Main item.
#[derive(Debug, Default, Clone)]
pub struct MainItem {
    pub kind: MainItemKind,
    pub logical_minimum: isize,
    pub logical_maximum: isize,
    pub report_count: u16,
    pub report_size: u16,
    pub padding_bits: Option<u16>,
}

#[derive(Debug, Clone)]
pub struct ReportUnaryField {
    pub bit_width: usize,
    pub descriptor_item: MainItem,
    pub ident: Ident,
}

/// analyze_field constructs a main item from an item spec & field.
pub fn analyze_field(field: Field, ft: Type, item: &ItemSpec) -> Result<ReportUnaryField> {
    if let Type::Path(p) = ft {
        if p.path.segments.len() != 1 {
            return Err(
                parse::Error::new(field.ident.unwrap().span(),"`#[gen_hid_descriptor]` internal error when unwrapping type")
            );
        }
        let type_ident = p.path.segments[0].ident.clone();
        let mut output = match type_ident.to_string().as_str() {
            "u8" => unsigned_unary_item(field.ident.clone().unwrap(), item.kind, 8),
            "u16" => unsigned_unary_item(field.ident.clone().unwrap(), item.kind, 16),
            "u32" => unsigned_unary_item(field.ident.clone().unwrap(), item.kind, 32),
            "i8" => signed_unary_item(field.ident.clone().unwrap(), item.kind, 8),
            "i16" => signed_unary_item(field.ident.clone().unwrap(), item.kind, 16),
            "i32" => signed_unary_item(field.ident.clone().unwrap(), item.kind, 32),
            _ => return Err(
                    parse::Error::new(type_ident.span(),"`#[gen_hid_descriptor]` type not supported")
            ),
        };
        if let Some(want_bits) = item.want_bits {
            output.descriptor_item.logical_minimum = 0;
            output.descriptor_item.logical_maximum = 1;
            output.descriptor_item.report_count = want_bits;
            output.descriptor_item.report_size = 1;
            let remaining_bits = output.bit_width as u16 - want_bits;
            if remaining_bits > 0 {
                output.descriptor_item.padding_bits = Some(remaining_bits);
            }
        };
        Ok(output)
    } else if let Type::Array(a) = ft {
        let mut size: usize = 0;
        if let Expr::Lit(ExprLit{lit, ..}) = a.len {
            if let Lit::Int(lit) = lit {
                if let Ok(num) = lit.base10_parse::<usize>() {
                    size = num;
                }
            }
        }
        if size == 0 {
            return Err(
                parse::Error::new(field.ident.unwrap().span(),"`#[gen_hid_descriptor]` array has invalid length")
            );
        }
        // Recurse for the native data type, then mutate it to account for the repetition.
        match analyze_field(field, *a.elem, item) {
            Err(e) => Err(e),
            Ok(mut f) => {
                    f.descriptor_item.report_count = f.descriptor_item.report_count * size as u16;
                    Ok(f)
            },
        }
    } else {
        Err(
            parse::Error::new(field.ident.unwrap().span(),"`#[gen_hid_descriptor]` cannot handle field type")
        )
    }
}

fn signed_unary_item(id: Ident, kind: MainItemKind, bit_width: usize) -> ReportUnaryField {
    let bound = 2u32.pow((bit_width-1) as u32) as isize - 1;
    ReportUnaryField{
        ident: id,
        bit_width: bit_width,
        descriptor_item: MainItem{
            kind: kind,
            logical_minimum: -bound,
            logical_maximum: bound,
            report_count: 1,
            report_size: bit_width as u16,
            padding_bits: None,
        },
    }
}

fn unsigned_unary_item(id: Ident, kind: MainItemKind, bit_width: usize) -> ReportUnaryField {
    ReportUnaryField{
        ident: id,
        bit_width: bit_width,
        descriptor_item: MainItem{
            kind: kind,
            logical_minimum: 0,
            logical_maximum: 2u32.pow(bit_width as u32) as isize - 1,
            report_count: 1,
            report_size: bit_width as u16,
            padding_bits: None,
        },
    }
}

pub fn field_decl(fields: &Fields, name: String) -> Field {
    for field in fields {
        let ident = field.ident.clone().unwrap().to_string();
        if ident == name {
            return field.clone();
        }
    }
    panic!(format!("internal error: could not find field {} which should exist", name))
}
