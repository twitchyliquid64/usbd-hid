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

/// Requests the set idle rate from the device
/// See (7.2.3): <https://www.usb.org/sites/default/files/hid1_11.pdf>
const HID_REQ_GET_IDLE: u8 = 0x02;

/// Requests device to not send a particular report until a new event occurs
/// or the specified amount of time passes.
/// See (7.2.4): <https://www.usb.org/sites/default/files/hid1_11.pdf>
const HID_REQ_SET_IDLE: u8 = 0x0a;

/// Requests the active protocol on the device (boot or report)
/// See (7.2.5): <https://www.usb.org/sites/default/files/hid1_11.pdf>
const HID_REQ_GET_PROTOCOL: u8 = 0x03;

/// Switches the device between boot and report protocols. Devices must default
/// to report protocol, it is the reponsibility of the host to set the device
/// to boot protocol (NOTE: Sadly many OSs, BIOSs and bootloaders do not adhere
/// to the USB spec here).
/// See (7.2.6): <https://www.usb.org/sites/default/files/hid1_11.pdf>
const HID_REQ_SET_PROTOCOL: u8 = 0x0b;

/// Allows a host to receive a report via the Control pipe
/// See (7.2.1): <https://www.usb.org/sites/default/files/hid1_11.pdf>
const HID_REQ_GET_REPORT: u8 = 0x01;

/// Allows the host to send a report to the device via the Control pipe
/// See (7.2.2): <https://www.usb.org/sites/default/files/hid1_11.pdf>
const HID_REQ_SET_REPORT: u8 = 0x09;

/// See CONTROL_BUF_LEN from usb-device.git src/control_pipe.rs
/// Will need to revisit how this is set once usb-device has true HiSpeed USB support.
const CONTROL_BUF_LEN: usize = 128;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum ReportType {
    Input = 1,
    Output = 2,
    Feature = 3,
    Reserved,
}

impl From<u8> for ReportType {
    fn from(rt: u8) -> ReportType {
        match rt {
            1 => ReportType::Input,
            2 => ReportType::Output,
            3 => ReportType::Feature,
            _ => ReportType::Reserved,
        }
    }
}

#[derive(Copy, Clone, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct ReportInfo {
    pub report_type: ReportType,
    pub report_id: u8,
    pub len: usize,
}

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
struct Report {
    info: ReportInfo,
    buf: [u8; CONTROL_BUF_LEN],
}

/// List of official USB HID country codes
/// See (6.2.1): <https://www.usb.org/sites/default/files/hid1_11.pdf>
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
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

/// Used to enable Boot mode descriptors for Mouse and Keyboard devices.
/// See (4.2): <https://www.usb.org/sites/default/files/hid1_11.pdf>
/// Boot mode descriptors are fixed and must follow a strict format.
/// See (Appendix F): <https://www.usb.org/sites/default/files/hid1_11.pdf>
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
pub enum HidSubClass {
    NoSubClass = 0,
    Boot = 1,
}

/// Defines fixed packet format
/// Only used if HidSubClass::Boot(1) is set
/// See (4.3): <https://www.usb.org/sites/default/files/hid1_11.pdf>
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
pub enum HidProtocol {
    Generic = 0,
    Keyboard = 1,
    Mouse = 2,
}

/// Get/Set Protocol mapping
/// See (7.2.5 and 7.2.6): <https://www.usb.org/sites/default/files/hid1_11.pdf>
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
pub enum HidProtocolMode {
    Boot = 0,
    Report = 1,
}

impl From<u8> for HidProtocolMode {
    fn from(mode: u8) -> HidProtocolMode {
        if mode == HidProtocolMode::Boot as u8 {
            HidProtocolMode::Boot
        } else {
            HidProtocolMode::Report
        }
    }
}

/// It is often necessary to override OS behavior in order to get around OS (and application) level
/// bugs. Forcing either Boot mode (6KRO) and Report mode (NKRO) are often necessary for NKRO
/// compatible keyboards. Mice that support boot mode are not common and generally only useful for
/// legacy OSs.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum ProtocolModeConfig {
    /// Allows the host to define boot or report mode. Defaults to report mode.
    DefaultBehavior,
    /// Forces protocol mode to boot mode
    ForceBoot,
    /// Forces protocol mode to report mode
    ForceReport,
}

/// Used to define specialized HID device settings
/// Most commonly used to setup Boot Mode (6KRO) or Report Mode (NKRO) keyboards.
/// Some OSs will also respect the HID locale setting of the keyboard to help choose the OS
/// keyboard layout.
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct HidClassSettings {
    pub subclass: HidSubClass,
    pub protocol: HidProtocol,
    pub config: ProtocolModeConfig,
    pub locale: HidCountryCode,
}

impl Default for HidClassSettings {
    fn default() -> Self {
        Self {
            subclass: HidSubClass::NoSubClass,
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
    /// Low-latency OUT buffer
    out_ep: Option<EndpointOut<'a, B>>,
    /// Low-latency IN buffer
    in_ep: Option<EndpointIn<'a, B>>,
    report_descriptor: &'static [u8],
    /// Control endpoint alternative OUT buffer (always used for setting feature reports)
    /// See: <https://www.usb.org/sites/default/files/documents/hid1_11.pdf> 7.2.1 and 7.2.2
    set_report_buf: Option<Report>,
    /// Used only by Keyboard and Mouse to define BIOS (Boot) mode vs Normal (Report) mode.
    /// This is used to switch between 6KRO (boot) and NKRO (report) endpoints.
    /// Boot mode configured endpoints may not parse the hid descriptor and expect an exact
    /// hid packet format. By default a device should start in normal (report) mode and the host
    /// must request the boot mode protocol if it requires it.
    ///
    /// If a device does not request boot mode, this is a host bug. For convenience this API allows
    /// manually setting the protocol.
    /// See <https://www.usb.org/sites/default/files/hid1_11.pdf> Section 7.2.6
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
    ///
    /// See new_with_settings() if you need to define protocol or locale settings for a IN/OUT
    /// HID interface.
    pub fn new<'a>(
        alloc: &'a UsbBusAllocator<B>,
        report_descriptor: &'static [u8],
        poll_ms: u8,
    ) -> HIDClass<'a, B> {
        let settings = HidClassSettings::default();
        HIDClass {
            if_num: alloc.interface(),
            out_ep: Some(alloc.interrupt(64, poll_ms)),
            in_ep: Some(alloc.interrupt(64, poll_ms)),
            report_descriptor,
            set_report_buf: None,
            protocol: determine_protocol_setting(&settings),
            settings,
        }
    }

    /// Same as new() but includes a settings field.
    /// The settings field is used to define both locale and protocol settings of the HID
    /// device (needed for HID keyboard and Mice).
    pub fn new_with_settings<'a>(
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
            set_report_buf: None,
            protocol: determine_protocol_setting(&settings),
            settings,
        }
    }

    /// Creates a new HIDClass with the provided UsbBus & HID report descriptor.
    /// See new() for more details.
    /// Please use new_ep_in_with_settings() if you are creating a keyboard or mouse.
    pub fn new_ep_in<'a>(
        alloc: &'a UsbBusAllocator<B>,
        report_descriptor: &'static [u8],
        poll_ms: u8,
    ) -> HIDClass<'a, B> {
        let settings = HidClassSettings::default();
        HIDClass {
            if_num: alloc.interface(),
            out_ep: None,
            in_ep: Some(alloc.interrupt(64, poll_ms)),
            report_descriptor,
            set_report_buf: None,
            protocol: determine_protocol_setting(&settings),
            settings,
        }
    }

    /// Same as new_ep_in() but includes a settings field.
    /// The settings field is used to define both locale and protocol settings of the HID
    /// device (needed for HID keyboard and Mice).
    pub fn new_ep_in_with_settings<'a>(
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
            set_report_buf: None,
            protocol: determine_protocol_setting(&settings),
            settings,
        }
    }

    /// Creates a new HIDClass with the provided UsbBus & HID report descriptor.
    /// See new() for more details.
    /// Please use new_ep_out_with_settings if you need the settings field.
    pub fn new_ep_out<'a>(
        alloc: &'a UsbBusAllocator<B>,
        report_descriptor: &'static [u8],
        poll_ms: u8,
    ) -> HIDClass<'a, B> {
        let settings = HidClassSettings::default();
        HIDClass {
            if_num: alloc.interface(),
            out_ep: Some(alloc.interrupt(64, poll_ms)),
            in_ep: None,
            report_descriptor,
            set_report_buf: None,
            protocol: determine_protocol_setting(&settings),
            settings,
        }
    }

    /// Same as new_ep_out() but includes a settings field.
    /// This should be uncommon (non-standard), but is included for completeness as there
    /// may be cases where setting the locale is useful.
    pub fn new_ep_out_with_settings<'a>(
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
            set_report_buf: None,
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
                        && self.settings.subclass != HidSubClass::NoSubClass)
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
                        && self.settings.subclass != HidSubClass::NoSubClass)
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

    /// Tries to read an incoming SET_REPORT report as raw bytes.
    /// Unlike OUT endpoints, report IDs are not prefixed in the buffer. Use the returned tuple
    /// instead to determine the buffer's usage.
    ///
    /// The most common usage of pull_raw_report is for keyboard lock LED status if an OUT endpoint
    /// is not defined. It is not necessary to call this function if you're not going to be using
    /// SET_REPORT functionality.
    pub fn pull_raw_report(&mut self, data: &mut [u8]) -> Result<ReportInfo> {
        let info = match &self.set_report_buf {
            Some(set_report_buf) => {
                let info = set_report_buf.info;

                // Make sure the given buffer is large enough for the stored report
                if data.len() < info.len {
                    return Err(UsbError::BufferOverflow);
                }

                // Copy buffer
                data[..info.len].copy_from_slice(&set_report_buf.buf[..info.len]);
                info
            }
            None => {
                return Err(UsbError::WouldBlock);
            }
        };

        // Clear the report
        self.set_report_buf = None;
        Ok(info)
    }

    /// Retrieves the currently set device protocol
    /// This is equivalent to the USB HID GET_PROTOCOL request
    /// See (7.2.5): <https://www.usb.org/sites/default/files/hid1_11.pdf>
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

    /// Forcibly sets the device protocol
    /// This is equivalent to the USB HID SET_PROTOCOL request.
    /// NOTE: If the OS does not support the new mode, the device may no longer work correctly.
    /// See (7.2.6): <https://www.usb.org/sites/default/files/hid1_11.pdf>
    ///
    /// If either, ForceBoot or ForceReport are set in config, the mode argument is ignored.
    /// In addition, if ForceBoot or ForceReport are set, then any SET_PROTOCOL requests are also
    /// ignored.
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
            writer.endpoint(ep)?;
        }
        if let Some(ep) = &self.in_ep {
            writer.endpoint(ep)?;
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
            (control::RequestType::Class, HID_REQ_GET_REPORT) => {
                // To support GET_REPORT correctly each request must be serviced immediately.
                // This complicates the current API and may require a standing copy of each
                // of the possible IN reports (as well as any FEATURE reports as well).
                // For most projects, GET_REPORT won't be necessary so until a project comes along
                // with a need for it, I think it's safe to leave unsupported.
                // See: https://www.usb.org/sites/default/files/documents/hid1_11.pdf 7.2.1
                xfer.reject().ok(); // Not supported for now
            }
            (control::RequestType::Class, HID_REQ_GET_IDLE) => {
                // XXX (HaaTa): As a note for future readers
                // GET/SET_IDLE tends to be rather buggy on the host side
                // macOS is known to set SET_IDLE for keyboards but most other OSs do not.
                // I haven't had much success in the past trying to enable GET/SET_IDLE for
                // macOS (it seems to expose other bugs in the macOS hid stack).
                // The interesting part is that SET_IDLE is not called for official Apple
                // keyboards. So beyond getting 100% compliance from the USB compliance tools
                // IDLE is useless (at least with respect to keyboards). Modern USB host
                // controllers should never have a problem keeping up with slow HID devices.
                //
                // To implement this correctly it would require integration with higher-level
                // functions to handle report expiry.
                // See https://www.usb.org/sites/default/files/documents/hid1_11.pdf 7.2.4
                //
                // Each Report ID can be configured independently.
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
                let report_type = ((req.value >> 8) as u8).into();
                let report_id = (req.value & 0xFF) as u8;
                let len = req.length as usize;

                // Validate that the incoming data isn't too large for the buffer
                if len > CONTROL_BUF_LEN {
                    self.set_report_buf = None;
                    xfer.reject().ok();
                } else {
                    let mut buf: [u8; CONTROL_BUF_LEN] = [0; CONTROL_BUF_LEN];
                    buf[..len].copy_from_slice(&xfer.data()[..len]);

                    // Overwrite previous buffer even if unused
                    self.set_report_buf = Some(Report {
                        info: ReportInfo {
                            report_type,
                            report_id,
                            len,
                        },
                        buf,
                    });
                    xfer.accept().ok();
                }
            }
            _ => {
                xfer.reject().ok();
            }
        }
    }
}
