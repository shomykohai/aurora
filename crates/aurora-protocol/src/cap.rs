use std::fmt::Display;

use crate::leds::LightMode;
use crate::op::Region;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CapId {
    DeviceInfo,
    Settings,
    Lighting,
    KeyColors,
    Keymap,
    Macros,
    Screen,
    Battery,
}

impl From<Region> for CapId {
    fn from(region: Region) -> Self {
        match region {
            Region::DeviceInfo => Self::DeviceInfo,
            Region::Settings => Self::Settings,
            Region::Lighting => Self::Lighting,
            Region::KeyColors => Self::KeyColors,
            Region::Keymap(_) => Self::Keymap,
            Region::Macros => Self::Macros,
        }
    }
}

/// A capability a board supports.
#[derive(Debug, Clone, Copy)]
pub struct Cap {
    pub id: CapId,
    /// Capabilities a board must support for this one to work.
    pub depends_on: &'static [CapId],
    /// Parameters for the capability.
    pub params: Params,
}

impl Display for Cap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.id)
    }
}

/// Parameters for a capability.
#[derive(Debug, Clone, Copy)]
pub enum Params {
    None,
    Rgb(RgbParams),
    Macros(MacroParams),
}

#[derive(Debug, Clone, Copy)]
pub struct RgbParams {
    /// Highest LED index the board can change without it being no-op.
    pub index_max: u8,
    /// Which mode the board needs to be in for the custom LED colours to be applied.
    pub needs_mode: Option<LightMode>,
}

#[derive(Debug, Clone, Copy)]
pub struct MacroParams {
    /// How many macros the board can store.
    pub slots: usize,
    /// How many blocks before we do an OOB write in the next region,
    /// because we like security!!
    pub max_blocks: u16,
}
