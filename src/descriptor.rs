//! Implements generation of HID report descriptors as well as common reports
extern crate usbd_hid_macros;
extern crate serde;
use serde::ser::{Serialize, Serializer, SerializeTuple};

pub use usbd_hid_macros::gen_hid_descriptor;

/// Types where serialized HID report descriptors are available.
pub trait SerializedDescriptor {
    fn desc() -> &'static[u8];
}

/// Prelude for modules which use the `gen_hid_descriptor` macro.
pub mod generator_prelude {
    pub use usbd_hid_macros::gen_hid_descriptor;
    pub use crate::descriptor::SerializedDescriptor;
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
