use std::fmt::Display;

use zerocopy::little_endian::U16;

use crate::error::Error;
use crate::traits::{FromBytes, ToBytes, bytes};

/// Each of the structs that are stored in the EPPROM finishes with this magic,
/// and the firmware validates it to confirm it's not garbage data.
pub const MAGIC: [u8; 4] = [0x00, 0x00, 0xAA, 0x55];
const TABLE_BYTES: usize = 576;
const TABLE_SLOTS: usize = 143;

#[bytes]
#[repr(u8)]
pub enum Debounce {
    L1 = 1,
    L2 = 2,
    L3 = 3,
    L4 = 4,
    L5 = 5,
}

impl TryFrom<u8> for Debounce {
    type Error = Error;

    fn try_from(v: u8) -> Result<Self, Self::Error> {
        Ok(match v {
            1 => Self::L1,
            2 => Self::L2,
            3 => Self::L3,
            4 => Self::L4,
            5 => Self::L5,
            _ => return Err(Error::InvalidDebounce(v)),
        })
    }
}

#[bytes]
#[repr(u8)]
pub enum SleepTimeout {
    Never = 0,
    OneMinute = 1,
    FiveMinutes = 2,
    ThirtyMinutes = 3,
}

impl Display for SleepTimeout {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Never => "never",
            Self::OneMinute => "1 minute",
            Self::FiveMinutes => "5 minutes",
            Self::ThirtyMinutes => "30 minutes",
        })
    }
}

impl TryFrom<u8> for SleepTimeout {
    type Error = Error;

    fn try_from(v: u8) -> Result<Self, Self::Error> {
        Ok(match v {
            0 => Self::Never,
            1 => Self::OneMinute,
            2 => Self::FiveMinutes,
            3 => Self::ThirtyMinutes,
            _ => return Err(Error::InvalidSleepTimeout(v)),
        })
    }
}

/// A slot in a table.
pub trait Slot:
    Copy
    + Default
    + zerocopy::TryFromBytes
    + zerocopy::IntoBytes
    + zerocopy::Immutable
    + zerocopy::KnownLayout
    + zerocopy::Unaligned
{
    /// Whether this slot is valid at the given position in the table
    fn valid_at(&self, _position: usize) -> bool {
        true
    }
}

/// A table of elements in the EEPROM.
/// It's an array of 143 elements, for configuring stuff like keymaps or LEDs.
#[bytes]
#[repr(C)]
pub struct Table<S: Slot> {
    slots: [S; TABLE_SLOTS],
    magic: [u8; 4],
}

impl<S: Slot> FromBytes for Table<S> {
    const SIZE: usize = size_of::<Self>();

    fn from_bytes(raw: &[u8]) -> Option<Self> {
        let t = <Self as zerocopy::TryFromBytes>::try_read_from_bytes(raw).ok()?;
        (t.magic == MAGIC).then_some(())?;
        t.slots.iter().enumerate().all(|(i, s)| s.valid_at(i)).then_some(t)
    }
}

impl<S: Slot> ToBytes for Table<S> {
    type Output = [u8; TABLE_BYTES];

    const SIZE: usize = size_of::<Self>();

    fn to_bytes(&self) -> Self::Output {
        let mut buf = [0u8; TABLE_BYTES];
        buf.copy_from_slice(zerocopy::IntoBytes::as_bytes(self));
        buf[572..576].copy_from_slice(&MAGIC);
        buf
    }
}

impl<S: Slot> Default for Table<S> {
    fn default() -> Self {
        Self { slots: [S::default(); TABLE_SLOTS], magic: MAGIC }
    }
}

impl<S: Slot> Table<S> {
    pub const fn slots(&self) -> &[S; TABLE_SLOTS] {
        &self.slots
    }

    pub const fn slots_mut(&mut self) -> &mut [S; TABLE_SLOTS] {
        &mut self.slots
    }
}

#[bytes]
#[repr(C, packed)]
pub struct Settings {
    // Firmware never checks for this, but
    // original software sets it to 0x100
    _reserved0: [u8; 2],
    /// Whether ALT+Tab is disabled
    alt_tab_disabled: bool,
    /// Whether ALT+F4 is disabled
    alt_f4_disabled: bool,
    /// Whether the Windows key is disabled
    win_disabled: bool,
    /// Whether the Fn key is to be held or toggled
    fn_toggle: bool,
    /// How long the keyboard waits before sleeping
    sleep: SleepTimeout,
    _pad: u8,
    /// How long the keyboard waits before registering a keypress
    debounce: Debounce,
    _reserved1: [u8; 51],
    /// Must be 0x55AA0000 to make the firmware accept the payload
    magic: [u8; 4],
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            _reserved0: [0x00, 0x01],
            alt_tab_disabled: false,
            alt_f4_disabled: false,
            win_disabled: false,
            fn_toggle: false,
            sleep: SleepTimeout::FiveMinutes,
            _pad: 0,
            debounce: Debounce::L2,
            _reserved1: [0; 51],
            magic: MAGIC,
        }
    }
}

impl FromBytes for Settings {
    const SIZE: usize = size_of::<Self>();

    fn from_bytes(raw: &[u8]) -> Option<Self> {
        let s = <Self as zerocopy::TryFromBytes>::try_read_from_bytes(raw).ok()?;
        (s.magic == MAGIC).then_some(s)
    }
}

impl ToBytes for Settings {
    type Output = [u8; size_of::<Self>()];

    const SIZE: usize = size_of::<Self>();

    fn to_bytes(&self) -> Self::Output {
        let mut buf = [0u8; size_of::<Self>()];
        buf.copy_from_slice(zerocopy::IntoBytes::as_bytes(self));
        buf[60..64].copy_from_slice(&MAGIC);
        buf
    }
}

impl Settings {
    /// Whether ALT+Tab is disabled
    pub const fn alt_tab_disabled(&self) -> bool {
        self.alt_tab_disabled
    }

    /// Whether ALT+F4 is disabled
    pub const fn alt_f4_disabled(&self) -> bool {
        self.alt_f4_disabled
    }

    /// Whether the Windows key is disabled
    pub const fn win_disabled(&self) -> bool {
        self.win_disabled
    }

    /// Whether the Fn key is to be held or toggled
    pub const fn fn_toggle(&self) -> bool {
        self.fn_toggle
    }

    /// How long the keyboard waits before sleeping
    pub const fn sleep(&self) -> SleepTimeout {
        self.sleep
    }

    /// How long the keyboard waits before registering a keypress
    pub const fn debounce(&self) -> Debounce {
        self.debounce
    }

    /// Disable or enable ALT+Tab.
    pub const fn set_alt_tab_disabled(&mut self, v: bool) {
        self.alt_tab_disabled = v
    }

    /// Disable or enable ALT+F4.
    pub const fn set_alt_f4_disabled(&mut self, v: bool) {
        self.alt_f4_disabled = v
    }

    /// Disable or enable the Windows key.
    pub const fn set_win_disabled(&mut self, v: bool) {
        self.win_disabled = v
    }

    /// Set whether the Fn key is to be held or toggled.
    pub const fn set_fn_toggle(&mut self, v: bool) {
        self.fn_toggle = v
    }

    /// Set how long the keyboard waits before sleeping.
    pub const fn set_sleep(&mut self, v: SleepTimeout) {
        self.sleep = v
    }

    /// Set how long the keyboard waits before registering a keypress.
    pub const fn set_debounce(&mut self, v: Debounce) {
        self.debounce = v
    }
}

#[bytes]
#[derive(FromBytes, ToBytes)]
#[repr(C, packed)]
pub struct DeviceInfo {
    pub payload_size: U16,
    pub max_blocks: U16,
    pub vid: U16,
    pub pid: U16,
    pub bcd_device: U16,
}
