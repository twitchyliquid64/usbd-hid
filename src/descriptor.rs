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

impl KeyboardReport {
    pub const fn default() -> Self {
        Self {
            modifier: 0,
            reserved: 0,
            leds: 0,
            keycodes: [0u8; 6],
        }
    }
}

/// KeyboardUsage describes the key codes to be used in implementing a USB keyboard.
///
/// The usage type of all key codes is Selectors, except for the modifier keys
/// Keyboard Left Control to Keyboard Right GUI which are Dynamic Flags.
///
/// Reference: <https://usb.org/sites/default/files/hut1_3_0.pdf> (Section 10, page 88)
#[repr(u8)]
#[allow(unused)]
#[non_exhaustive]
#[derive(Copy, Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum KeyboardUsage {
    // 0x00: Reserved
    /// Keyboard ErrorRollOver (Footnote 1)
    KeyboardErrorRollOver = 0x01,
    /// Keyboard POSTFail (Footnote 1)
    KeyboardPOSTFail = 0x02,
    /// Keyboard ErrorUndefined (Footnote 1)
    KeyboardErrorUndefined = 0x03,
    /// Keyboard a and A (Footnote 2)
    KeyboardAa = 0x04,
    /// Keyboard b and B
    KeyboardBb = 0x05,
    /// Keyboard c and C (Footnote 2)
    KeyboardCc = 0x06,
    /// Keyboard d and D
    KeyboardDd = 0x07,
    /// Keyboard e and E
    KeyboardEe = 0x08,
    /// Keyboard f and F
    KeyboardFf = 0x09,
    /// Keyboard g and G
    KeyboardGg = 0x0A,
    /// Keyboard h and H
    KeyboardHh = 0x0B,
    /// Keyboard i and I
    KeyboardIi = 0x0C,
    /// Keyboard j and J
    KeyboardJj = 0x0D,
    /// Keyboard k and K
    KeyboardKk = 0x0E,
    /// Keyboard l and L
    KeyboardLl = 0x0F,
    /// Keyboard m and M (Footnote 2)
    KeyboardMm = 0x10,
    /// Keyboard n and N
    KeyboardNn = 0x11,
    /// Keyboard o and O (Footnote 2)
    KeyboardOo = 0x12,
    /// Keyboard p and P (Footnote 2)
    KeyboardPp = 0x13,
    /// Keyboard q and Q (Footnote 2)
    KeyboardQq = 0x14,
    /// Keyboard r and R
    KeyboardRr = 0x15,
    /// Keyboard s and S
    KeyboardSs = 0x16,
    /// Keyboard t and T
    KeyboardTt = 0x17,
    /// Keyboard u and U
    KeyboardUu = 0x18,
    /// Keyboard v and V
    KeyboardVv = 0x19,
    /// Keyboard w and W (Footnote 2)
    KeyboardWw = 0x1A,
    /// Keyboard x and X (Footnote 2)
    KeyboardXx = 0x1B,
    /// Keyboard y and Y (Footnote 2)
    KeyboardYy = 0x1C,
    /// Keyboard z and Z (Footnote 2)
    KeyboardZz = 0x1D,
    /// Keyboard 1 and ! (Footnote 2)
    Keyboard1Exclamation = 0x1E,
    /// Keyboard 2 and @ (Footnote 2)
    Keyboard2At = 0x1F,
    /// Keyboard 3 and # (Footnote 2)
    Keyboard3Hash = 0x20,
    /// Keyboard 4 and $ (Footnote 2)
    Keyboard4Dollar = 0x21,
    /// Keyboard 5 and % (Footnote 2)
    Keyboard5Percent = 0x22,
    /// Keyboard 6 and ^ (Footnote 2)
    Keyboard6Caret = 0x23,
    /// Keyboard 7 and & (Footnote 2)
    Keyboard7Ampersand = 0x24,
    /// Keyboard 8 and * (Footnote 2)
    Keyboard8Asterisk = 0x25,
    /// Keyboard 9 and ( (Footnote 2)
    Keyboard9OpenParens = 0x26,
    /// Keyboard 0 and ) (Footnote 2)
    Keyboard0CloseParens = 0x27,
    /// Keyboard Return (ENTER) (Footnote 3)
    ///
    ///  (Footnote 3): Keyboard Enter and Keypad Enter generate different Usage codes.
    KeyboardEnter = 0x28,
    /// Keyboard ESCAPE
    KeyboardEscape = 0x29,
    /// Keyboard DELETE (Backspace) (Footnote 4)
    KeyboardBackspace = 0x2A,
    /// Keyboard Tab
    KeyboardTab = 0x2B,
    /// Keyboard Spacebar
    KeyboardSpacebar = 0x2C,
    /// Keyboard - and _ (Footnote 2)
    KeyboardDashUnderscore = 0x2D,
    /// Keyboard = and + (Footnote 2)
    KeyboardEqualPlus = 0x2E,
    /// Keyboard [ and { (Footnote 2)
    KeyboardOpenBracketBrace = 0x2F,
    /// Keyboard ] and } (Footnote 2)
    KeyboardCloseBracketBrace = 0x30,
    /// Keyboard \ and |
    KeyboardBackslashBar = 0x31,
    /// Keyboard Non-US # and (Footnote 5)
    KeyboardNonUSHash = 0x32,
    /// Keyboard ; and : (Footnote 2)
    KeyboardSemiColon = 0x33,
    /// Keyboard ' and " (Footnote 2)
    KeyboardSingleDoubleQuote = 0x34,
    /// Keyboard ` and ~ (Footnote 2)
    KeyboardBacktickTilde = 0x35,
    /// Keyboard , and < (Footnote 2)
    KeyboardCommaLess = 0x36,
    /// Keyboard . and > (Footnote 2)
    KeyboardPeriodGreater = 0x37,
    /// Keyboard / and ? (Footnote 2)
    KeyboardSlashQuestion = 0x38,
    /// Keyboard Caps Lock (Footnote 6)
    KeyboardCapsLock = 0x39,
    /// Keyboard F1
    KeyboardF1 = 0x3A,
    /// Keyboard F2
    KeyboardF2 = 0x3B,
    /// Keyboard F3
    KeyboardF3 = 0x3C,
    /// Keyboard F4
    KeyboardF4 = 0x3D,
    /// Keyboard F5
    KeyboardF5 = 0x3E,
    /// Keyboard F6
    KeyboardF6 = 0x3F,
    /// Keyboard F7
    KeyboardF7 = 0x40,
    /// Keyboard F8
    KeyboardF8 = 0x41,
    /// Keyboard F9
    KeyboardF9 = 0x42,
    /// Keyboard F10
    KeyboardF10 = 0x43,
    /// Keyboard F11
    KeyboardF11 = 0x44,
    /// Keyboard F12
    KeyboardF12 = 0x45,
    /// Keyboard PrintScreen (Footnote 7)
    KeyboardPrintScreen = 0x46,
    /// Keyboard ScrollLock (Footnote 6)
    KeyboardScrollLock = 0x47,
    /// Keyboard Pause (Footnote 7)
    KeyboardPause = 0x48,
    /// Keyboard Insert (Footnote 7)
    KeyboardInsert = 0x49,
    /// Keyboard Home (Footnote 7)
    KeyboardHome = 0x4A,
    /// Keyboard PageUp (Footnote 7)
    KeyboardPageUp = 0x4B,
    /// Keyboard Delete Forward (Footnote 7) (Footnote 8)
    KeyboardDelete = 0x4C,
    /// Keyboard End (Footnote 7)
    KeyboardEnd = 0x4D,
    /// Keyboard PageDown (Footnote 7)
    KeyboardPageDown = 0x4E,
    /// Keyboard RightArrow (Footnote 7)
    KeyboardRightArrow = 0x4F,
    /// Keyboard LeftArrow (Footnote 7)
    KeyboardLeftArrow = 0x50,
    /// Keyboard DownArrow (Footnote 7)
    KeyboardDownArrow = 0x51,
    /// Keyboard UpArrow (Footnote 7)
    KeyboardUpArrow = 0x52,
    /// Keypad Num Lock and Clear (Footnote 6)
    KeypadNumLock = 0x53,
    /// Keypad / (Footnote 7)
    KeypadDivide = 0x54,
    /// Keypad *
    KeypadMultiply = 0x55,
    /// Keypad -
    KeypadMinus = 0x56,
    /// Keypad +
    KeypadPlus = 0x57,
    /// Keypad ENTER (Footnote 3)
    KeypadEnter = 0x58,
    /// Keypad 1 and End
    Keypad1End = 0x59,
    /// Keypad 2 and DownArrow
    Keypad2DownArrow = 0x5A,
    /// Keypad 3 and PageDown
    Keypad3PageDown = 0x5B,
    /// Keypad 4 and LeftArrow
    Keypad4LeftArrow = 0x5C,
    /// Keypad 5
    Keypad5 = 0x5D,
    /// Keypad 6 and RightArrow
    Keypad6RightArrow = 0x5E,
    /// Keypad 7 and Home
    Keypad7Home = 0x5F,
    /// Keypad 8 and UpArrow
    Keypad8UpArrow = 0x60,
    /// Keypad 9 and PageUp
    Keypad9PageUp = 0x61,
    /// Keypad 0 and Insert
    Keypad0Insert = 0x62,
    /// Keypad . and Delete
    KeypadPeriodDelete = 0x63,
    /// Keyboard Non-US \ and | (Footnote 9) (Footnote 10)
    KeyboardNonUSSlash = 0x64,
    /// Keyboard Application (Footnote 11)
    KeyboardApplication = 0x65,
    /// Keyboard Power (Footnote 1)
    KeyboardPower = 0x66,
    /// Keypad =
    KeypadEqual = 0x67,
    /// Keyboard F13
    KeyboardF13 = 0x68,
    /// Keyboard F14
    KeyboardF14 = 0x69,
    /// Keyboard F15
    KeyboardF15 = 0x6A,
    /// Keyboard F16
    KeyboardF16 = 0x6B,
    /// Keyboard F17
    KeyboardF17 = 0x6C,
    /// Keyboard F18
    KeyboardF18 = 0x6D,
    /// Keyboard F19
    KeyboardF19 = 0x6E,
    /// Keyboard F20
    KeyboardF20 = 0x6F,
    /// Keyboard F21
    KeyboardF21 = 0x70,
    /// Keyboard F22
    KeyboardF22 = 0x71,
    /// Keyboard F23
    KeyboardF23 = 0x72,
    /// Keyboard F24
    KeyboardF24 = 0x73,
    /// Keyboard Execute
    KeyboardExecute = 0x74,
    /// Keyboard Help
    KeyboardHelp = 0x75,
    /// Keyboard Menu
    KeyboardMenu = 0x76,
    /// Keyboard Select
    KeyboardSelect = 0x77,
    /// Keyboard Stop
    KeyboardStop = 0x78,
    /// Keyboard Again
    KeyboardAgain = 0x79,
    /// Keyboard Undo
    KeyboardUndo = 0x7A,
    /// Keyboard Cut
    KeyboardCut = 0x7B,
    /// Keyboard Copy
    KeyboardCopy = 0x7C,
    /// Keyboard Paste
    KeyboardPaste = 0x7D,
    /// Keyboard Find
    KeyboardFind = 0x7E,
    /// Keyboard Mute
    KeyboardMute = 0x7F,
    /// Keyboard Volume Up
    KeyboardVolumeUp = 0x80,
    /// Keyboard Volume Down
    KeyboardVolumeDown = 0x81,
    /// Keyboad Locking Caps Lock (Footnote 12)
    KeyboardLockingCapsLock = 0x82,
    /// Keyboad Locking Num Lock (Footnote 12)
    KeyboardLockingNumLock = 0x83,
    /// Keyboad Locking Scroll Lock (Footnote 12)
    KeyboardLockingScrollLock = 0x84,
    /// Keypad Comma (Footnote 13)
    KeypadComma = 0x85,
    /// Keypad Equal Sign (Footnote 14)
    KeypadEqualSign = 0x86,
    /// Keyboard International1 (Footnote 15) (Footnote 16)
    KeyboardInternational1 = 0x87,
    /// Keyboard International2 (Footnote 17)
    KeyboardInternational2 = 0x88,
    /// Keyboard International3 (Footnote 18)
    KeyboardInternational3 = 0x89,
    /// Keyboard International4 (Footnote 19)
    KeyboardInternational4 = 0x8A,
    /// Keyboard International5 (Footnote 20)
    KeyboardInternational5 = 0x8B,
    /// Keyboard International6 (Footnote 21)
    KeyboardInternational6 = 0x8C,
    /// Keyboard International7 (Footnote 22)
    KeyboardInternational7 = 0x8D,
    /// Keyboard International8 (Footnote 23)
    KeyboardInternational8 = 0x8E,
    /// Keyboard International9 (Footnote 23)
    KeyboardInternational9 = 0x8F,
    /// Keyboard LANG1 (Footnote 24)
    KeyboardLANG1 = 0x90,
    /// Keyboard LANG2 (Footnote 25)
    KeyboardLANG2 = 0x91,
    /// Keyboard LANG3 (Footnote 26)
    KeyboardLANG3 = 0x92,
    /// Keyboard LANG4 (Footnote 27)
    KeyboardLANG4 = 0x93,
    /// Keyboard LANG5 (Footnote 28)
    KeyboardLANG5 = 0x94,
    /// Keyboard LANG6 (Footnote 29)
    KeyboardLANG6 = 0x95,
    /// Keyboard LANG7 (Footnote 29)
    KeyboardLANG7 = 0x96,
    /// Keyboard LANG8 (Footnote 29)
    KeyboardLANG8 = 0x97,
    /// Keyboard LANG9 (Footnote 29)
    KeyboardLANG9 = 0x98,
    /// Keyboard Alternate Erase (Footnote 30)
    KeyboardAlternateErase = 0x99,
    /// Keyboard SysReq/Attention (Footnote 7)
    KeyboardSysReqAttention = 0x9A,
    /// Keyboard Cancel
    KeyboardCancel = 0x9B,
    /// Keyboard Clear
    KeyboardClear = 0x9C,
    /// Keyboard Prior
    KeyboardPrior = 0x9D,
    /// Keyboard Return
    KeyboardReturn = 0x9E,
    /// Keyboard Separator
    KeyboardSeparator = 0x9F,
    /// Keyboard Out
    KeyboardOut = 0xA0,
    /// Keyboard Oper
    KeyboardOper = 0xA1,
    /// Keyboard Clear/Again
    KeyboardClearAgain = 0xA2,
    /// Keyboard CrSel/Props
    KeyboardCrSelProps = 0xA3,
    /// Keyboard ExSel
    KeyboardExSel = 0xA4,
    // 0xA5-0xAF: Reserved
    /// Keypad 00
    Keypad00 = 0xB0,
    /// Keypad 000
    Keypad000 = 0xB1,
    /// Thousands Separator (Footnote 31)
    ThousandsSeparator = 0xB2,
    /// Decimal Separator (Footnote 31)
    DecimalSeparator = 0xB3,
    /// Currency Unit (Footnote 32)
    CurrencyUnit = 0xB4,
    /// Currency Sub-unit (Footnote 32)
    CurrencySubunit = 0xB5,
    /// Keypad (
    KeypadOpenParens = 0xB6,
    /// Keypad )
    KeypadCloseParens = 0xB7,
    /// Keypad {
    KeypadOpenBrace = 0xB8,
    /// Keypad }
    KeypadCloseBrace = 0xB9,
    /// Keypad Tab
    KeypadTab = 0xBA,
    /// Keypad Backspace
    KeypadBackspace = 0xBB,
    /// Keypad A
    KeypadA = 0xBC,
    /// Keypad B
    KeypadB = 0xBD,
    /// Keypad C
    KeypadC = 0xBE,
    /// Keypad D
    KeypadD = 0xBF,
    /// Keypad E
    KeypadE = 0xC0,
    /// Keypad F
    KeypadF = 0xC1,
    /// Keypad XOR
    KeypadBitwiseXor = 0xC2,
    /// Keypad ^
    KeypadLogicalXor = 0xC3,
    /// Keypad %
    KeypadModulo = 0xC4,
    /// Keypad <
    KeypadLeftShift = 0xC5,
    /// Keypad >
    KeypadRightShift = 0xC6,
    /// Keypad &
    KeypadBitwiseAnd = 0xC7,
    /// Keypad &&
    KeypadLogicalAnd = 0xC8,
    /// Keypad |
    KeypadBitwiseOr = 0xC9,
    /// Keypad ||
    KeypadLogicalOr = 0xCA,
    /// Keypad :
    KeypadColon = 0xCB,
    /// Keypad #
    KeypadHash = 0xCC,
    /// Keypad Space
    KeypadSpace = 0xCD,
    /// Keypad @
    KeypadAt = 0xCE,
    /// Keypad !
    KeypadExclamation = 0xCF,
    /// Keypad Memory Store
    KeypadMemoryStore = 0xD0,
    /// Keypad Memory Recall
    KeypadMemoryRecall = 0xD1,
    /// Keypad Memory Clear
    KeypadMemoryClear = 0xD2,
    /// Keypad Memory Add
    KeypadMemoryAdd = 0xD3,
    /// Keypad Memory Subtract
    KeypadMemorySubtract = 0xD4,
    /// Keypad Memory Multiply
    KeypadMemoryMultiply = 0xD5,
    /// Keypad Memory Divice
    KeypadMemoryDivide = 0xD6,
    /// Keypad +/-
    KeypadPositiveNegative = 0xD7,
    /// Keypad Clear
    KeypadClear = 0xD8,
    /// Keypad Clear Entry
    KeypadClearEntry = 0xD9,
    /// Keypad Binary
    KeypadBinary = 0xDA,
    /// Keypad Octal
    KeypadOctal = 0xDB,
    /// Keypad Decimal
    KeypadDecimal = 0xDC,
    /// Keypad Hexadecimal
    KeypadHexadecimal = 0xDD,
    // 0xDE-0xDF: Reserved
    /// Keyboard LeftControl
    KeyboardLeftControl = 0xE0,
    /// Keyboard LeftShift
    KeyboardLeftShift = 0xE1,
    /// Keyboard LeftAlt
    KeyboardLeftAlt = 0xE2,
    /// Keyboard LeftGUI (Footnote 11) (Footnote 33)
    KeyboardLeftGUI = 0xE3,
    /// Keyboard RightControl
    KeyboardRightControl = 0xE4,
    /// Keyboard RightShift
    KeyboardRightShift = 0xE5,
    /// Keyboard RightAlt
    KeyboardRightAlt = 0xE6,
    /// Keyboard RightGUI (Footnote 11) (Footnote 34)
    KeyboardRightGUI = 0xE7,
    /// Reserved keyboard values (used for all reserved / invalid values)
    Reserved = 0xE8,
    // 0xE8-0xFF: Reserved
}

impl From<u8> for KeyboardUsage {
    fn from(k: u8) -> Self {
        match k {
            0x01 => Self::KeyboardErrorRollOver,
            0x02 => Self::KeyboardPOSTFail,
            0x03 => Self::KeyboardErrorUndefined,
            0x04 => Self::KeyboardAa,
            0x05 => Self::KeyboardBb,
            0x06 => Self::KeyboardCc,
            0x07 => Self::KeyboardDd,
            0x08 => Self::KeyboardEe,
            0x09 => Self::KeyboardFf,
            0x0A => Self::KeyboardGg,
            0x0B => Self::KeyboardHh,
            0x0C => Self::KeyboardIi,
            0x0D => Self::KeyboardJj,
            0x0E => Self::KeyboardKk,
            0x0F => Self::KeyboardLl,
            0x10 => Self::KeyboardMm,
            0x11 => Self::KeyboardNn,
            0x12 => Self::KeyboardOo,
            0x13 => Self::KeyboardPp,
            0x14 => Self::KeyboardQq,
            0x15 => Self::KeyboardRr,
            0x16 => Self::KeyboardSs,
            0x17 => Self::KeyboardTt,
            0x18 => Self::KeyboardUu,
            0x19 => Self::KeyboardVv,
            0x1A => Self::KeyboardWw,
            0x1B => Self::KeyboardXx,
            0x1C => Self::KeyboardYy,
            0x1D => Self::KeyboardZz,
            0x1E => Self::Keyboard1Exclamation,
            0x1F => Self::Keyboard2At,
            0x20 => Self::Keyboard3Hash,
            0x21 => Self::Keyboard4Dollar,
            0x22 => Self::Keyboard5Percent,
            0x23 => Self::Keyboard6Caret,
            0x24 => Self::Keyboard7Ampersand,
            0x25 => Self::Keyboard8Asterisk,
            0x26 => Self::Keyboard9OpenParens,
            0x27 => Self::Keyboard0CloseParens,
            0x28 => Self::KeyboardEnter,
            0x29 => Self::KeyboardEscape,
            0x2A => Self::KeyboardBackspace,
            0x2B => Self::KeyboardTab,
            0x2C => Self::KeyboardSpacebar,
            0x2D => Self::KeyboardDashUnderscore,
            0x2E => Self::KeyboardEqualPlus,
            0x2F => Self::KeyboardOpenBracketBrace,
            0x30 => Self::KeyboardCloseBracketBrace,
            0x31 => Self::KeyboardBackslashBar,
            0x32 => Self::KeyboardNonUSHash,
            0x33 => Self::KeyboardSemiColon,
            0x34 => Self::KeyboardSingleDoubleQuote,
            0x35 => Self::KeyboardBacktickTilde,
            0x36 => Self::KeyboardCommaLess,
            0x37 => Self::KeyboardPeriodGreater,
            0x38 => Self::KeyboardSlashQuestion,
            0x39 => Self::KeyboardCapsLock,
            0x3A => Self::KeyboardF1,
            0x3B => Self::KeyboardF2,
            0x3C => Self::KeyboardF3,
            0x3D => Self::KeyboardF4,
            0x3E => Self::KeyboardF5,
            0x3F => Self::KeyboardF6,
            0x40 => Self::KeyboardF7,
            0x41 => Self::KeyboardF8,
            0x42 => Self::KeyboardF9,
            0x43 => Self::KeyboardF10,
            0x44 => Self::KeyboardF11,
            0x45 => Self::KeyboardF12,
            0x46 => Self::KeyboardPrintScreen,
            0x47 => Self::KeyboardScrollLock,
            0x48 => Self::KeyboardPause,
            0x49 => Self::KeyboardInsert,
            0x4A => Self::KeyboardHome,
            0x4B => Self::KeyboardPageUp,
            0x4C => Self::KeyboardDelete,
            0x4D => Self::KeyboardEnd,
            0x4E => Self::KeyboardPageDown,
            0x4F => Self::KeyboardRightArrow,
            0x50 => Self::KeyboardLeftArrow,
            0x51 => Self::KeyboardDownArrow,
            0x52 => Self::KeyboardUpArrow,
            0x53 => Self::KeypadNumLock,
            0x54 => Self::KeypadDivide,
            0x55 => Self::KeypadMultiply,
            0x56 => Self::KeypadMinus,
            0x57 => Self::KeypadPlus,
            0x58 => Self::KeypadEnter,
            0x59 => Self::Keypad1End,
            0x5A => Self::Keypad2DownArrow,
            0x5B => Self::Keypad3PageDown,
            0x5C => Self::Keypad4LeftArrow,
            0x5D => Self::Keypad5,
            0x5E => Self::Keypad6RightArrow,
            0x5F => Self::Keypad7Home,
            0x60 => Self::Keypad8UpArrow,
            0x61 => Self::Keypad9PageUp,
            0x62 => Self::Keypad0Insert,
            0x63 => Self::KeypadPeriodDelete,
            0x64 => Self::KeyboardNonUSSlash,
            0x65 => Self::KeyboardApplication,
            0x66 => Self::KeyboardPower,
            0x67 => Self::KeypadEqual,
            0x68 => Self::KeyboardF13,
            0x69 => Self::KeyboardF14,
            0x6A => Self::KeyboardF15,
            0x6B => Self::KeyboardF16,
            0x6C => Self::KeyboardF17,
            0x6D => Self::KeyboardF18,
            0x6E => Self::KeyboardF19,
            0x6F => Self::KeyboardF20,
            0x70 => Self::KeyboardF21,
            0x71 => Self::KeyboardF22,
            0x72 => Self::KeyboardF23,
            0x73 => Self::KeyboardF24,
            0x74 => Self::KeyboardExecute,
            0x75 => Self::KeyboardHelp,
            0x76 => Self::KeyboardMenu,
            0x77 => Self::KeyboardSelect,
            0x78 => Self::KeyboardStop,
            0x79 => Self::KeyboardAgain,
            0x7A => Self::KeyboardUndo,
            0x7B => Self::KeyboardCut,
            0x7C => Self::KeyboardCopy,
            0x7D => Self::KeyboardPaste,
            0x7E => Self::KeyboardFind,
            0x7F => Self::KeyboardMute,
            0x80 => Self::KeyboardVolumeUp,
            0x81 => Self::KeyboardVolumeDown,
            0x82 => Self::KeyboardLockingCapsLock,
            0x83 => Self::KeyboardLockingNumLock,
            0x84 => Self::KeyboardLockingScrollLock,
            0x85 => Self::KeypadComma,
            0x86 => Self::KeypadEqualSign,
            0x87 => Self::KeyboardInternational1,
            0x88 => Self::KeyboardInternational2,
            0x89 => Self::KeyboardInternational3,
            0x8A => Self::KeyboardInternational4,
            0x8B => Self::KeyboardInternational5,
            0x8C => Self::KeyboardInternational6,
            0x8D => Self::KeyboardInternational7,
            0x8E => Self::KeyboardInternational8,
            0x8F => Self::KeyboardInternational9,
            0x90 => Self::KeyboardLANG1,
            0x91 => Self::KeyboardLANG2,
            0x92 => Self::KeyboardLANG3,
            0x93 => Self::KeyboardLANG4,
            0x94 => Self::KeyboardLANG5,
            0x95 => Self::KeyboardLANG6,
            0x96 => Self::KeyboardLANG7,
            0x97 => Self::KeyboardLANG8,
            0x98 => Self::KeyboardLANG9,
            0x99 => Self::KeyboardAlternateErase,
            0x9A => Self::KeyboardSysReqAttention,
            0x9B => Self::KeyboardCancel,
            0x9C => Self::KeyboardClear,
            0x9D => Self::KeyboardPrior,
            0x9E => Self::KeyboardReturn,
            0x9F => Self::KeyboardSeparator,
            0xA0 => Self::KeyboardOut,
            0xA1 => Self::KeyboardOper,
            0xA2 => Self::KeyboardClearAgain,
            0xA3 => Self::KeyboardCrSelProps,
            0xA4 => Self::KeyboardExSel,
            0xB0 => Self::Keypad00,
            0xB1 => Self::Keypad000,
            0xB2 => Self::ThousandsSeparator,
            0xB3 => Self::DecimalSeparator,
            0xB4 => Self::CurrencyUnit,
            0xB5 => Self::CurrencySubunit,
            0xB6 => Self::KeypadOpenParens,
            0xB7 => Self::KeypadCloseParens,
            0xB8 => Self::KeypadOpenBrace,
            0xB9 => Self::KeypadCloseBrace,
            0xBA => Self::KeypadTab,
            0xBB => Self::KeypadBackspace,
            0xBC => Self::KeypadA,
            0xBD => Self::KeypadB,
            0xBE => Self::KeypadC,
            0xBF => Self::KeypadD,
            0xC0 => Self::KeypadE,
            0xC1 => Self::KeypadF,
            0xC2 => Self::KeypadBitwiseXor,
            0xC3 => Self::KeypadLogicalXor,
            0xC4 => Self::KeypadModulo,
            0xC5 => Self::KeypadLeftShift,
            0xC6 => Self::KeypadRightShift,
            0xC7 => Self::KeypadBitwiseAnd,
            0xC8 => Self::KeypadLogicalAnd,
            0xC9 => Self::KeypadBitwiseOr,
            0xCA => Self::KeypadLogicalOr,
            0xCB => Self::KeypadColon,
            0xCC => Self::KeypadHash,
            0xCD => Self::KeypadSpace,
            0xCE => Self::KeypadAt,
            0xCF => Self::KeypadExclamation,
            0xD0 => Self::KeypadMemoryStore,
            0xD1 => Self::KeypadMemoryRecall,
            0xD2 => Self::KeypadMemoryClear,
            0xD3 => Self::KeypadMemoryAdd,
            0xD4 => Self::KeypadMemorySubtract,
            0xD5 => Self::KeypadMemoryMultiply,
            0xD6 => Self::KeypadMemoryDivide,
            0xD7 => Self::KeypadPositiveNegative,
            0xD8 => Self::KeypadClear,
            0xD9 => Self::KeypadClearEntry,
            0xDA => Self::KeypadBinary,
            0xDB => Self::KeypadOctal,
            0xDC => Self::KeypadDecimal,
            0xDD => Self::KeypadHexadecimal,
            0xE0 => Self::KeyboardLeftControl,
            0xE1 => Self::KeyboardLeftShift,
            0xE2 => Self::KeyboardLeftAlt,
            0xE3 => Self::KeyboardLeftGUI,
            0xE4 => Self::KeyboardRightControl,
            0xE5 => Self::KeyboardRightShift,
            0xE6 => Self::KeyboardRightAlt,
            0xE7 => Self::KeyboardRightGUI,
            _ => Self::Reserved,
        }
    }
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
#[derive(Clone, Copy, Debug, PartialEq)]
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
    Reserved = 0xEB,
}

impl From<MediaKey> for u16 {
    fn from(mk: MediaKey) -> u16 {
        mk as u16
    }
}

impl From<u8> for MediaKey {
    fn from(k: u8) -> Self {
        match k {
            0x00 => Self::Zero,
            0xB0 => Self::Play,
            0xB1 => Self::Pause,
            0xB2 => Self::Record,
            0xB5 => Self::NextTrack,
            0xB6 => Self::PrevTrack,
            0xB7 => Self::Stop,
            0xB9 => Self::RandomPlay,
            0xBC => Self::Repeat,
            0xCD => Self::PlayPause,
            0xE2 => Self::Mute,
            0xE9 => Self::VolumeIncrement,
            0xEA => Self::VolumeDecrement,
            _ => Self::Reserved,
        }
    }
}

impl From<u16> for MediaKey {
    fn from(k: u16) -> Self {
        (k as u8).into()
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
#[derive(Clone, Copy, Debug, PartialEq)]
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
    // Use this reserved value to represent all reserved keys / invalid values
    Reserved = 0xB8,
}

impl From<SystemControlKey> for u8 {
    fn from(sck: SystemControlKey) -> u8 {
        sck as u8
    }
}

impl From<u8> for SystemControlKey {
    fn from(k: u8) -> Self {
        match k {
            0x81 => Self::PowerDown,
            0x82 => Self::Sleep,
            0x83 => Self::WakeUp,
            0x84 => Self::ContextMenu,
            0x85 => Self::MainMenu,
            0x86 => Self::AppMenu,
            0x87 => Self::MenuHelp,
            0x88 => Self::MenuExit,
            0x89 => Self::MenuSelect,
            0x8A => Self::MenuRight,
            0x8B => Self::MenuLeft,
            0x8C => Self::MenuUp,
            0x8D => Self::MenuDown,
            0x8E => Self::ColdRestart,
            0x8F => Self::WarmRestart,
            0x90 => Self::DpadUp,
            0x91 => Self::DpadDown,
            0x92 => Self::DpadRight,
            0x93 => Self::DpadLeft,
            0x97 => Self::SystemFunctionShift,
            0x98 => Self::SystemFunctionShiftLock,
            0x9A => Self::SystemDismissNotification,
            0x9B => Self::SystemDoNotDisturb,
            0xA0 => Self::Dock,
            0xA1 => Self::Undock,
            0xA2 => Self::Setup,
            0xA3 => Self::Break,
            0xA4 => Self::DebuggerBreak,
            0xA5 => Self::ApplicationBreak,
            0xA6 => Self::ApplicationDebuggerBreak,
            0xA7 => Self::SpeakerMute,
            0xA8 => Self::Hibernate,
            0xB0 => Self::DisplayInvert,
            0xB1 => Self::DisplayInternal,
            0xB2 => Self::DisplayExternal,
            0xB3 => Self::DisplayBoth,
            0xB4 => Self::DisplayDual,
            0xB5 => Self::DisplayToggleInternalExternal,
            0xB6 => Self::DisplaySwapPrimarySecondary,
            0xB7 => Self::DisplayLcdAutoscale,
            _ => Self::Reserved,
        }
    }
}
