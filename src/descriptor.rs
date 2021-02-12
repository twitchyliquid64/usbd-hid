//! Implements generation of HID report descriptors as well as common reports

/// Report types where serialized HID report descriptors are available.
pub trait HIDDescriptor {
    fn desc() -> &'static [u8];
}

/// Report types where serializable or deserializable in/out types are available.
pub trait HIDDescriptorTypes: HIDDescriptor {
    type DeviceToHostReport;
    type HostToDeviceReport;
}

/// Placeholder Type that is nither serializable nor deserializable
pub struct UnsupportedDescriptor;

/// Prelude for modules which use the `gen_hid_descriptor` macro.
/// To use managed serialize/deserialize features, crate `serde` must be
/// included e.g. `serde = { version = "~1.0", default-features = false }`
pub mod generator_prelude {
    pub use usbd_hid_macros::gen_hid_descriptor;
    pub use crate::descriptor::{HIDDescriptor, HIDDescriptorTypes, UnsupportedDescriptor};
    pub use serde::{Serialize, Deserialize};
}

use generator_prelude::*;

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
                (usage = WHEEL,) = {
                    #[item_settings data,variable,relative] wheel=input;
                };
            };
        };
    }
)]
#[allow(dead_code)]
#[derive(Debug)]
pub struct MouseReport {
    pub buttons: u8,
    pub x: i8,
    pub y: i8,
    pub wheel: i8, // Scroll down (negative) or up (positive) this many units
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
    }
)]
#[allow(dead_code)]
#[derive(Debug)]
pub struct KeyboardReport {
    pub modifier: u8,
    pub leds: u8,
    pub keycodes: [u8; 6],
}

/// MediaKeyboardReport describes a report and descriptor that can be used to
/// send consumer control commands to the host.
///
/// This is commonly used for sending media player for keyboards with media player
/// keys, but can be used for all sorts of Consumer Page functionality.
///
/// Reference: https://usb.org/sites/default/files/hut1_2.pdf
///
#[gen_hid_descriptor(
    (collection = APPLICATION, usage_page = CONSUMER, usage = CONSUMER_CONTROL) = {
        (usage_page = CONSUMER, usage_min = 0x00, usage_max = 0x514) = {
            #[item_settings data,array,absolute,not_null] usage_id=input;
        };
    }
)]
#[allow(dead_code)]
pub struct MediaKeyboardReport {
    pub usage_id: u16,
}

/// Media player usage ids that can be used in MediaKeyboardReport
#[non_exhaustive]
#[repr(u16)]
#[derive(Debug)]
pub enum MediaKey {
    Zero = 0x00,
    Play = 0xB0,
    Pause = 0xB1,
    Record = 0xB2,
    NextTrack =0xB5,
    PrevTrack = 0xB6,
    Stop = 0xB7,
    RandomPlay = 0xB9,
    Repeat = 0xBC,
    PlayPause = 0xCD,
    VolumeIncrement = 0xE9,
    VolumeDecrement = 0xEA,
}

impl Into<u16> for MediaKey {
    fn into(self) -> u16 {
        self as u16
    }
}