extern crate usbd_hid_descriptors;

use syn::{parse, Field, Fields, Type, Expr, Result, Ident, ExprLit, Lit, TypePath};
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
    let (p, size) = parse_type(&field, ft)?;

    if p.path.segments.len() != 1 {
        return Err(
            parse::Error::new(field.ident.unwrap().span(), "`#[gen_hid_descriptor]` internal error when unwrapping type")
        );
    }
    let type_ident = p.path.segments[0].ident.clone();

    let type_str = type_ident.to_string();
    let (sign, size_str) = type_str.as_str().split_at(1);
    let bit_width = size_str.parse();
    let type_setter: Option<fn(&mut ReportUnaryField, usize)> = match sign {
        "u" => Some(set_unsigned_unary_item),
        "i" => Some(set_signed_unary_item),
        &_ => None
    };

    if bit_width.is_err() || type_setter.is_none() {
        return Err(
            parse::Error::new(type_ident.span(), "`#[gen_hid_descriptor]` type not supported")
        )
    }
    let bit_width = bit_width.unwrap();

    if bit_width >= 64 {
        return Err(
            parse::Error::new(type_ident.span(), "`#[gen_hid_descriptor]` integer larger than 64 is not supported in ssmarshal")
        )
    }

    let mut output = unary_item(field.ident.clone().unwrap(), item.kind, bit_width);

    if let Some(want_bits) = item.want_bits {  // bitpack
        output.descriptor_item.logical_minimum = 0;
        output.descriptor_item.logical_maximum = 1;
        output.descriptor_item.report_count = want_bits;
        output.descriptor_item.report_size = 1;
        let width = output.bit_width * size;
        if width < want_bits as usize {
            return Err(
                parse::Error::new(field.ident.unwrap().span(), format!("`#[gen_hid_descriptor]` not enough space, missing {} bit(s)", want_bits as usize - width))
            )
        }
        let remaining_bits = width as u16 - want_bits;
        if remaining_bits > 0 {
            output.descriptor_item.padding_bits = Some(remaining_bits);
        }
    } else { // array of reports
        type_setter.unwrap()(&mut output, bit_width);
        output.descriptor_item.report_count *= size as u16;
    }

    Ok(output)
}

fn parse_type(field: &Field, ft: Type) -> Result<(TypePath, usize)> {
    match ft {
        Type::Array(a) => {
            let mut size: usize = 0;

            if let Expr::Lit(ExprLit { lit, .. }) = a.len {
                if let Lit::Int(lit) = lit {
                    if let Ok(num) = lit.base10_parse::<usize>() {
                        size = num;
                    }
                }
            }
            if size == 0 {
                Err(
                    parse::Error::new(field.ident.as_ref().unwrap().span(), "`#[gen_hid_descriptor]` array has invalid length")
                )
            } else {
                Ok((parse_type(&field, *a.elem)?.0, size))
            }
        }
        Type::Path(p) => {
            Ok((p, 1))
        }
        _ => {
            Err(
                parse::Error::new(field.ident.as_ref().unwrap().span(),"`#[gen_hid_descriptor]` cannot handle field type")
            )
        }
    }
}

fn set_signed_unary_item(out: &mut ReportUnaryField, bit_width: usize) {
    let bound = 2u32.pow((bit_width-1) as u32) as isize - 1;
    out.descriptor_item.logical_minimum = -bound;
    out.descriptor_item.logical_maximum = bound;
}

fn set_unsigned_unary_item(out: &mut ReportUnaryField, bit_width: usize) {
    out.descriptor_item.logical_minimum = 0;
    out.descriptor_item.logical_maximum = 2u32.pow(bit_width as u32) as isize - 1;
}

fn unary_item(id: Ident, kind: MainItemKind, bit_width: usize) -> ReportUnaryField {
    ReportUnaryField{
        ident: id,
        bit_width,
        descriptor_item: MainItem{
            kind,
            logical_minimum: 0,
            logical_maximum: 0,
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
    panic!("internal error: could not find field {} which should exist", name)
}
