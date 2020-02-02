//! Internal implementation details of usbd-hid.

extern crate proc_macro;
extern crate usbd_hid_descriptors;

use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{parse, parse_macro_input, ItemStruct, Field, Fields, Type, Expr, Path};
use syn::{Result, Token, ExprAssign, ExprPath, Pat, PatSlice, Ident};
use syn::{ExprTuple, ExprLit, Lit, ExprBlock, Block, Stmt};
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::token::Bracket;

use std::string::String;
use std::collections::HashMap;
use usbd_hid_descriptors::*;
use byteorder::{ByteOrder, LittleEndian};

/// Attribute to generate a HID descriptor
///
/// You are expected to provide two inputs to this generator:
///
///  - A struct of named fields (which follows the `gen_hid_descriptor` attribute)
///  - A specially-formatted section describing the properties of the descriptor (this
///    section must be provided as arguments to the `gen_hid_descriptor()` attribute)
///
/// The generated HID descriptor will be available as a `&[u8]` by calling
/// `YourStructType::desc()`. `YourStructType` also now implements `SerializedDescriptor`.
///
/// As long as a descriptor describes only input or output types, and a report ID is
/// not used, the wire format for transmitting and recieving the data described by the
/// descriptor is simply the packed representation of the struct itself.
/// Where report ID's are used anywhere in the descriptor, you must prepend the relevant
/// report ID to the packed representation of the struct prior to transmission.
///
/// If inputs and outputs are mixed within the same HID descriptor, then only the struct
/// fields used in that direction can be present in a payload being transmitted in that
/// direction.
///
/// # Examples
///
/// - Custom 32-octet array, sent from device to host
///
/// ``` no_run
/// #[gen_hid_descriptor(
///     (collection = APPLICATION, usage_page = VENDOR_DEFINED_START, usage = 0x01) = {
///         buff=input;
///     }
/// )]
/// struct CustomInputReport {
///     buff: [u8; 32],
/// }
/// ```
///
/// - Custom input / output, sent in either direction
///
/// ``` no_run
/// #[gen_hid_descriptor(
///     (collection = APPLICATION, usage_page = VENDOR_DEFINED_START, usage = 0x01) = {
///         input_buffer=input;
///         output_buffer=output;
///     }
/// )]
/// struct CustomBidirectionalReport {
///     input_buffer: [u8; 32],
///     output_buffer: [u8; 32],
/// }
/// ```
///
/// Because both inputs and outputs are used, the data format when sending / recieving is the
/// 32 bytes in the relevant direction, **NOT** the full 64 bytes contained within the struct.
///
/// - Packed bitfields
///
/// ``` no_run
/// #[gen_hid_descriptor(
///     (report_id = 0x01,) = {
///         #[packed_bits 3] f1=input;
///         #[packed_bits 9] f2=input;
///     }
/// )]
/// struct CustomPackedBits {
///     f1: u8,
///     f2: u16,
/// }
/// ```
///
/// Because the `#[packed_bits]` sub-attribute was used, the two input fields specified are
/// interpreted as packed bits. As such, `f1` describes 3 boolean inputs, and `f2` describes
/// 9 boolean inputs. Padding constants are automatically generated.
///
/// The `#[packed_bits <num bits>]` feature is intended to be used for describing button presses.
///
/// - Customizing the settings on a report item
///
/// ``` no_run
/// #[gen_hid_descriptor(
///     (collection = APPLICATION, usage_page = VENDOR_DEFINED_START, usage = 0x01) = {
///         (usage_min = X, usage_max = Y) = {
///             #[item_settings data,variable,relative] x=input;
///             #[item_settings data,variable,relative] y=input;
///         };
///     }
/// )]
/// struct CustomCoords {
///     x: i8,
///     y: i8,
/// }
/// ```
///
/// The above example describes a report which sends X & Y co-ordinates. As indicated in
/// the `#[item_settings]` sub-attribute, the individual inputs are described as:
///
///  - Datapoints (`data`) - as opposed to constant
///  - Variable (`variable`) - as opposed to an array
///  - Relative (`relative`) - as opposed to absolute
///
/// # Supported struct types
///
/// The struct following the attribute must consist entirely of named fields, using
/// only types enumerated below, or fixed-size arrays of the types enumerated below.
///
///  - u8 / i8
///  - u16 / i16
///  - u32 / i32
///
/// `LOGICAL_MINIMUM` & `LOGICAL_MAXIMUM` are automatically set in the descriptor, based
/// on the type & whether `#[packed_bits]` was set on the field or not.
///
/// # Descriptor format
///
/// The parameters of the HID descriptor should be provided as arguments to the attribute.
/// The arguments should follow the basic form:
///
/// ```
/// #[gen_hid_descriptor(
///     <collection-spec> OR <item-spec>;
///     <collection-spec> OR <item-spec>;
///     ...
///     <collection-spec> OR <item-spec>
/// )]
/// ```
///
/// ## `collection-spec`:
///
/// ```
///     (parameter = <constant or 0xxxx>, ...) = {
///         <collection-spec> OR <item-spec>;
///         ...
///     }
/// ```
///
/// Note: All collection specs must end in a semicolon, except the top-level one.
///
/// Note: Parameters are a tuple, so make sure you have a trailing comma if you only have one
/// parameter.
///
/// The valid parameters are `collection`, `usage_page`, `usage`, `usage_min`, `usage_max`, and
/// `report_id`. These simply configure parameters that apply to contained items in the report.
/// Use of the `collection` parameter automatically creates a collection feature for all items
/// which are contained within it, and other parameters specified in the same collection-spec
/// apply to the collection, not directly to the elements of the collection (ie: defining a
/// collection + a usage generates a descriptor where the usage is set on the collection, not the
/// items contained within the collection).
///
/// ## `item-spec`:
///
/// ```
///     #[packed_bits <num_items>] #[item_settings <setting>,...] <fieldname>=input OR output;
/// ```
///
/// The two sub-attributes are both optional.
///
///   - `fieldname` refers to the name of a field within the struct. All fields must be specified.
///   - `input` fields are sent in reports from device to host. `output` fields are sent in reports
///     from host to device. This matches the terminology used in the USB & HID specifications.
///   - `packed_bits` configures the field as a set of `num_items` booleans rather than a number.
///     Any left over bits are automatically set as constants within the report. This is typically
///     used to implement buttons.
///   - `item_settings` describes settings on the input/output item, as enumerated in section
///     6.2.2.5 of the [HID specification, version 1.11](https://www.usb.org/sites/default/files/documents/hid1_11.pdf).
///     By default, all items are configured as `(Data,Var,Abs,No Wrap,Linear,Preferred State,No Null Position)`.

#[proc_macro_attribute]
pub fn gen_hid_descriptor(args: TokenStream, input: TokenStream) -> TokenStream {
    let decl = parse_macro_input!(input as ItemStruct);
    let spec = parse_macro_input!(args as GroupSpec);
    let ident = decl.ident.clone();

    // Error if the struct doesn't name its fields.
    match decl.clone().fields {
        Fields::Named(_) => (),
        _ => return parse::Error::new(ident.span(),"`#[gen_hid_descriptor]` type must name fields")
            .to_compile_error()
            .into(),
    };

    let descriptor = match compile(spec, &decl.fields){
        Ok(d) => d,
        Err(e) => return e.to_compile_error().into(),
    };
    // let descriptor_len = Index::from(size);
    let out = quote! {
        #[derive(Debug, Clone, Copy)]
        #[repr(C, packed)]
        #decl

        impl SerializedDescriptor for #ident {
            fn desc() -> &'static[u8] {
                &#descriptor
            }
        }
    };
    TokenStream::from(out)
}


// Spec describes an item within a HID report.
#[derive(Debug, Clone)]
enum Spec {
    MainItem(ItemSpec),
    Collection(GroupSpec),
}

// ItemSpec describes settings that apply to a single field.
#[derive(Debug, Clone, Default)]
struct ItemSpec {
    kind: MainItemKind,
    settings: Option<MainItemSetting>,
    want_bits: Option<u16>,
}

/// GroupSpec keeps track of consecutive fields with shared global
/// parameters. Fields are configured based on the attributes
/// used in the procedural macro's invocation.
#[derive(Debug, Clone, Default)]
struct GroupSpec {
    fields: HashMap<String, Spec>,
    field_order: Vec<String>,

    report_id: Option<u32>,
    usage_page: Option<u32>,
    collection: Option<u32>,

    // Local items
    usage: Option<u32>,
    usage_min: Option<u32>,
    usage_max: Option<u32>,
}

impl GroupSpec {
    fn set_item(&mut self, name: String, item_kind: MainItemKind, settings: Option<MainItemSetting>, bits: Option<u16>) {
        if let Some(field) = self.fields.get_mut(&name) {
            if let Spec::MainItem(field) = field {
                field.kind = item_kind;
                field.settings = settings;
                field.want_bits = bits;
            }
        } else {
            self.fields.insert(name.clone(), Spec::MainItem(ItemSpec{ kind: item_kind, settings: settings, want_bits: bits, ..Default::default() }));
            self.field_order.push(name);
        }
    }

    fn add_nested_group(&mut self, ng: GroupSpec) {
        let name = (0..self.fields.len()+1).map(|_| "_").collect::<String>();
        self.fields.insert(name.clone(), Spec::Collection(ng));
        self.field_order.push(name);
    }

    fn get(&self, name: String) -> Option<&Spec> {
        self.fields.get(&name)
    }

    fn try_set_attr(&mut self, input: ParseStream, name: String, val: u32) -> Result<()> {
        match name.as_str() {
            "report_id" => {
                self.report_id = Some(val);
                Ok(())
            },
            "usage_page" => {
                self.usage_page = Some(val);
                Ok(())
            },
            "collection" => {
                self.collection = Some(val);
                Ok(())
            },
            // Local items.
            "usage" => {
                self.usage = Some(val);
                Ok(())
            },
            "usage_min" => {
                self.usage_min = Some(val);
                Ok(())
            },
            "usage_max" => {
                self.usage_max = Some(val);
                Ok(())
            },
            _ => Err(parse::Error::new(input.span(), format!("`#[gen_hid_descriptor]` unknown group spec key: {}", name.clone()))),
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

fn try_resolve_constant(key_name: String, path: String) -> Option<u32> {
    match (key_name.as_str(), path.as_str()) {
        ("collection", "PHYSICAL") => Some(0x0),
        ("collection", "APPLICATION") => Some(0x1),
        ("collection", "LOGICAL") => Some(0x3),
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


        (_, _) => None,
    }
}

fn parse_group_spec(input: ParseStream, field: Expr) -> Result<GroupSpec> {
    let mut collection_attrs: Vec<(String, u32)> = vec![];

    if let Expr::Assign(ExprAssign {left, .. }) = field.clone() {
        if let Expr::Tuple(ExprTuple{elems, ..}) = *left {
            for elem in elems {
                let group_attr = maybe_parse_kv_lhs(elem.clone());
                if group_attr.is_none() || group_attr.clone().unwrap().len() != 1 {
                    return Err(parse::Error::new(input.span(), "`#[gen_hid_descriptor]` group spec key can only have a single element"));
                }
                let group_attr = group_attr.unwrap()[0].clone();

                let mut val: Option<u32> = None;
                if let Expr::Assign(ExprAssign{right, .. }) = elem {
                    if let Expr::Lit(ExprLit{lit, ..}) = *right {
                        if let Lit::Int(lit) = lit {
                            if let Ok(num) = lit.base10_parse::<u32>() {
                                val = Some(num);
                            }
                        }
                    } else if let Expr::Path(ExprPath{path: Path{segments, ..}, ..}) = *right {
                        val = try_resolve_constant(group_attr.clone(), quote! { #segments }.to_string());
                        if val.is_none() {
                            return Err(parse::Error::new(input.span(), format!("`#[gen_hid_descriptor]` unrecognized constant: {}", quote! { #segments }.to_string())));
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
        return Err(parse::Error::new(input.span(), "`#[gen_hid_descriptor]` group spec lhs must contain value pairs"));
    }
    let mut out = GroupSpec{ ..Default::default() };
    for (key, val) in collection_attrs {
        if let Err(e) = out.try_set_attr(input, key, val) {
            return Err(e);
        }
    }

    // Match out the item kind on the right of the equals.
    if let Expr::Assign(ExprAssign {right, .. }) = field {
        if let Expr::Block(ExprBlock{block: Block{stmts, ..}, ..}) = *right {
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
        };
    };
    Ok(out)
}

/// maybe_parse_kv_lhs returns a vector of :: separated idents.
fn maybe_parse_kv_lhs(field: Expr) -> Option<Vec<String>> {
    if let Expr::Assign(ExprAssign {left, .. }) = field {
        if let Expr::Path(ExprPath{path: Path{segments, ..}, ..}) = *left {
            let mut out: Vec<String> = vec![];
            for s in segments {
                out.push(s.ident.to_string());
            }
            return Some(out);
        }
    }
    return None;
}

fn parse_item_attrs(attrs: Vec<syn::Attribute>) -> (Option<MainItemSetting>, Option<u16>) {
    let mut out: MainItemSetting = MainItemSetting{ 0: 0 };
    let mut had_settings: bool = false;
    let mut packed_bits: Option<u16> = None;

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
            p => {
                println!("WARNING: Unknown item attribute: {}", p);
            },
        }
    }

    if had_settings {
        return (Some(out), packed_bits);
    }
    (None, packed_bits)
}

// maybe_parse_kv tries to parse an expression like 'blah=blah'.
fn maybe_parse_kv(field: Expr) -> Option<(String, String, Option<MainItemSetting>, Option<u16>)> {
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
    let item_settings = if let Expr::Assign(ExprAssign {attrs, .. }) = field.clone() {
        parse_item_attrs(attrs)
    } else {
        (None, None)
    };

    // Match out the item kind on the right of the equals.
    let mut val: Option<String> = None;
    if let Expr::Assign(ExprAssign {right, .. }) = field {
        if let Expr::Path(ExprPath{path: Path{segments, ..}, ..}) = *right {
            val = Some(segments[0].ident.clone().to_string());
        }
    };
    if val.is_none() {
        return None;
    }

    Some((name, val.unwrap(), item_settings.0, item_settings.1))
}

impl Parse for GroupSpec {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut out = GroupSpec { ..Default::default() };
        let fields: Punctuated<Expr, Token![,]> = input.parse_terminated(Expr::parse)?;
        if fields.len() == 0 {
            return Err(parse::Error::new(input.span(), "`#[gen_hid_descriptor]` expected information about the HID report"));
        }
        for field in fields {
            if let Err(e) =  out.from_field(input, field) {
                return Err(e);
            }
        }
        Ok(out)
    }
}

impl GroupSpec {
    fn from_field(&mut self, input: ParseStream, field: Expr) -> Result<()> {
        if let Some(i) = maybe_parse_kv(field.clone()) {
            let (name, item_kind, settings, bits) = i;
            self.set_item(name, item_kind.into(), settings, bits);
            return Ok(())
        };
        match parse_group_spec(input, field.clone()) {
            Err(e) => return Err(e),
            Ok(g) => self.add_nested_group(g),
        };
        Ok(())
    }
}

fn byte_literal(lit: u8) -> Pat {
    // print!("{:x} ", lit);
    // println!();
    Pat::Lit(
        syn::PatLit{
            attrs: vec![],
            expr: Box::new(
                Expr::Lit(
                    syn::ExprLit{
                        attrs: vec![],
                        lit: syn::Lit::Byte(syn::LitByte::new(lit, Span::call_site())),
                    }
                )
            ),
        }
    )
}

#[derive(Default)]
struct DescCompilation {
    // usage_page: u8,
    // usage: u8,
    // collection: Option<u8>,
    logical_minimum: Option<isize>,
    logical_maximum: Option<isize>,
    report_size: Option<u16>,
    report_count: Option<u16>,
}

impl DescCompilation {
    fn emit(&self, elems: &mut Punctuated<Pat, syn::token::Comma>, prefix: &mut ItemPrefix, buf: [u8; 4], signed: bool) {
        // println!("buf: {:?}", buf);
        if buf[1..4] == [0,0,0] && !(signed && buf[0] == 255) {
            prefix.set_byte_count(1);
            elems.push(byte_literal(prefix.0));
            elems.push(byte_literal(buf[0]));
        }
        else if buf[2..4] == [0,0] && !(signed && buf[1] == 255) {
            prefix.set_byte_count(2);
            elems.push(byte_literal(prefix.0));
            elems.push(byte_literal(buf[0]));
            elems.push(byte_literal(buf[1]));
        }
        else {
            prefix.set_byte_count(3);
            elems.push(byte_literal(prefix.0));
            elems.push(byte_literal(buf[0]));
            elems.push(byte_literal(buf[1]));
            elems.push(byte_literal(buf[2]));
            elems.push(byte_literal(buf[3]));
        }
        // println!("emitted {} data bytes", prefix.byte_count());
    }

    fn emit_item(&self, elems: &mut Punctuated<Pat, syn::token::Comma>, typ: u8, kind: u8, num: isize, signed: bool) {
        let mut prefix = ItemPrefix(0);
        prefix.set_tag(kind);
        prefix.set_type(typ);

        // TODO: Support long tags.

        // Section 6.2.2.4: An Input item could have a data size of zero (0)
        // bytes. In this case the value of each data bit for the item can be
        // assumed to be zero. This is functionally identical to using a item
        // tag that specifies a 4-byte data item followed by four zero bytes.
        let allow_short = typ == ItemType::Main.into() && kind == MainItemKind::Input.into();
        if allow_short && num == 0 {
            prefix.set_byte_count(0);
            elems.push(byte_literal(prefix.0));
            return;
        }

        let mut buf = [0; 4];
        LittleEndian::write_i32(&mut buf, num as i32);
        self.emit(elems, &mut prefix, buf, signed);
    }

    fn handle_globals(&mut self, elems: &mut Punctuated<Pat, syn::token::Comma>, item: MainItem) {
        if self.logical_minimum.is_none() || self.logical_minimum.clone().unwrap() != item.logical_minimum {
            self.emit_item(elems, ItemType::Global.into(), GlobalItemKind::LogicalMin.into(), item.logical_minimum as isize, true);
            self.logical_minimum = Some(item.logical_minimum);
        }
        if self.logical_maximum.is_none() || self.logical_maximum.clone().unwrap() != item.logical_maximum {
            self.emit_item(elems, ItemType::Global.into(), GlobalItemKind::LogicalMax.into(), item.logical_maximum as isize, true);
            self.logical_maximum = Some(item.logical_maximum);
        }
        if self.report_size.is_none() || self.report_size.clone().unwrap() != item.report_size {
            self.emit_item(elems, ItemType::Global.into(), GlobalItemKind::ReportSize.into(), item.report_size as isize, true);
            self.report_size = Some(item.report_size);
        }
        if self.report_count.is_none() || self.report_count.clone().unwrap() != item.report_count {
            self.emit_item(elems, ItemType::Global.into(), GlobalItemKind::ReportCount.into(), item.report_count as isize, true);
            self.report_count = Some(item.report_count);
        }
    }

    fn emit_field(&mut self, elems: &mut Punctuated<Pat, syn::token::Comma>, i: &ItemSpec, item: MainItem) {
        self.handle_globals(elems, item.clone());
        let item_data = match &i.settings {
            Some(s) => s.0 as isize,
            None => 0x02, // 0x02 = Data,Var,Abs
        };
        self.emit_item(elems, ItemType::Main.into(), item.kind.into(), item_data, true);

        if let Some(padding) = item.padding_bits {
            // Make another item of type constant to carry the remaining bits.
            let padding = MainItem{ report_size: 1, report_count: padding, ..item };
            self.handle_globals(elems, padding.clone());

            let mut const_settings = MainItemSetting{ 0: 0};
            const_settings.set_constant(true);
            const_settings.set_variable(true);
            self.emit_item(elems, ItemType::Main.into(), item.kind.into(), const_settings.0 as isize, true);
        }
    }

    fn emit_group(&mut self, elems: &mut Punctuated<Pat, syn::token::Comma>, spec: &GroupSpec, fields: &Fields) -> Result<()> {
        // println!("GROUP: {:?}", spec);

        if let Some(usage_page) = spec.usage_page {
            self.emit_item(elems, ItemType::Global.into(), GlobalItemKind::UsagePage.into(), usage_page as isize, false);
        }
        if let Some(usage) = spec.usage {
            self.emit_item(elems, ItemType::Local.into(), LocalItemKind::Usage.into(), usage as isize, false);
        }
        if let Some(usage_min) = spec.usage_min {
            self.emit_item(elems, ItemType::Local.into(), LocalItemKind::UsageMin.into(), usage_min as isize, false);
        }
        if let Some(usage_max) = spec.usage_max {
            self.emit_item(elems, ItemType::Local.into(), LocalItemKind::UsageMax.into(), usage_max as isize, false);
        }
        if let Some(report_id) = spec.report_id {
            self.emit_item(elems, ItemType::Global.into(), GlobalItemKind::ReportID.into(), report_id as isize, false);
        }
        if let Some(collection) = spec.collection {
            self.emit_item(elems, ItemType::Main.into(), MainItemKind::Collection.into(), collection as isize, false);
        }

        for name in spec.clone() {
            let f = spec.get(name.clone()).unwrap();
            match f {
                Spec::MainItem(i) => {
                    // println!("field: {:?}", i);
                    let d = field_decl(fields, name);
                    match analyze_field(d.clone(), d.ty, i) {
                        Ok(item) => self.emit_field(elems, i, item.descriptor_item),
                        Err(e) => return Err(e),
                    }
                },
                Spec::Collection(g) => if let Err(e) = self.emit_group(elems, g, fields) {
                    return Err(e);
                },
            }
        }

        if let Some(_) = spec.collection { // Close collection.
            elems.push(byte_literal(0xc0));
        }
        Ok(())
    }
}

fn field_decl(fields: &Fields, name: String) -> Field {
    for field in fields {
        let ident = field.ident.clone().unwrap().to_string();
        if ident == name {
            return field.clone();
        }
    }
    panic!(format!("internal error: could not find field {} which should exist", name))
}

fn compile(spec: GroupSpec, fields: &Fields) -> Result<PatSlice> {
    let mut compiler = DescCompilation{ ..Default::default() };
    let mut elems = Punctuated::new();

    if let Err(e) = compiler.emit_group(&mut elems, &spec, fields) {
        return Err(e);
    };

    Ok(PatSlice{
        attrs: vec![],
        elems: elems,
        bracket_token: Bracket{span: Span::call_site()},
    })
}

// MainItem describes all the mandatory data points of a Main item.
#[derive(Debug, Default, Clone)]
struct MainItem {
    kind: MainItemKind,
    logical_minimum: isize,
    logical_maximum: isize,
    report_count: u16,
    report_size: u16,
    padding_bits: Option<u16>,
}

#[derive(Debug)]
struct ReportUnaryField {
    bit_width: usize,
    descriptor_item: MainItem,
    ident: Ident,
}

fn analyze_field(field: Field, ft: Type, item: &ItemSpec) -> Result<ReportUnaryField> {
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
