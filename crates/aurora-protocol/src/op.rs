use std::fmt::Display;
use std::str::FromStr;

use crate::error::{Error, Result};

/// An area of device storage for various configurations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Region {
    /// Read only info about the device. Requires
    /// [`CapId::DeviceInfo`](crate::cap::CapId::DeviceInfo).
    DeviceInfo,
    /// Global settings about the board. Requires [`CapId::Settings`](crate::cap::CapId::Settings).
    Settings,
    /// LEDs mode, colour and more. Requires [`CapId::Lighting`](crate::cap::CapId::Lighting).
    Lighting,
    /// Per key colours. Requires [`CapId::KeyColors`](crate::cap::CapId::KeyColors).
    KeyColors,
    /// Index into the board's `LAYERS`.
    Keymap(u8),
    /// Macros. Requires [`CapId::Macros`](crate::cap::CapId::Macros).
    Macros,
}

/// Weird abstraction because I can't code
/// Basically: abstract a region request because the transport layer might encode the payload
/// differently depending on the connection type (why have one sane protocol when you cave
/// have 3!!).
#[derive(Debug, Clone)]
pub enum RegionRequest {
    /// Read the whole region back.
    Read(Region),

    /// Replace the whole region.
    Write { region: Region, data: Vec<u8> },

    /// Replace just a u32 (or 4 bytes if we prefer :D) at the given slot.
    WriteSlot { region: Region, index: u8, slot: [u8; 4] },

    /// Make the device apply the new settings.
    Reload,

    /// Acknowledge the device doesn't hate us.
    Ping,
}

impl Display for Region {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DeviceInfo => f.write_str("device-info"),
            Self::Settings => f.write_str("settings"),
            Self::Lighting => f.write_str("lighting"),
            Self::KeyColors => f.write_str("key-colors"),
            Self::Keymap(layer) => write!(f, "keymap-{layer}"),
            Self::Macros => f.write_str("macros"),
        }
    }
}

impl FromStr for Region {
    type Err = Error;

    fn from_str(name: &str) -> Result<Self> {
        Ok(match name {
            "device-info" => Self::DeviceInfo,
            "settings" => Self::Settings,
            "lighting" => Self::Lighting,
            "key-colors" => Self::KeyColors,
            "macros" => Self::Macros,
            _ => name
                .strip_prefix("keymap-")
                .and_then(|layer| layer.parse().ok())
                .map(Self::Keymap)
                .ok_or_else(|| Error::UnknownRegion(name.to_owned()))?,
        })
    }
}
