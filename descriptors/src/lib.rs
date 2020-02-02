use bitfield::bitfield;

/// GlobalItemKind describes global item tags as described in section 6.2.2.7
/// 'Report Descriptor' of the spec, version 1.11.
#[repr(u8)]
#[allow(unused)]
#[derive(Copy, Debug, Clone, Eq, PartialEq)]
pub enum GlobalItemKind {
    UsagePage = 0,
    LogicalMin = 1,
    LogicalMax = 2,
    PhysicalMin = 3,
    PhysicalMax = 4,
    UnitExponent = 5,
    Unit = 6,
    ReportSize = 7,
    ReportID = 8,
    ReportCount = 9,
}

impl Into<u8> for GlobalItemKind {
    fn into(self) -> u8 {
        unsafe { core::mem::transmute(self) }
    }
}

/// LocalItemKind describes local item tags as described in section 6.2.2.8
/// 'Local Items' of the spec, version 1.11.
#[repr(u8)]
#[allow(unused)]
#[derive(Copy, Debug, Clone, Eq, PartialEq)]
pub enum LocalItemKind {
    Usage = 0,
    UsageMin = 1,
    UsageMax = 2,
    DesignatorIdx = 3,
    DesignatorMin = 4,
    DesignatorMax = 5,
    StringIdx = 7,
    StringMin = 8,
    StringMax = 9,
    Delimiter = 10,
}

impl Into<u8> for LocalItemKind {
    fn into(self) -> u8 {
        unsafe { core::mem::transmute(self) }
    }
}

/// MainItemKind describes main item tags as described in section 6.2.2.4
/// 'Report Descriptor' of the spec, version 1.11.
#[repr(u8)]
#[allow(unused)]
#[derive(Copy, Debug, Clone, Eq, PartialEq)]
pub enum MainItemKind {
    Input = 0b1000,
    Output = 0b1001,
    Feature = 0b1011,
    Collection = 0b1010,
    EndCollection = 0b1100,
}

impl Into<u8> for MainItemKind {
    fn into(self) -> u8 {
        unsafe { core::mem::transmute(self) }
    }
}

impl Default for MainItemKind {
    fn default() -> Self { MainItemKind::Input }
}

impl Into<MainItemKind> for String {
    fn into(self) -> MainItemKind {
        match self.as_str() {
            "feature" => MainItemKind::Feature,
            "output" => MainItemKind::Output,
            "collection" => MainItemKind::Collection,
            "ecollection" => MainItemKind::EndCollection,
            "input" => MainItemKind::Input,
            _ => MainItemKind::Input,
        }
    }
}


/// ItemType describes types of items as described in section 6.2.2.7
/// 'Report Descriptor' of the spec, version 1.11.
#[repr(u8)]
#[allow(unused)]
#[derive(Copy, Debug, Clone, Eq, PartialEq)]
pub enum ItemType {
    Main = 0,
    Global = 1,
    Local = 2,
}

impl Into<u8> for ItemType {
    fn into(self) -> u8 {
        unsafe { core::mem::transmute(self) }
    }
}

impl Default for ItemType {
    fn default() -> Self { ItemType::Main }
}


bitfield! {
    /// MainItemSetting describes the bits which configure invariants on a MainItem.
    #[derive(Clone,Debug)]
    pub struct MainItemSetting(u8);
    pub is_constant, set_constant: 0;
    pub is_variable, set_variable: 1;
    pub is_relative, set_relative: 2;
    pub is_wrap, set_wrap: 3;
    pub is_non_linear, set_non_linear: 4;
    pub has_no_preferred_state, set_no_preferred_state: 5;
    pub has_null_state, set_has_null_state: 6;
    pub volatile, set_volatile: 7;
}

bitfield! {
    /// ItemPrefix describes the 1 byte prefix describing an item in a descriptor.
    pub struct ItemPrefix(u8);
    impl Debug;
    pub byte_count, set_byte_count: 1, 0;
    pub typ, set_type: 3, 2;
    pub tag, set_tag: 7, 4;
}
