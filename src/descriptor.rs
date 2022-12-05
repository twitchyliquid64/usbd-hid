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

/// KeyboardUsage describes the key codes to be used in implementing a USB keyboard.
///
/// The usage type of all key codes is Selectors, except for the modifier keys
/// Keyboard Left Control to Keyboard Right GUI which are Dynamic Flags.
///
/// Reference: <https://usb.org/sites/default/files/hut1_3_0.pdf>
///
#[repr(u8)]
#[allow(unused)]
#[non_exhaustive]
#[derive(Copy, Debug, Clone, Eq, PartialEq)]
pub enum KeyboardUsage {
    // 0x00: Reserved
    KeyboardErrorRollOver = 0x01,
    KeyboardPOSTFail = 0x02,
    KeyboardErrorUndefined = 0x03,
    KeyboardAa = 0x04,
    KeyboardBb = 0x05,
    KeyboardCc = 0x06,
    KeyboardDd = 0x07,
    KeyboardEe = 0x08,
    KeyboardFf = 0x09,
    KeyboardGg = 0x0A,
    KeyboardHh = 0x0B,
    KeyboardIi = 0x0C,
    KeyboardJj = 0x0D,
    KeyboardKk = 0x0E,
    KeyboardLl = 0x0F,
    KeyboardMm = 0x10,
    KeyboardNn = 0x11,
    KeyboardOo = 0x12,
    KeyboardPp = 0x13,
    KeyboardQq = 0x14,
    KeyboardRr = 0x15,
    KeyboardSs = 0x16,
    KeyboardTt = 0x17,
    KeyboardUu = 0x18,
    KeyboardVv = 0x19,
    KeyboardWw = 0x1A,
    KeyboardXx = 0x1B,
    KeyboardYy = 0x1C,
    KeyboardZz = 0x1D,
    Keyboard1Bang = 0x1E,
    Keyboard2At = 0x1F,
    Keyboard3Hash = 0x20,
    Keyboard4Dollar = 0x21,
    Keyboard5Percent = 0x22,
    Keyboard6Caret = 0x23,
    Keyboard7Ampersand = 0x24,
    Keyboard8Star = 0x25,
    Keyboard9OpenPar = 0x26,
    Keyboard0ClosePar = 0x27,
    KeyboardEnter = 0x28,
    KeyboardEscape = 0x29,
    KeyboardBackspace = 0x2A,
    KeyboardTab = 0x2B,
    KeyboardSpacebar = 0x2C,
    KeyboardDashUnderscore = 0x2D,
    KeyboardEqualPlus = 0x2E,
    KeyboardOpenBracket = 0x2F,
    KeyboardCloseBracket = 0x30,
    KeyboardBackslashBar = 0x31,
    KeyboardNonUSHash = 0x32,
    KeyboardSemiColon = 0x33,
    KeyboardSingleDoubleQuote = 0x34,
    KeyboardTickTilde = 0x35,
    KeyboardCommaLess = 0x36,
    KeyboardPeriodGreater = 0x37,
    KeyboardSlashQuestion = 0x38,
    KeyboardCapsLock = 0x39,
    KeyboardF1 = 0x3A,
    KeyboardF2 = 0x3B,
    KeyboardF3 = 0x3C,
    KeyboardF4 = 0x3D,
    KeyboardF5 = 0x3E,
    KeyboardF6 = 0x3F,
    KeyboardF7 = 0x40,
    KeyboardF8 = 0x41,
    KeyboardF9 = 0x42,
    KeyboardF10 = 0x43,
    KeyboardF11 = 0x44,
    KeyboardF12 = 0x45,
    KeyboardPrintScreen = 0x46,
    KeyboardScrollLock = 0x47,
    KeyboardPause = 0x48,
    KeyboardInsert = 0x49,
    KeyboardHome = 0x4A,
    KeyboardPageUp = 0x4B,
    KeyboardDelete = 0x4C,
    KeyboardEnd = 0x4D,
    KeyboardPageDown = 0x4E,
    KeyboardRightArrow = 0x4F,
    KeyboardLeftArrow = 0x50,
    KeyboardDownArrow = 0x51,
    KeyboardUpArrow = 0x52,
    KeypadNumLock = 0x53,
    KeypadDivide = 0x54,
    KeypadMultiply = 0x55,
    KeypadMinus = 0x56,
    KeypadPlus = 0x57,
    KeypadEnter = 0x58,
    Keypad1End = 0x59,
    Keypad2DownArrow = 0x5A,
    Keypad3PageDown = 0x5B,
    Keypad4LeftArrow = 0x5C,
    Keypad5 = 0x5D,
    Keypad6RightArrow = 0x5E,
    Keypad7Home = 0x5F,
    Keypad8UpArrow = 0x60,
    Keypad9PageUp = 0x61,
    Keypad0Insert = 0x62,
    KeypadPeriodDelete = 0x63,
    KeyboardNonUSSlash = 0x64,
    KeyboardApplication = 0x65,
    KeyboardPower = 0x66,
    KeypadEqual = 0x67,
    KeyboardF13 = 0x68,
    KeyboardF14 = 0x69,
    KeyboardF15 = 0x6A,
    KeyboardF16 = 0x6B,
    KeyboardF17 = 0x6C,
    KeyboardF18 = 0x6D,
    KeyboardF19 = 0x6E,
    KeyboardF20 = 0x6F,
    KeyboardF21 = 0x70,
    KeyboardF22 = 0x71,
    KeyboardF23 = 0x72,
    KeyboardF24 = 0x73,
    KeyboardExecute = 0x74,
    KeyboardHelp = 0x75,
    KeyboardMenu = 0x76,
    KeyboardSelect = 0x77,
    KeyboardStop = 0x78,
    KeyboardAgain = 0x79,
    KeyboardUndo = 0x7A,
    KeyboardCut = 0x7B,
    KeyboardCopy = 0x7C,
    KeyboardPaste = 0x7D,
    KeyboardFind = 0x7E,
    KeyboardMute = 0x7F,
    KeyboardVolumeUp = 0x80,
    KeyboardVolumeDown = 0x81,
    KeyboardLockingCapsLock = 0x82,
    KeyboardLockingNumLock = 0x83,
    KeyboardLockingScrollLock = 0x84,
    KeypadComma = 0x85,
    KeypadEqualAS400 = 0x86,
    KeyboardInternational1 = 0x87,
    KeyboardInternational2 = 0x88,
    KeyboardInternational3 = 0x89,
    KeyboardInternational4 = 0x8A,
    KeyboardInternational5 = 0x8B,
    KeyboardInternational6 = 0x8C,
    KeyboardInternational7 = 0x8D,
    KeyboardInternational8 = 0x8E,
    KeyboardInternational9 = 0x8F,
    KeyboardLANG1 = 0x90,
    KeyboardLANG2 = 0x91,
    KeyboardLANG3 = 0x92,
    KeyboardLANG4 = 0x93,
    KeyboardLANG5 = 0x94,
    KeyboardLANG6 = 0x95,
    KeyboardLANG7 = 0x96,
    KeyboardLANG8 = 0x97,
    KeyboardLANG9 = 0x98,
    KeyboardAlternateErase = 0x99,
    KeyboardSysReqAttention = 0x9A,
    KeyboardCancel = 0x9B,
    KeyboardClear = 0x9C,
    KeyboardPrior = 0x9D,
    KeyboardReturn = 0x9E,
    KeyboardSeparator = 0x9F,
    KeyboardOut = 0xA0,
    KeyboardOper = 0xA1,
    KeyboardClearAgain = 0xA2,
    KeyboardCrSelProps = 0xA3,
    KeyboardExSel = 0xA4,
    // 0xA5-0xAF: Reserved
    Keypad00 = 0xB0,
    Keypad000 = 0xB1,
    ThousandsSeparator = 0xB2,
    DecimalSeparator = 0xB3,
    CurrencyUnit = 0xB4,
    CurrencySubunit = 0xB5,
    KeypadOpenPar = 0xB6,
    KeypadClosePar = 0xB7,
    KeypadOpenCurlyBrace = 0xB8,
    KeypadCloseCurlyBrace = 0xB9,
    KeypadTab = 0xBA,
    KeypadBackspace = 0xBB,
    KeypadA = 0xBC,
    KeypadB = 0xBD,
    KeypadC = 0xBE,
    KeypadD = 0xBF,
    KeypadE = 0xC0,
    KeypadF = 0xC1,
    KeypadBitwiseXor = 0xC2,
    KeypadLogicalXor = 0xC3,
    KeypadModulo = 0xC4,
    KeypadLeftShift = 0xC5,
    KeypadRightShift = 0xC6,
    KeypadBitwiseAnd = 0xC7,
    KeypadLogicalAnd = 0xC8,
    KeypadBitwiseOr = 0xC9,
    KeypadLogicalOr = 0xCA,
    KeypadColon = 0xCB,
    KeypadHash = 0xCC,
    KeypadSpace = 0xCD,
    KeypadAt = 0xCE,
    KeypadBang = 0xCF,
    KeypadMemoryStore = 0xD0,
    KeypadMemoryRecall = 0xD1,
    KeypadMemoryClear = 0xD2,
    KeypadMemoryAdd = 0xD3,
    KeypadMemorySubtract = 0xD4,
    KeypadMemoryMultiply = 0xD5,
    KeypadMemoryDivide = 0xD6,
    KeypadPositiveNegative = 0xD7,
    KeypadClear = 0xD8,
    KeypadClearEntry = 0xD9,
    KeypadBinary = 0xDA,
    KeypadOctal = 0xDB,
    KeypadDecimal = 0xDC,
    KeypadHexadecimal = 0xDD,
    // 0xDE-0xDF: Reserved
    KeyboardLeftControl = 0xE0,
    KeyboardLeftShift = 0xE1,
    KeyboardLeftAlt = 0xE2,
    KeyboardLeftGUI = 0xE3,
    KeyboardRightControl = 0xE4,
    KeyboardRightShift = 0xE5,
    KeyboardRightAlt = 0xE6,
    KeyboardRightGUI = 0xE7,
    // 0xE8-0xFF: Reserved
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
