//! Implements generation of HID report descriptors as well as common reports
extern crate usbd_hid_macros;
extern crate serde;
use serde::ser::{Serialize, Serializer, SerializeTuple};

pub use usbd_hid_macros::gen_hid_descriptor;

/// Report types where serialized HID report descriptors are available.
pub trait SerializedDescriptor {
    fn desc() -> &'static[u8];
}

/// Report types which serialize into input reports, ready for transmission.
pub trait AsInputReport: Serialize {}

/// Prelude for modules which use the `gen_hid_descriptor` macro.
pub mod generator_prelude {
    pub use usbd_hid_macros::gen_hid_descriptor;
    pub use crate::descriptor::{SerializedDescriptor, AsInputReport};
    pub use serde::ser::{Serialize, SerializeTuple, Serializer};
}

/// MouseReport describes a report and its companion descriptor than can be used
/// to send mouse movements and button presses to a host.
#[gen_hid_descriptor(
    (collection = APPLICATION, usage_page = GENERIC_DESKTOP, usage = MOUSE) = {
        (collection = PHYSICAL, usage = POINTER) = {
            (usage_page = BUTTON, usage_min = BUTTON_1, usage_max = BUTTON_3) = {
                #[packed_bits 3] #[item_settings data,variable,absolute] buttons=input;
            };
            (usage_page = GENERIC_DESKTOP,) = {
                (usage = X,) = {
                    #[item_settings data,variable,relative] x=input;
                };
                (usage = Y,) = {
                    #[item_settings data,variable,relative] y=input;
                };
            };
        };
    }
)]
#[allow(dead_code)]
pub struct MouseReport {
    pub buttons: u8,
    pub x: i8,
    pub y: i8,
}

/// KeyboardReport describes a report and its companion descriptor that can be
/// used to send keyboard button presses to a host and receive the status of the
/// keyboard LEDs.
#[gen_hid_descriptor(
    (collection = APPLICATION, usage_page = GENERIC_DESKTOP, usage = KEYBOARD) = {
        (usage_page = KEYBOARD, usage_min = 0xE0, usage_max = 0xE7) = {
            #[packed_bits 8] #[item_settings data,variable,absolute] modifier=input;
        };
        (usage_page = LEDS, usage_min = 0x01, usage_max = 0x05) = {
            #[packed_bits 5] #[item_settings data,variable,absolute] leds=output;
        };
        (usage_page = KEYBOARD, usage_min = 0x00, usage_max = 0x65) = {
            #[item_settings data,array,absolute] keycodes=input;
        };
        (usage_page = CONSUMER, usage_min = 0x0, usage_max = 0x23C) = {
            #[item_settings data,array,absolute,not_null] mediakey=input;
        };
    }
)]
#[allow(dead_code)]
pub struct KeyboardReport {
    pub modifier: u8,
    pub leds: u8,
    pub keycodes: [u8; 6],
    pub mediakey: u16,
}
