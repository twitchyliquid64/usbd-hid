//! Internal implementation details of usbd-hid.

extern crate proc_macro;
extern crate usbd_hid_descriptors;

use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::punctuated::Punctuated;
use syn::token::Bracket;
use syn::{parse, parse_macro_input, Expr, Fields, ItemStruct};
use syn::{Pat, PatSlice, Result};

use byteorder::{ByteOrder, LittleEndian};
use usbd_hid_descriptors::*;

mod spec;
use spec::*;
mod item;
use item::*;
mod packer;
use packer::{gen_serializer, uses_report_ids};

/// Attribute to generate a HID descriptor & serialization code
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
/// If report ID's are not used, input (device-to-host) serialization code is generated
/// automatically, and is represented by the implementation of the `AsInputReport` trait.
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
/// ```text
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
///     If the number of packed bits is less than the natural bit width of the field, the
///     remaining most-significant bits are set as constants within the report and are not used.
///     `packed_bits` is typically used to implement buttons.
///   - `item_settings` describes settings on the input/output item, as enumerated in section
///     6.2.2.5 of the [HID specification, version 1.11](https://www.usb.org/sites/default/files/documents/hid1_11.pdf).
///     By default, all items are configured as `(Data,Var,Abs,No Wrap,Linear,Preferred State,No Null Position)`.
///
/// ## Quirks
///
/// By default generated descriptors are such to maximize compatibility. To change this
/// behaviour, you can use a `#[quirks <settings>]` attribute on the relevant input/output
/// item.
/// For now, the only quirk is `#[quirks allow_short]`, which allows global features to be
/// serialized in a 1 byte form. This is disabled by default as the Windows HID parser
/// considers it invalid.
#[proc_macro_attribute]
pub fn gen_hid_descriptor(args: TokenStream, input: TokenStream) -> TokenStream {
    let decl = parse_macro_input!(input as ItemStruct);
    let spec = parse_macro_input!(args as GroupSpec);
    let ident = decl.ident.clone();

    // Error if the struct doesn't name its fields.
    match decl.clone().fields {
        Fields::Named(_) => (),
        _ => {
            return parse::Error::new(
                ident.span(),
                "`#[gen_hid_descriptor]` type must name fields",
            )
            .to_compile_error()
            .into()
        }
    };

    let do_serialize = !uses_report_ids(&Spec::Collection(spec.clone()));

    let output = match compile_descriptor(spec, &decl.fields) {
        Ok(d) => d,
        Err(e) => return e.to_compile_error().into(),
    };
    let (descriptor, fields) = output;

    let mut out = quote! {
        #[derive(Debug, Clone, Copy)]
        #[repr(C, packed)]
        #decl

        impl SerializedDescriptor for #ident {
            fn desc() -> &'static[u8] {
                &#descriptor
            }
        }
    };

    if do_serialize {
        let input_serializer = match gen_serializer(fields, MainItemKind::Input) {
            Ok(s) => s,
            Err(e) => return e.to_compile_error().into(),
        };

        out = quote! {
            #out

            impl Serialize for #ident {
                fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                where
                    S: Serializer,
                {
                    #input_serializer
                }
            }
            impl AsInputReport for #ident {}
        };
    }

    TokenStream::from(out)
}

fn compile_descriptor(
    spec: GroupSpec,
    fields: &Fields,
) -> Result<(PatSlice, Vec<ReportUnaryField>)> {
    let mut compiler = DescCompilation {
        ..Default::default()
    };
    let mut elems = Punctuated::new();

    if let Err(e) = compiler.emit_group(&mut elems, &spec, fields) {
        return Err(e);
    };

    Ok((
        PatSlice {
            attrs: vec![],
            elems: elems,
            bracket_token: Bracket {
                span: Span::call_site(),
            },
        },
        compiler.report_fields(),
    ))
}

#[derive(Default)]
struct DescCompilation {
    logical_minimum: Option<isize>,
    logical_maximum: Option<isize>,
    report_size: Option<u16>,
    report_count: Option<u16>,
    processed_fields: Vec<ReportUnaryField>,
}

impl DescCompilation {
    fn report_fields(&self) -> Vec<ReportUnaryField> {
        self.processed_fields.clone()
    }

    fn emit(
        &self,
        elems: &mut Punctuated<Pat, syn::token::Comma>,
        prefix: &mut ItemPrefix,
        buf: [u8; 4],
        signed: bool,
    ) {
        // println!("buf: {:?}", buf);
        if buf[1..4] == [0, 0, 0] && !(signed && buf[0] == 255) {
            prefix.set_byte_count(1);
            elems.push(byte_literal(prefix.0));
            elems.push(byte_literal(buf[0]));
        } else if buf[2..4] == [0, 0] && !(signed && buf[1] == 255) {
            prefix.set_byte_count(2);
            elems.push(byte_literal(prefix.0));
            elems.push(byte_literal(buf[0]));
            elems.push(byte_literal(buf[1]));
        } else {
            prefix.set_byte_count(3);
            elems.push(byte_literal(prefix.0));
            elems.push(byte_literal(buf[0]));
            elems.push(byte_literal(buf[1]));
            elems.push(byte_literal(buf[2]));
            elems.push(byte_literal(buf[3]));
        }
        // println!("emitted {} data bytes", prefix.byte_count());
    }

    fn emit_item(
        &self,
        elems: &mut Punctuated<Pat, syn::token::Comma>,
        typ: u8,
        kind: u8,
        num: isize,
        signed: bool,
        allow_short_form: bool,
    ) {
        let mut prefix = ItemPrefix(0);
        prefix.set_tag(kind);
        prefix.set_type(typ);

        // TODO: Support long tags.

        // Section 6.2.2.4: An Input item could have a data size of zero (0)
        // bytes. In this case the value of each data bit for the item can be
        // assumed to be zero. This is functionally identical to using a item
        // tag that specifies a 4-byte data item followed by four zero bytes.
        let allow_short = typ == ItemType::Main.into() && kind == MainItemKind::Input.into();
        if allow_short_form && allow_short && num == 0 {
            prefix.set_byte_count(0);
            elems.push(byte_literal(prefix.0));
            return;
        }

        let mut buf = [0; 4];
        LittleEndian::write_i32(&mut buf, num as i32);
        self.emit(elems, &mut prefix, buf, signed);
    }

    fn handle_globals(&mut self, elems: &mut Punctuated<Pat, syn::token::Comma>, item: MainItem, quirks: ItemQuirks) {
        if self.logical_minimum.is_none()
            || self.logical_minimum.clone().unwrap() != item.logical_minimum
        {
            self.emit_item(
                elems,
                ItemType::Global.into(),
                GlobalItemKind::LogicalMin.into(),
                item.logical_minimum as isize,
                true,
                quirks.allow_short_form,
            );
            self.logical_minimum = Some(item.logical_minimum);
        }
        if self.logical_maximum.is_none()
            || self.logical_maximum.clone().unwrap() != item.logical_maximum
        {
            self.emit_item(
                elems,
                ItemType::Global.into(),
                GlobalItemKind::LogicalMax.into(),
                item.logical_maximum as isize,
                true,
                quirks.allow_short_form,
            );
            self.logical_maximum = Some(item.logical_maximum);
        }
        if self.report_size.is_none() || self.report_size.clone().unwrap() != item.report_size {
            self.emit_item(
                elems,
                ItemType::Global.into(),
                GlobalItemKind::ReportSize.into(),
                item.report_size as isize,
                true,
                quirks.allow_short_form,
            );
            self.report_size = Some(item.report_size);
        }
        if self.report_count.is_none() || self.report_count.clone().unwrap() != item.report_count {
            self.emit_item(
                elems,
                ItemType::Global.into(),
                GlobalItemKind::ReportCount.into(),
                item.report_count as isize,
                true,
                quirks.allow_short_form,
            );
            self.report_count = Some(item.report_count);
        }
    }

    fn emit_field(
        &mut self,
        elems: &mut Punctuated<Pat, syn::token::Comma>,
        i: &ItemSpec,
        item: MainItem,
    ) {
        self.handle_globals(elems, item.clone(), i.quirks);
        let item_data = match &i.settings {
            Some(s) => s.0 as isize,
            None => 0x02, // 0x02 = Data,Var,Abs
        };
        self.emit_item(
            elems,
            ItemType::Main.into(),
            item.kind.into(),
            item_data,
            true,
            i.quirks.allow_short_form,
        );

        if let Some(padding) = item.padding_bits {
            // Make another item of type constant to carry the remaining bits.
            let padding = MainItem {
                report_size: 1,
                report_count: padding,
                ..item
            };
            self.handle_globals(elems, padding.clone(), i.quirks);

            let mut const_settings = MainItemSetting { 0: 0 };
            const_settings.set_constant(true);
            const_settings.set_variable(true);
            self.emit_item(
                elems,
                ItemType::Main.into(),
                item.kind.into(),
                const_settings.0 as isize,
                true,
                i.quirks.allow_short_form,
            );
        }
    }

    fn emit_group(
        &mut self,
        elems: &mut Punctuated<Pat, syn::token::Comma>,
        spec: &GroupSpec,
        fields: &Fields,
    ) -> Result<()> {
        // println!("GROUP: {:?}", spec);

        if let Some(usage_page) = spec.usage_page {
            self.emit_item(
                elems,
                ItemType::Global.into(),
                GlobalItemKind::UsagePage.into(),
                usage_page as isize,
                false,
                false,
            );
        }
        for usage in &spec.usage {
            self.emit_item(
                elems,
                ItemType::Local.into(),
                LocalItemKind::Usage.into(),
                *usage as isize,
                false,
                false,
            );
        }
        if let Some(usage_min) = spec.usage_min {
            self.emit_item(
                elems,
                ItemType::Local.into(),
                LocalItemKind::UsageMin.into(),
                usage_min as isize,
                false,
                false,
            );
        }
        if let Some(usage_max) = spec.usage_max {
            self.emit_item(
                elems,
                ItemType::Local.into(),
                LocalItemKind::UsageMax.into(),
                usage_max as isize,
                false,
                false,
            );
        }
        if let Some(report_id) = spec.report_id {
            self.emit_item(
                elems,
                ItemType::Global.into(),
                GlobalItemKind::ReportID.into(),
                report_id as isize,
                false,
                false,
            );
        }
        if let Some(collection) = spec.collection {
            self.emit_item(
                elems,
                ItemType::Main.into(),
                MainItemKind::Collection.into(),
                collection as isize,
                false,
                false,
            );
        }
        if let Some(logical_minimum) = spec.logical_min {
            // Set to 0 to indicate that we've already set the default
            // See handle_globals
            self.logical_minimum = Some(0);
            self.emit_item(
                elems,
                ItemType::Global.into(),
                GlobalItemKind::LogicalMin.into(),
                logical_minimum as isize,
                false,
                false,
            );
        }

        for name in spec.clone() {
            let f = spec.get(name.clone()).unwrap();
            match f {
                Spec::MainItem(i) => {
                    let d = field_decl(fields, name);
                    match analyze_field(d.clone(), d.ty, i) {
                        Ok(item) => {
                            self.processed_fields.push(item.clone());
                            self.emit_field(elems, i, item.descriptor_item)
                        }
                        Err(e) => return Err(e),
                    }
                }
                Spec::Collection(g) => {
                    if let Err(e) = self.emit_group(elems, g, fields) {
                        return Err(e);
                    }
                }
            }
        }

        if let Some(_) = spec.collection {
            // Close collection.
            elems.push(byte_literal(0xc0));
        }
        Ok(())
    }
}

fn byte_literal(lit: u8) -> Pat {
    // print!("{:x} ", lit);
    // println!();
    Pat::Lit(syn::PatLit {
        attrs: vec![],
        expr: Box::new(Expr::Lit(syn::ExprLit {
            attrs: vec![],
            lit: syn::Lit::Byte(syn::LitByte::new(lit, Span::call_site())),
        })),
    })
}
