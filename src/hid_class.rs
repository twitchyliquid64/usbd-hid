//! Implements HID functionality for a usb-device device.
use usb_device::class_prelude::*;
use usb_device::Result;

use crate::descriptor::AsInputReport;
extern crate ssmarshal;
use ssmarshal::serialize;

const USB_CLASS_HID: u8 = 0x03;

// HID
const HID_DESC_DESCTYPE_HID: u8 = 0x21;
const HID_DESC_DESCTYPE_HID_REPORT: u8 = 0x22;
const HID_DESC_SPEC_1_10: [u8; 2] = [0x10, 0x01];

const HID_REQ_GET_IDLE: u8 = 0x02;
const HID_REQ_SET_IDLE: u8 = 0x0a;
const HID_REQ_GET_PROTOCOL: u8 = 0x03;
const HID_REQ_SET_PROTOCOL: u8 = 0x0b;
const HID_REQ_GET_REPORT: u8 = 0x01;
const HID_REQ_SET_REPORT: u8 = 0x09;

#[derive(Copy, Clone, Debug, PartialEq)]
#[repr(u8)]
pub enum HidCountryCode {
    NotSupported = 0,
    Arabic = 1,
    Belgian = 2,
    CanadianBilingual = 3,
    CanadianFrench = 4,
    CzechRepublic = 5,
    Danish = 6,
    Finnish = 7,
    French = 8,
    German = 9,
    Greek = 10,
    Hebrew = 11,
    Hungary = 12,
    InternationalISO = 13,
    Italian = 14,
    JapanKatakana = 15,
    Korean = 16,
    LatinAmerica = 17,
    NetherlandsDutch = 18,
    Norwegian = 19,
    PersianFarsi = 20,
    Poland = 21,
    Portuguese = 22,
    Russia = 23,
    Slovakia = 24,
    Spanish = 25,
    Swedish = 26,
    SwissFrench = 27,
    SwissGerman = 28,
    Switzerland = 29,
    Taiwan = 30,
    TurkishQ = 31,
    UK = 32,
    US = 33,
    Yugoslavia = 34,
    TurkishF = 35,
}

#[derive(Copy, Clone, Debug, PartialEq)]
#[repr(u8)]
pub enum HidSubClass {
    NoSubclass = 0,
    Boot = 1,
}

#[derive(Copy, Clone, Debug, PartialEq)]
#[repr(u8)]
pub enum HidProtocol {
    Generic = 0,
    Keyboard = 1,
    Mouse = 2,
}

#[derive(Copy, Clone, Debug, PartialEq)]
#[repr(u8)]
pub enum HidProtocolMode {
    Report = 0,
    Boot = 1,
}

impl From<u8> for HidProtocolMode {
    fn from(mode: u8) -> HidProtocolMode {
        if mode == 1 {
            HidProtocolMode::Boot
        } else {
            HidProtocolMode::Report
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ProtocolModeConfig {
    /// Allows the host to define boot or report mode. Defaults to report mode.
    DefaultBehavior,
    /// Forces protocol mode to boot mode
    ForceBoot,
    /// Forces protocol mode to report mode
    ForceReport,
}

pub struct HidClassSettings {
    pub subclass: HidSubClass,
    pub protocol: HidProtocol,
    pub config: ProtocolModeConfig,
    pub locale: HidCountryCode,
}

impl Default for HidClassSettings {
    fn default() -> Self {
        Self {
            subclass: HidSubClass::NoSubclass,
            protocol: HidProtocol::Generic,
            config: ProtocolModeConfig::DefaultBehavior,
            locale: HidCountryCode::NotSupported,
        }
    }
}

/// HIDClass provides an interface to declare, read & write HID reports.
///
/// Users are expected to provide the report descriptor, as well as pack
/// and unpack reports which are read or staged for transmission.
pub struct HIDClass<'a, B: UsbBus> {
    if_num: InterfaceNumber,
    out_ep: Option<EndpointOut<'a, B>>,
    in_ep: Option<EndpointIn<'a, B>>,
    report_descriptor: &'static [u8],
    /// Used only by Keyboard and Mouse to define BIOS (Boot) mode vs Normal (Report) mode.
    /// This is used to switch between 6KRO (boot) and NKRO (report) endpoints.
    /// Boot mode configured endpoints may not parse the hid descriptor and expect an exact
    /// hid packet format. By default a device should start in normal (report) mode and the host
    /// must request the boot mode protocol if it requires it.
    ///
    /// If a device does not request boot mode, this is a host bug. For convenience this API allows
    /// manually setting the protocol.
    /// See https://www.usb.org/sites/default/files/hid1_11.pdf Section 7.2.6
    protocol: Option<HidProtocolMode>,
    settings: HidClassSettings,
}

fn determine_protocol_setting(settings: &HidClassSettings) -> Option<HidProtocolMode> {
    if settings.protocol == HidProtocol::Keyboard || settings.protocol == HidProtocol::Mouse {
        match settings.config {
            ProtocolModeConfig::DefaultBehavior | ProtocolModeConfig::ForceReport => {
                Some(HidProtocolMode::Report)
            }
            ProtocolModeConfig::ForceBoot => Some(HidProtocolMode::Boot),
        }
    } else {
        None
    }
}

impl<B: UsbBus> HIDClass<'_, B> {
    /// Creates a new HIDClass with the provided UsbBus & HID report descriptor.
    ///
    /// poll_ms configures how frequently the host should poll for reading/writing
    /// HID reports. A lower value means better throughput & latency, at the expense
    /// of CPU on the device & bandwidth on the bus. A value of 10 is reasonable for
    /// high performance uses, and a value of 255 is good for best-effort usecases.
    ///
    /// This allocates two endpoints (IN and OUT).
    /// See new_ep_in (IN endpoint only) and new_ep_out (OUT endpoint only) to only create a single
    /// endpoint.
    pub fn new<'a>(
        alloc: &'a UsbBusAllocator<B>,
        report_descriptor: &'static [u8],
        poll_ms: u8,
        settings: HidClassSettings,
    ) -> HIDClass<'a, B> {
        HIDClass {
            if_num: alloc.interface(),
            out_ep: Some(alloc.interrupt(64, poll_ms)),
            in_ep: Some(alloc.interrupt(64, poll_ms)),
            report_descriptor,
            protocol: determine_protocol_setting(&settings),
            settings,
        }
    }

    /// Creates a new HIDClass with the provided UsbBus & HID report descriptor.
    /// See new() for more details.
    pub fn new_ep_in<'a>(
        alloc: &'a UsbBusAllocator<B>,
        report_descriptor: &'static [u8],
        poll_ms: u8,
        settings: HidClassSettings,
    ) -> HIDClass<'a, B> {
        HIDClass {
            if_num: alloc.interface(),
            out_ep: None,
            in_ep: Some(alloc.interrupt(64, poll_ms)),
            report_descriptor,
            protocol: determine_protocol_setting(&settings),
            settings,
        }
    }

    /// Creates a new HIDClass with the provided UsbBus & HID report descriptor.
    /// See new() for more details.
    pub fn new_ep_out<'a>(
        alloc: &'a UsbBusAllocator<B>,
        report_descriptor: &'static [u8],
        poll_ms: u8,
        settings: HidClassSettings,
    ) -> HIDClass<'a, B> {
        HIDClass {
            if_num: alloc.interface(),
            out_ep: Some(alloc.interrupt(64, poll_ms)),
            in_ep: None,
            report_descriptor,
            protocol: determine_protocol_setting(&settings),
            settings,
        }
    }

    /// Tries to write an input report by serializing the given report structure.
    /// A BufferOverflow error is returned if the serialized report is greater than
    /// 64 bytes in size.
    pub fn push_input<IR: AsInputReport>(&self, r: &IR) -> Result<usize> {
        // Do not push data if protocol settings do not match (only for keyboard and mouse)
        match self.settings.protocol {
            HidProtocol::Keyboard | HidProtocol::Mouse => {
                if let Some(protocol) = self.protocol {
                    if (protocol == HidProtocolMode::Report
                        && self.settings.subclass != HidSubClass::NoSubclass)
                        || (protocol == HidProtocolMode::Boot
                            && self.settings.subclass != HidSubClass::Boot)
                    {
                        return Err(UsbError::InvalidState);
                    }
                }
            }
            _ => {}
        }

        if let Some(ep) = &self.in_ep {
            let mut buff: [u8; 64] = [0; 64];
            let size = match serialize(&mut buff, r) {
                Ok(l) => l,
                Err(_) => return Err(UsbError::BufferOverflow),
            };
            ep.write(&buff[0..size])
        } else {
            Err(UsbError::InvalidEndpoint)
        }
    }

    /// Tries to write an input (device-to-host) report from the given raw bytes.
    /// Data is expected to be a valid HID report for INPUT items. If report ID's
    /// were used in the descriptor, the report ID corresponding to this report
    /// must be be present before the contents of the report.
    pub fn push_raw_input(&self, data: &[u8]) -> Result<usize> {
        // Do not push data if protocol settings do not match (only for keyboard and mouse)
        match self.settings.protocol {
            HidProtocol::Keyboard | HidProtocol::Mouse => {
                if let Some(protocol) = self.protocol {
                    if (protocol == HidProtocolMode::Report
                        && self.settings.subclass != HidSubClass::NoSubclass)
                        || (protocol == HidProtocolMode::Boot
                            && self.settings.subclass != HidSubClass::Boot)
                    {
                        return Err(UsbError::InvalidState);
                    }
                }
            }
            _ => {}
        }

        if let Some(ep) = &self.in_ep {
            ep.write(data)
        } else {
            Err(UsbError::InvalidEndpoint)
        }
    }

    /// Tries to read an output (host-to-device) report as raw bytes. Data
    /// is expected to be sized appropriately to contain any valid HID report
    /// for OUTPUT items, including the report ID prefix if report IDs are used.
    pub fn pull_raw_output(&self, data: &mut [u8]) -> Result<usize> {
        if let Some(ep) = &self.out_ep {
            ep.read(data)
        } else {
            Err(UsbError::InvalidEndpoint)
        }
    }

    pub fn get_protocol_mode(&self) -> Result<HidProtocolMode> {
        // Protocol mode only has meaning if Keyboard or Mouse Protocol is set
        match self.settings.protocol {
            HidProtocol::Keyboard | HidProtocol::Mouse => {}
            _ => {
                return Err(UsbError::Unsupported);
            }
        }

        if let Some(protocol) = self.protocol {
            Ok(protocol)
        } else {
            Err(UsbError::InvalidState)
        }
    }

    pub fn set_protocol_mode(
        &mut self,
        mode: HidProtocolMode,
        config: ProtocolModeConfig,
    ) -> Result<()> {
        // Protocol mode only has meaning if Keyboard or Mouse Protocol is set
        match self.settings.protocol {
            HidProtocol::Keyboard | HidProtocol::Mouse => {}
            _ => {
                return Err(UsbError::Unsupported);
            }
        }

        // Update the protocol setting behavior and update the protocol mode
        match config {
            ProtocolModeConfig::DefaultBehavior => self.protocol = Some(mode),
            ProtocolModeConfig::ForceBoot => {
                self.protocol = Some(HidProtocolMode::Boot);
            }
            ProtocolModeConfig::ForceReport => {
                self.protocol = Some(HidProtocolMode::Report);
            }
        }
        self.settings.config = config;
        Ok(())
    }
}

impl<B: UsbBus> UsbClass<B> for HIDClass<'_, B> {
    fn get_configuration_descriptors(&self, writer: &mut DescriptorWriter) -> Result<()> {
        writer.interface(
            self.if_num,
            USB_CLASS_HID,
            self.settings.subclass as u8,
            self.settings.protocol as u8,
        )?;

        // HID descriptor
        writer.write(
            HID_DESC_DESCTYPE_HID,
            &[
                // HID Class spec version
                HID_DESC_SPEC_1_10[0],
                HID_DESC_SPEC_1_10[1],
                self.settings.locale as u8,
                // Number of following descriptors
                1,
                // We have a HID report descriptor the host should read
                HID_DESC_DESCTYPE_HID_REPORT,
                // HID report descriptor size,
                (self.report_descriptor.len() & 0xFF) as u8,
                (self.report_descriptor.len() >> 8 & 0xFF) as u8,
            ],
        )?;

        if let Some(ep) = &self.out_ep {
            writer.endpoint(&ep)?;
        }
        if let Some(ep) = &self.in_ep {
            writer.endpoint(&ep)?;
        }
        Ok(())
    }

    // Handle control requests to the host.
    fn control_in(&mut self, xfer: ControlIn<B>) {
        let req = xfer.request();

        // Bail out if its not relevant to our interface.
        if req.index != u8::from(self.if_num) as u16 {
            return;
        }

        match (req.request_type, req.request) {
            (control::RequestType::Standard, control::Request::GET_DESCRIPTOR) => {
                match (req.value >> 8) as u8 {
                    HID_DESC_DESCTYPE_HID_REPORT => {
                        xfer.accept_with_static(self.report_descriptor).ok();
                    }
                    HID_DESC_DESCTYPE_HID => {
                        let buf = &[
                            // Length of buf inclusive of size prefix
                            9,
                            // Descriptor type
                            HID_DESC_DESCTYPE_HID,
                            // HID Class spec version
                            HID_DESC_SPEC_1_10[0],
                            HID_DESC_SPEC_1_10[1],
                            self.settings.locale as u8,
                            // Number of following descriptors
                            1,
                            // We have a HID report descriptor the host should read
                            HID_DESC_DESCTYPE_HID_REPORT,
                            // HID report descriptor size,
                            (self.report_descriptor.len() & 0xFF) as u8,
                            (self.report_descriptor.len() >> 8 & 0xFF) as u8,
                        ];
                        xfer.accept_with(buf).ok();
                    }
                    _ => {}
                }
            }
            (control::RequestType::Class, HID_REQ_GET_IDLE) => {
                xfer.reject().ok(); // Not supported for now
            }
            (control::RequestType::Class, HID_REQ_GET_PROTOCOL) => {
                // Only accept in supported configurations
                if let Some(protocol) = self.protocol {
                    xfer.accept_with(&[protocol as u8]).ok();
                } else {
                    xfer.reject().ok();
                }
            }
            (control::RequestType::Class, HID_REQ_GET_REPORT) => {
                xfer.reject().ok(); // Not supported for now
            }
            _ => {}
        }
    }

    // Handle a control request from the host.
    fn control_out(&mut self, xfer: ControlOut<B>) {
        let req = xfer.request();

        // Bail out if its not relevant to our interface.
        if !(req.recipient == control::Recipient::Interface
            && req.index == u8::from(self.if_num) as u16)
        {
            return;
        }

        match req.request {
            HID_REQ_SET_IDLE => {
                xfer.accept().ok();
            }
            HID_REQ_SET_PROTOCOL => {
                // Only accept in supported configurations
                if let Some(_protocol) = self.protocol {
                    // Only set if configured to
                    if self.settings.config == ProtocolModeConfig::DefaultBehavior {
                        self.protocol = Some(((req.value & 0xFF) as u8).into());
                    }
                    xfer.accept().ok();
                } else {
                    xfer.reject().ok();
                }
            }
            HID_REQ_SET_REPORT => {
                xfer.reject().ok(); // Not supported for now
            }
            _ => {
                xfer.reject().ok();
            }
        }
    }
}
