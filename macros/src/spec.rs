extern crate usbd_hid_descriptors;

use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{parse, Attribute, Expr, ExprAssign, ExprPath, Path, Result, Token};
use syn::{Block, ExprBlock, ExprLit, ExprTuple, Lit, Stmt};

use std::collections::HashMap;
use std::string::String;
use usbd_hid_descriptors::*;
use syn::spanned::Spanned;
use syn::visit::Visit;

// Spec describes an item within a HID report.
#[derive(Debug, Clone)]
pub enum Spec {
    MainItem(ItemSpec),
    Collection(GroupSpec),
}

// ItemQuirks describes minor settings which can be tweaked for
// compatibility.
#[derive(Debug, Clone, Default, Copy)]
pub struct ItemQuirks {
    pub allow_short_form: bool,
}

// ItemSpec describes settings that apply to a single field.
#[derive(Debug, Clone, Default)]
pub struct ItemSpec {
    pub kind: MainItemKind,
    pub quirks: ItemQuirks,
    pub settings: Option<MainItemSetting>,
    pub want_bits: Option<u16>,
}

/// GroupSpec keeps track of consecutive fields with shared global
/// parameters. Fields are configured based on the attributes
/// used in the procedural macro's invocation.
#[derive(Debug, Clone, Default)]
pub struct GroupSpec {
    pub fields: HashMap<String, Spec>,
    pub field_order: Vec<String>,

    pub report_id: Option<u32>,
    pub usage_page: Option<u32>,
    pub collection: Option<u32>,
    pub logical_min: Option<u32>,

    // Local items
    pub usage: Vec<u32>,
    pub usage_min: Option<u32>,
    pub usage_max: Option<u32>,
}

impl GroupSpec {
    pub fn set_item(
        &mut self,
        name: String,
        item_kind: MainItemKind,
        settings: Option<MainItemSetting>,
        bits: Option<u16>,
        quirks: ItemQuirks,
    ) {
        if let Some(field) = self.fields.get_mut(&name) {
            if let Spec::MainItem(field) = field {
                field.kind = item_kind;
                field.settings = settings;
                field.want_bits = bits;
            }
        } else {
            self.fields.insert(
                name.clone(),
                Spec::MainItem(ItemSpec {
                    kind: item_kind,
                    settings: settings,
                    want_bits: bits,
                    quirks: quirks,
                    ..Default::default()
                }),
            );
            self.field_order.push(name);
        }
    }

    pub fn add_nested_group(&mut self, ng: GroupSpec) {
        let name = (0..self.fields.len() + 1).map(|_| "_").collect::<String>();
        self.fields.insert(name.clone(), Spec::Collection(ng));
        self.field_order.push(name);
    }

    pub fn get(&self, name: String) -> Option<&Spec> {
        self.fields.get(&name)
    }

    pub fn try_set_attr(&mut self, input: ParseStream, name: String, val: u32) -> Result<()> {
        match name.as_str() {
            "report_id" => {
                self.report_id = Some(val);
                Ok(())
            }
            "usage_page" => {
                self.usage_page = Some(val);
                Ok(())
            }
            "collection" => {
                self.collection = Some(val);
                Ok(())
            }
            // Local items.
            "usage" => {
                self.usage.push(val);
                Ok(())
            }
            "usage_min" => {
                self.usage_min = Some(val);
                Ok(())
            }
            "usage_max" => {
                self.usage_max = Some(val);
                Ok(())
            }
            "logical_min" => {
                self.logical_min = Some(val);
                Ok(())
            }
            _ => Err(parse::Error::new(
                input.span(),
                format!(
                    "`#[gen_hid_descriptor]` unknown group spec key: {}",
                    name.clone()
                ),
            )),
        }
    }
}

impl IntoIterator for GroupSpec {
    type Item = String;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.field_order.into_iter()
    }
}

pub fn try_resolve_constant(key_name: String, path: String) -> Option<u32> {
    match (key_name.as_str(), path.as_str()) {
        ("collection", "PHYSICAL") => Some(0x0),
        ("collection", "APPLICATION") => Some(0x1),
        ("collection", "LOGICAL") => Some(0x2),
        ("collection", "REPORT") => Some(0x3),
        ("collection", "NAMED_ARRAY") => Some(0x4),
        ("collection", "USAGE_SWITCH") => Some(0x5),
        ("collection", "USAGE_MODIFIER") => Some(0x06),

        ("usage_page", "UNDEFINED") => Some(0x00),
        ("usage_page", "GENERIC_DESKTOP") => Some(0x01),
        ("usage_page", "SIMULATION_CONTROLS") => Some(0x02),
        ("usage_page", "VR_CONTROLS") => Some(0x03),
        ("usage_page", "SPORT_CONTROLS") => Some(0x04),
        ("usage_page", "GAME_CONTROLS") => Some(0x05),
        ("usage_page", "GENERIC_DEVICE_CONTROLS") => Some(0x06),
        ("usage_page", "KEYBOARD") => Some(0x07),
        ("usage_page", "LEDS") => Some(0x08),
        ("usage_page", "BUTTON") => Some(0x09),
        ("usage_page", "ORDINAL") => Some(0x0A),
        ("usage_page", "TELEPHONY") => Some(0x0B),
        ("usage_page", "CONSUMER") => Some(0x0C),
        ("usage_page", "DIGITIZER") => Some(0x0D),
        ("usage_page", "ALPHANUMERIC_DISPLAY") => Some(0x14),
        ("usage_page", "BARCODE_SCANNER") => Some(0x8C),
        ("usage_page", "VENDOR_DEFINED_START") => Some(0xFF00),
        ("usage_page", "VENDOR_DEFINED_END") => Some(0xFFFF),

        // Desktop usage_page usage ID's.
        ("usage", "POINTER") => Some(0x01),
        ("usage", "MOUSE") => Some(0x02),
        ("usage", "JOYSTICK") => Some(0x04),
        ("usage", "GAMEPAD") => Some(0x05),
        ("usage", "KEYBOARD") => Some(0x06),
        ("usage", "KEYPAD") => Some(0x07),
        ("usage", "MULTI_AXIS_CONTROLLER") => Some(0x08),
        ("usage", "X") | ("usage_min", "X") | ("usage_max", "X") => Some(0x30),
        ("usage", "Y") | ("usage_min", "Y") | ("usage_max", "Y") => Some(0x31),
        ("usage", "Z") | ("usage_min", "Z") | ("usage_max", "Z") => Some(0x32),
        ("usage", "WHEEL") => Some(0x38),
        ("usage", "SYSTEM_CONTROL") => Some(0x80),

        // LED usage_page usage ID's.
        ("usage", "NUM_LOCK") => Some(0x01),
        ("usage", "CAPS_LOCK") => Some(0x02),
        ("usage", "SCROLL_LOCK") => Some(0x03),
        ("usage", "POWER") => Some(0x06),
        ("usage", "SHIFT") => Some(0x07),
        ("usage", "MUTE") => Some(0x09),
        ("usage", "RING") => Some(0x18),

        // Button usage_page usage ID's.
        ("usage", "BUTTON_NONE") => Some(0x00),
        ("usage", "BUTTON_1") | ("usage_min", "BUTTON_1") => Some(0x01),
        ("usage", "BUTTON_2") => Some(0x02),
        ("usage", "BUTTON_3") | ("usage_max", "BUTTON_3") => Some(0x03),
        ("usage", "BUTTON_4") | ("usage_max", "BUTTON_4") => Some(0x04),
        ("usage", "BUTTON_5") => Some(0x05),
        ("usage", "BUTTON_6") => Some(0x06),
        ("usage", "BUTTON_7") => Some(0x07),
        ("usage", "BUTTON_8") | ("usage_max", "BUTTON_8") => Some(0x08),

        // Alpha-numeric display usage_page usage ID's.
        ("usage", "CLEAR_DISPLAY") => Some(0x25),
        ("usage", "DISPLAY_ENABLE") => Some(0x26),
        ("usage", "CHARACTER_REPORT") => Some(0x2B),
        ("usage", "CHARACTER_DATA") => Some(0x2C),

        // Consumer usage
        ("usage", "CONSUMER_CONTROL") => Some(0x01),
        ("usage", "NUMERIC_KEYPAD") => Some(0x02),
        ("usage", "PROGRAMMABLE_BUTTONS") => Some(0x03),
        ("usage", "MICROPHONE") => Some(0x04),
        ("usage", "HEADPHONE") => Some(0x05),
        ("usage", "GRAPHIC_EQUALIZER") => Some(0x06),
        ("usage", "AC_PAN") => Some(0x0238),

        (_, _) => None,
    }
}

fn parse_group_spec(input: ParseStream, field: Expr) -> Result<GroupSpec> {
    let mut collection_attrs: Vec<(String, u32)> = vec![];

    if let Expr::Assign(ExprAssign { left, .. }) = field.clone() {
        if let Expr::Tuple(ExprTuple { elems, .. }) = *left {
            for elem in elems {
                let group_attr = maybe_parse_kv_lhs(elem.clone());
                if group_attr.is_none() || group_attr.clone().unwrap().len() != 1 {
                    return Err(parse::Error::new(
                        input.span(),
                        "`#[gen_hid_descriptor]` group spec key can only have a single element",
                    ));
                }
                let group_attr = group_attr.unwrap()[0].clone();

                let mut val: Option<u32> = None;
                if let Expr::Assign(ExprAssign { right, .. }) = elem {
                    if let Expr::Lit(ExprLit { lit, .. }) = *right {
                        if let Lit::Int(lit) = lit {
                            if let Ok(num) = lit.base10_parse::<u32>() {
                                val = Some(num);
                            }
                        }
                    } else if let Expr::Path(ExprPath {
                        path: Path { segments, .. },
                        ..
                    }) = *right
                    {
                        val = try_resolve_constant(
                            group_attr.clone(),
                            quote! { #segments }.to_string(),
                        );
                        if val.is_none() {
                            return Err(parse::Error::new(
                                input.span(),
                                format!(
                                    "`#[gen_hid_descriptor]` unrecognized constant: {}",
                                    quote! { #segments }.to_string()
                                ),
                            ));
                        }
                    }
                }
                if val.is_none() {
                    return Err(parse::Error::new(input.span(), "`#[gen_hid_descriptor]` group spec attribute value must be a numeric literal or recognized constant"));
                }
                collection_attrs.push((group_attr, val.unwrap()));
            }
        }
    }
    if collection_attrs.len() == 0 {
        return Err(parse::Error::new(
            input.span(),
            "`#[gen_hid_descriptor]` group spec lhs must contain value pairs",
        ));
    }
    let mut out = GroupSpec {
        ..Default::default()
    };
    for (key, val) in collection_attrs {
        if let Err(e) = out.try_set_attr(input, key, val) {
            return Err(e);
        }
    }

    // Match out the item kind on the right of the equals.
    if let Expr::Assign(ExprAssign { right, .. }) = field {
        if let Expr::Block(ExprBlock {
            block: Block { stmts, .. },
            ..
        }) = *right
        {
            for stmt in stmts {
                if let Stmt::Expr(e) = stmt {
                    if let Err(e) = out.from_field(input, e) {
                        return Err(e);
                    }
                } else if let Stmt::Semi(e, _) = stmt {
                    if let Err(e) = out.from_field(input, e) {
                        return Err(e);
                    }
                } else {
                    return Err(parse::Error::new(input.span(), "`#[gen_hid_descriptor]` group spec body can only contain semicolon-separated fields"));
                }
            }
        } else {
            return Err(parse::Error::new(
                right.span(),
                "`#[gen_hid_descriptor]` group spec rhs must be a block (did you miss a `,`)",
            ))
        };
    };
    Ok(out)
}

/// maybe_parse_kv_lhs returns a vector of :: separated idents.
fn maybe_parse_kv_lhs(field: Expr) -> Option<Vec<String>> {
    if let Expr::Assign(ExprAssign { left, .. }) = field {
        if let Expr::Path(ExprPath {
            path: Path { segments, .. },
            ..
        }) = *left
        {
            let mut out: Vec<String> = vec![];
            for s in segments {
                out.push(s.ident.to_string());
            }
            return Some(out);
        }
    }
    return None;
}

fn parse_item_attrs(attrs: Vec<Attribute>) -> (Option<MainItemSetting>, Option<u16>, ItemQuirks) {
    let mut out: MainItemSetting = MainItemSetting { 0: 0 };
    let mut had_settings: bool = false;
    let mut packed_bits: Option<u16> = None;
    let mut quirks: ItemQuirks = ItemQuirks{ ..Default::default() };

    for attr in attrs {
        match attr.path.segments[0].ident.to_string().as_str() {
            "packed_bits" => {
                for tok in attr.tokens {
                    if let proc_macro2::TokenTree::Literal(lit) = tok {
                        if let Ok(num) = lit.to_string().parse::<u16>() {
                            packed_bits = Some(num);
                            break;
                        }
                    }
                }
                if packed_bits.is_none() {
                    println!("WARNING!: bitfield attribute specified but failed to read number of bits from token!");
                }
            },

            "item_settings" => {
                had_settings = true;
                for setting in attr.tokens {
                    if let proc_macro2::TokenTree::Ident(id) = setting {
                        match id.to_string().as_str() {
                            "constant" => out.set_constant(true),
                            "data" => out.set_constant(false),

                            "variable" => out.set_variable(true),
                            "array" => out.set_variable(false),

                            "relative" => out.set_relative(true),
                            "absolute" => out.set_relative(false),

                            "wrap" => out.set_wrap(true),
                            "no_wrap" => out.set_wrap(false),

                            "non_linear" => out.set_non_linear(true),
                            "linear" => out.set_non_linear(false),

                            "no_preferred" => out.set_no_preferred_state(true),
                            "preferred" => out.set_no_preferred_state(false),

                            "null" => out.set_has_null_state(true),
                            "not_null" => out.set_has_null_state(false),

                            "volatile" => out.set_volatile(true),
                            "not_volatile" => out.set_volatile(false),
                            p => println!("WARNING: Unknown item_settings parameter: {}", p),
                        }
                    }
                }
            },

            "quirks" => {
                for setting in attr.tokens {
                    if let proc_macro2::TokenTree::Ident(id) = setting {
                        match id.to_string().as_str() {
                            "allow_short" => quirks.allow_short_form = true,
                            p => println!("WARNING: Unknown item_settings parameter: {}", p),
                        }
                    }
                }
            },

            p => {
                println!("WARNING: Unknown item attribute: {}", p);
            }
        }
    }

    if had_settings {
        return (Some(out), packed_bits, quirks);
    }
    (None, packed_bits, quirks)
}

// maybe_parse_kv tries to parse an expression like 'blah=blah'.
fn maybe_parse_kv(field: Expr) -> Option<(String, String, Option<MainItemSetting>, Option<u16>, ItemQuirks)> {
    // Match out the identifier on the left of the equals.
    let name: String;
    if let Some(lhs) = maybe_parse_kv_lhs(field.clone()) {
        if lhs.len() != 1 {
            return None;
        }
        name = lhs[0].clone();
    } else {
        return None;
    }

    // Decode item settings.
    let item_settings = if let Some(attrs) = AttributeCollector::all(&field) {
        parse_item_attrs(attrs)
    } else {
        (None, None, ItemQuirks::default())
    };

    // Match out the item kind on the right of the equals.
    let mut val: Option<String> = None;
    if let Expr::Assign(ExprAssign { right, .. }) = field {
        if let Expr::Path(ExprPath {
            path: Path { segments, .. },
            ..
        }) = *right
        {
            val = Some(segments[0].ident.clone().to_string());
        }
    };
    if val.is_none() {
        return None;
    }

    Some((name, val.unwrap(), item_settings.0, item_settings.1, item_settings.2))
}

struct AttributeCollector(Vec<Attribute>);

impl <'ast> AttributeCollector {

    fn new() -> Self {
        Self(vec![])
    }

    /// Recursively finds all Attributes contained by an Expr.
    /// Returns None when no attributes are found.
    pub fn all(expr: &'ast Expr) -> Option<Vec<Attribute>> {
        let mut visitor = Self::new();
        visitor.visit_expr(expr);
        if visitor.0.is_empty() {
            None
        } else {
            Some(visitor.0)
        }
    }
}

impl <'ast> Visit<'ast> for AttributeCollector {
    fn visit_attribute(&mut self, node: &'ast Attribute) {
        self.0.push(node.to_owned());
    }
}

impl Parse for GroupSpec {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut out = GroupSpec {
            ..Default::default()
        };
        let fields: Punctuated<Expr, Token![,]> = input.parse_terminated(Expr::parse)?;
        if fields.len() == 0 {
            return Err(parse::Error::new(
                input.span(),
                "`#[gen_hid_descriptor]` expected information about the HID report",
            ));
        }
        for field in fields {
            if let Err(e) = out.from_field(input, field) {
                return Err(e);
            }
        }
        Ok(out)
    }
}

impl GroupSpec {
    fn from_field(&mut self, input: ParseStream, field: Expr) -> Result<()> {
        if let Some(i) = maybe_parse_kv(field.clone()) {
            let (name, item_kind, settings, bits, quirks) = i;
            self.set_item(name, item_kind.into(), settings, bits, quirks);
            return Ok(());
        };
        match parse_group_spec(input, field.clone()) {
            Err(e) => return Err(e),
            Ok(g) => self.add_nested_group(g),
        };
        Ok(())
    }
}
