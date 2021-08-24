//! Implements generation of HID report descriptors as well as common reports
extern crate serde;
extern crate usbd_hid_macros;
use serde::ser::{Serialize, SerializeTuple, Serializer};

pub use usbd_hid_macros::gen_hid_descriptor;

/// Report types where serialized HID report descriptors are available.
pub trait SerializedDescriptor {
    fn desc() -> &'static [u8];
}

/// Report types which serialize into input reports, ready for transmission.
pub trait AsInputReport: Serialize {}

/// Prelude for modules which use the `gen_hid_descriptor` macro.
pub mod generator_prelude {
    pub use crate::descriptor::{AsInputReport, SerializedDescriptor};
    pub use serde::ser::{Serialize, SerializeTuple, Serializer};
    pub use usbd_hid_macros::gen_hid_descriptor;
}

/// MouseReport describes a report and its companion descriptor than can be used
/// to send mouse movements and button presses to a host.
#[gen_hid_descriptor(
    (collection = APPLICATION, usage_page = GENERIC_DESKTOP, usage = MOUSE) = {
        (collection = PHYSICAL, usage = POINTER) = {
            (usage_page = BUTTON, usage_min = BUTTON_1, usage_max = BUTTON_8) = {
                #[packed_bits 8] #[item_settings data,variable,absolute] buttons=input;
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
            (usage_page = CONSUMER,) = {
                (usage = AC_PAN,) = {
                    #[item_settings data,variable,relative] pan=input;
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
    pub wheel: i8, // Scroll down (negative) or up (positive) this many units
    pub pan: i8,   // Scroll left (negative) or right (positive) this many units
}

/// KeyboardReport describes a report and its companion descriptor that can be
/// used to send keyboard button presses to a host and receive the status of the
/// keyboard LEDs.
#[gen_hid_descriptor(
    (collection = APPLICATION, usage_page = GENERIC_DESKTOP, usage = KEYBOARD) = {
        (usage_page = KEYBOARD, usage_min = 0xE0, usage_max = 0xE7) = {
            #[packed_bits 8] #[item_settings data,variable,absolute] modifier=input;
        };
        (usage_min = 0x00, usage_max = 0xFF) = {
            #[item_settings constant,variable,absolute] reserved=input;
        };
        (usage_page = LEDS, usage_min = 0x01, usage_max = 0x05) = {
            #[packed_bits 5] #[item_settings data,variable,absolute] leds=output;
        };
        (usage_page = KEYBOARD, usage_min = 0x00, usage_max = 0xDD) = {
            #[item_settings data,array,absolute] keycodes=input;
        };
    }
)]
#[allow(dead_code)]
pub struct KeyboardReport {
    pub modifier: u8,
    pub reserved: u8,
    pub leds: u8,
    pub keycodes: [u8; 6],
}

/// MediaKeyboardReport describes a report and descriptor that can be used to
/// send consumer control commands to the host.
///
/// This is commonly used for sending media player for keyboards with media player
/// keys, but can be used for all sorts of Consumer Page functionality.
///
/// Reference: <https://usb.org/sites/default/files/hut1_2.pdf>
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
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum MediaKey {
    Zero = 0x00,
    Play = 0xB0,
    Pause = 0xB1,
    Record = 0xB2,
    NextTrack = 0xB5,
    PrevTrack = 0xB6,
    Stop = 0xB7,
    RandomPlay = 0xB9,
    Repeat = 0xBC,
    PlayPause = 0xCD,
    Mute = 0xE2,
    VolumeIncrement = 0xE9,
    VolumeDecrement = 0xEA,
}

impl From<MediaKey> for u16 {
    fn from(mk: MediaKey) -> u16 {
        mk as u16
    }
}

/// SystemControlReport describes a report and descriptor that can be used to
/// send system control commands to the host.
///
/// This is commonly used to enter sleep mode, power down, hibernate, etc.
///
/// Reference: <https://usb.org/sites/default/files/hut1_2.pdf>
///
/// NOTE: For Windows compatibility usage_min should start at 0x81
/// NOTE: For macOS scrollbar compatibility, logical minimum should start from 1
///       (scrollbars disappear if logical_min is set to 0)
#[gen_hid_descriptor(
    (collection = APPLICATION, usage_page = GENERIC_DESKTOP, usage = SYSTEM_CONTROL) = {
        (usage_min = 0x81, usage_max = 0xB7, logical_min = 1) = {
            #[item_settings data,array,absolute,not_null] usage_id=input;
        };
    }
)]
#[allow(dead_code)]
pub struct SystemControlReport {
    pub usage_id: u8,
}

/// System control usage ids to use with SystemControlReport
#[non_exhaustive]
#[repr(u8)]
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum SystemControlKey {
    PowerDown = 0x81,
    Sleep = 0x82,
    WakeUp = 0x83,
    ContextMenu = 0x84,
    MainMenu = 0x85,
    AppMenu = 0x86,
    MenuHelp = 0x87,
    MenuExit = 0x88,
    MenuSelect = 0x89,
    MenuRight = 0x8A,
    MenuLeft = 0x8B,
    MenuUp = 0x8C,
    MenuDown = 0x8D,
    ColdRestart = 0x8E,
    WarmRestart = 0x8F,
    DpadUp = 0x90,
    DpadDown = 0x91,
    DpadRight = 0x92,
    DpadLeft = 0x93,
    SystemFunctionShift = 0x97,
    SystemFunctionShiftLock = 0x98,
    SystemDismissNotification = 0x9A,
    SystemDoNotDisturb = 0x9B,
    Dock = 0xA0,
    Undock = 0xA1,
    Setup = 0xA2,
    Break = 0xA3,
    DebuggerBreak = 0xA4,
    ApplicationBreak = 0xA5,
    ApplicationDebuggerBreak = 0xA6,
    SpeakerMute = 0xA7,
    Hibernate = 0xA8,
    DisplayInvert = 0xB0,
    DisplayInternal = 0xB1,
    DisplayExternal = 0xB2,
    DisplayBoth = 0xB3,
    DisplayDual = 0xB4,
    DisplayToggleInternalExternal = 0xB5,
    DisplaySwapPrimarySecondary = 0xB6,
    DisplayLcdAutoscale = 0xB7,
}

impl From<SystemControlKey> for u8 {
    fn from(sck: SystemControlKey) -> u8 {
        sck as u8
    }
}
