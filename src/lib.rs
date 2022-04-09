//! HID report descriptor generation & USB HID class implementation
//!
//! This crate implements components necessary to build a USB HID device. This
//! includes generation of the report descriptor, serialization of input reports,
//! and communicating with a host that implements USB HID.
#![no_std]

pub use usb_device::{Result, UsbError};
pub mod descriptor;
pub mod hid_class;

#[cfg(test)]
#[allow(unused_imports)]
mod tests {
    use crate::descriptor::generator_prelude::*;
    use crate::descriptor::{KeyboardReport, MouseReport, SystemControlReport};

    // This should generate this descriptor:
    // 0x06, 0x00, 0xFF,  // Usage Page (Vendor Defined 0xFF00)
    // 0x09, 0x01,        // Usage (0x01)
    // 0xA1, 0x01,        // Collection (Application)
    // 0x15, 0x00,        //   Logical Minimum (0)
    // 0x26, 0xFF, 0x00,  //   Logical Maximum (255)
    // 0x75, 0x08,        //   Report Size (8)
    // 0x95, 0x01,        //   Report Count (1)
    // 0x81, 0x02,        //   Input (Data,Var,Abs,No Wrap,Linear,Preferred State,No Null Position)
    // 0x27, 0xFF, 0xFF, 0x00, 0x00,  //   Logical Maximum (65534)
    // 0x75, 0x10,        //   Report Size (16)
    // 0x91, 0x02,        //   Output (Data,Var,Abs,No Wrap,Linear,Preferred State,No Null Position,Non-volatile)
    // 0xC1,              // End Collection
    #[gen_hid_descriptor(
        (collection = 0x01, usage = 0x01, usage_page = 0xff00) = {
            f1=input;
            f2=output;
        }
    )]
    #[allow(dead_code)]
    struct CustomUnaryUnsignedFrame {
        f1: u8,
        f2: u16,
    }

    #[test]
    fn test_custom_unsigned() {
        let expected = &[
            6u8, 0u8, 255u8, 9u8, 1u8, 161u8, 1u8, 21u8, 0u8, 38u8, 255u8, 0u8, 117u8, 8u8, 149u8,
            1u8, 129u8, 2u8, 39u8, 255u8, 255u8, 0u8, 0u8, 117u8, 16u8, 145u8, 2u8, 192u8,
        ];
        assert_eq!(CustomUnaryUnsignedFrame::desc(), expected);
    }

    // This should generate this descriptor:
    // 0x06, 0x00, 0xFF,                // Usage Page (Vendor Defined 0xFF00)
    // 0x09, 0x01,                      // Usage (0x01)
    // 0xA1, 0x01,                      // Collection (Application)
    // 0x17, 0x81, 0xFF, 0xFF, 0xFF,    //   Logical Minimum (-128)
    // 0x25, 0x7F,                      //   Logical Maximum (127)
    // 0x75, 0x08,                      //   Report Size (8)
    // 0x95, 0x01,                      //   Report Count (1)
    // 0x81, 0x02,                      //   Input (Data,Var,Abs,No Wrap,Linear,Preferred State,No Null Position)
    // 0x17, 0x01, 0x80, 0xFF, 0xFF,    //   Logical Minimum (-32768)
    // 0x26, 0xFF, 0x7F,                //   Logical Maximum (32767)
    // 0x75, 0x10,                      //   Report Size (16)
    // 0x91, 0x02,                      //   Output (Data,Var,Abs,No Wrap,Linear,Preferred State,No Null Position,Non-volatile)
    // 0xC0,                            // End Collection
    #[gen_hid_descriptor(
        (collection = 0x01, usage = 0x01, usage_page = 0xff00) = {
            f1=input;
            f2=output;
        }
    )]
    #[allow(dead_code)]
    struct CustomUnarySignedFrame {
        f1: i8,
        f2: i16,
    }

    #[test]
    fn test_custom_signed() {
        let expected = &[
            6u8, 0u8, 255u8, 9u8, 1u8, 161u8, 1u8, 23u8, 129u8, 255u8, 255u8, 255u8, 37u8, 127u8,
            117u8, 8u8, 149u8, 1u8, 129u8, 2u8, 23u8, 1u8, 128u8, 255u8, 255u8, 38u8, 255u8, 127u8,
            117u8, 16u8, 145u8, 2u8, 192u8,
        ];
        assert_eq!(CustomUnarySignedFrame::desc()[0..32], expected[0..32]);
    }

    #[gen_hid_descriptor(
        (report_id = 0x01,) = {
            f1=input
        },
        (report_id = 0x02,) = {
            f2=input
        },
    )]
    #[allow(dead_code)]
    struct CustomMultiReport {
        f1: u8,
        f2: u8,
    }

    #[test]
    fn test_custom_reports() {
        let expected: &[u8] = &[
            133, 1, 21, 0, 38, 255, 0, 117, 8, 149, 1, 129, 2, 133, 2, 129, 2,
        ];
        assert_eq!(CustomMultiReport::desc(), expected);
    }

    // This should generate the following descriptor:
    // 0x06, 0x00, 0xFF,  // Usage Page (Vendor Defined 0xFF00)
    // 0x09, 0x01,        // Usage (0x01)
    // 0xA1, 0x01,        // Collection (Application)
    // 0x15, 0x00,        //   Logical Minimum (0)
    // 0x26, 0xFF, 0x00,  //   Logical Maximum (255)
    // 0x75, 0x08,        //   Report Size (8)
    // 0x95, 0x20,        //   Report Count (32)
    // 0x81, 0x02,        //   Input (Data,Var,Abs,No Wrap,Linear,Preferred State,No Null Position)
    // 0xC0,              // End Collection
    #[gen_hid_descriptor(
        (collection = 0x01, usage = 0x01, usage_page = 0xff00) = {
            buff=input;
        }
    )]
    #[allow(dead_code)]
    struct CustomArray {
        buff: [u8; 32],
    }

    #[test]
    fn test_array() {
        let expected: &[u8] = &[
            6, 0, 255, 9, 1, 161, 1, 21, 0, 38, 255, 0, 117, 8, 149, 32, 129, 2, 192,
        ];
        assert_eq!(CustomArray::desc(), expected);
    }

    #[gen_hid_descriptor(
        (collection = APPLICATION, usage_page = VENDOR_DEFINED_START, usage = 0x01) = {
            (usage_min = BUTTON_1, usage_max = BUTTON_3) = {
                #[item_settings data,variable,relative] f1=input;
            };
        }
    )]
    #[allow(dead_code)]
    struct CustomConst {
        f1: u8,
    }

    #[test]
    fn test_custom_const() {
        let expected = &[
            6u8, 0u8, 255u8, 9u8, 1u8, 161u8, 1u8, 25u8, 1u8, 41u8, 3u8, 21u8, 0u8, 38u8, 255u8,
            0u8, 117u8, 8u8, 149u8, 1u8, 129u8, 6u8, 192u8,
        ];
        assert_eq!(CustomConst::desc(), expected);
    }

    // This should generate the following descriptor:
    // 0x85, 0x01,        // Report ID (1)
    // 0x15, 0x00,        // Logical Minimum (0)
    // 0x25, 0x01,        // Logical Maximum (1)
    // 0x75, 0x01,        // Report Size (1)
    // 0x95, 0x03,        // Report Count (3)
    // 0x81, 0x02,        // Input (Data,Var,Abs,No Wrap,Linear,Preferred State,No Null Position)
    // 0x95, 0x05,        // Report Count (5)
    // 0x81, 0x03,        // Input (Const,Var,Abs,No Wrap,Linear,Preferred State,No Null Position)
    // 0x95, 0x09,        // Report Count (9)
    // 0x81, 0x02,        // Input (Data,Var,Abs,No Wrap,Linear,Preferred State,No Null Position)
    // 0x95, 0x07,        // Report Count (7)
    // 0x81, 0x03,        // Input (Const,Var,Abs,No Wrap,Linear,Preferred State,No Null Position)
    // 0x95, 0x14,        // Report Count (20)
    // 0x81, 0x02,        // Input (Data,Var,Abs,No Wrap,Linear,Preferred State,No Null Position)
    // 0x95, 0x04,        // Report Count (4)
    // 0x81, 0x03,        // Input (Const,Var,Abs,No Wrap,Linear,Preferred State,No Null Position)
    #[gen_hid_descriptor(
        (report_id = 0x01,) = {
            #[packed_bits 3] f1=input;
            #[packed_bits 9] f2=input;
            #[packed_bits 20] f3=input;
        }
    )]
    #[allow(dead_code)]
    struct CustomPackedBits {
        f1: u8,
        f2: u16,
        f3: [u8; 3],
    }

    #[test]
    fn test_custom_packed_bits() {
        let expected = &[
            133u8, 1u8, 21u8, 0u8, 37u8, 1u8, 117u8, 1u8, 149u8, 3u8, 129u8, 2u8, 149u8, 5u8,
            129u8, 3u8, 149u8, 9u8, 129u8, 2u8, 149u8, 7u8, 129u8, 3u8, 149u8, 20u8, 129u8, 2u8,
            149u8, 4u8, 129u8, 3u8,
        ];
        assert_eq!(CustomPackedBits::desc(), expected);
    }

    #[test]
    fn test_mouse_descriptor() {
        let expected = &[
            5u8, 1u8, 9u8, 2u8, 161u8, 1u8, 9u8, 1u8, 161u8, 0u8, 5u8, 9u8, 25u8, 1u8, 41u8, 8u8,
            21u8, 0u8, 37u8, 1u8, 117u8, 1u8, 149u8, 8u8, 129u8, 2u8, 5u8, 1u8, 9u8, 48u8, 23u8,
            129u8, 255u8, 255u8, 255u8, 37u8, 127u8, 117u8, 8u8, 149u8, 1u8, 129u8, 6u8, 9u8, 49u8,
            129u8, 6u8, 9u8, 56u8, 129u8, 6u8, 5u8, 12u8, 10u8, 56u8, 2u8, 129u8, 6u8, 192u8,
            192u8,
        ];
        assert_eq!(MouseReport::desc()[0..32], expected[0..32]);
    }

    #[test]
    fn test_keyboard_descriptor() {
        let expected = &[
            0x05, 0x01, // Usage Page (Generic Desktop)
            0x09, 0x06, // Usage (Keyboard)
            0xa1, 0x01, // Collection (Application)
            0x05, 0x07, // Usage Page (Key Codes)
            0x19, 0xe0, // Usage Minimum (224)
            0x29, 0xe7, // Usage Maximum (231)
            0x15, 0x00, // Logical Minimum (0)
            0x25, 0x01, // Logical Maximum (1)
            0x75, 0x01, // Report Size (1)
            0x95, 0x08, // Report count (8)
            0x81, 0x02, // Input (Data, Variable, Absolute)
            0x19, 0x00, // Usage Minimum (0)
            0x29, 0xFF, // Usage Maximum (255)
            0x26, 0xFF, 0x00, // Logical Maximum (255)
            0x75, 0x08, // Report Size (8)
            0x95, 0x01, // Report Count (1)
            0x81, 0x03, // Input (Const, Variable, Absolute)
            0x05, 0x08, // Usage Page (LEDs)
            0x19, 0x01, // Usage Minimum (1)
            0x29, 0x05, // Usage Maximum (5)
            0x25, 0x01, // Logical Maximum (1)
            0x75, 0x01, // Report Size (1)
            0x95, 0x05, // Report Count (5)
            0x91, 0x02, // Output (Data, Variable, Absolute)
            0x95, 0x03, // Report Count (3)
            0x91, 0x03, // Output (Constant, Variable, Absolute)
            0x05, 0x07, // Usage Page (Key Codes)
            0x19, 0x00, // Usage Minimum (0)
            0x29, 0xDD, // Usage Maximum (221)
            0x26, 0xFF, 0x00, // Logical Maximum (255)
            0x75, 0x08, // Report Size (8)
            0x95, 0x06, // Report Count (6)
            0x81, 0x00, // Input (Data, Array, Absolute)
            0xc0, // End Collection
        ];
        assert_eq!(KeyboardReport::desc(), expected);
    }

    #[test]
    fn test_system_control_descriptor() {
        let expected = &[
            0x05, 0x01, // Usage Page (Generic Desktop Ctrls)
            0x09, 0x80, // Usage (Sys Control)
            0xA1, 0x01, // Collection (Application)
            0x19, 0x81, //   Usage Minimum (Sys Power Down)
            0x29, 0xB7, //   Usage Maximum (Sys Display LCD Autoscale)
            0x15, 0x01, //   Logical Minimum (1)
            0x26, 0xFF, 0x00, //   Logical Maximum (255)
            0x75, 0x08, //   Report Size (8)
            0x95, 0x01, //   Report Count (1)
            0x81,
            0x00, //   Input (Data,Array,Abs,No Wrap,Linear,Preferred State,No Null Position)
            0xC0, // End Collection
        ];
        assert_eq!(SystemControlReport::desc(), expected);
    }
}
