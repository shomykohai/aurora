use crate::cap::{Cap, CapId};
use crate::keys::Key;
use crate::op::Region;

/// How a board is recognised
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UsbId {
    pub vid: u16,
    pub pid: u16,
    /// `bcdDevice` range, or `None` to accept any firmware revision.
    pub bcd: Option<(u16, u16)>,
    pub transport: TransportKind,
    /// Which HID usage page we can use to modify the board's state.
    pub usage_page: u16,
    /// Which interface, when usage page is present on multiple interfaces.
    /// `None` to use the first one found.
    pub interface: Option<i32>,
}

/// For some reason, the firmware transfer protocol changes depending on the
/// connection type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransportKind {
    /// The keyboard is connected through a USB cable
    Wired,
    /// The keyboard is connected through a wireless 2.4GHz dongle
    Dongle,
}

/// A key on the board
#[derive(Debug, Clone, Copy)]
pub struct KeyDesc {
    pub key: Key,
    /// Slot in the keymap tables
    pub key_index: u8,
    /// Slot in the per-key colour table. Not generally equal to `key_index`, but mostly is.
    pub light_index: u8,
    pub row: u8,
    pub col: u8,
}

/// Firmware specific behaviours
#[derive(Debug, Clone, Copy)]
pub struct Quirks {
    /// Avoid OOB writes :D
    pub macro_block_cap: u16,
    /// The value an unused macro slot must hold to be considered empty.
    pub macro_empty_marker: u32,
    /// Milliseconds to wait before every transfer.
    pub inter_transfer_delay_ms: u32,
    /// Milliseconds to wait after a flash write and before the device will accept another one.
    pub flash_program_delay_ms: u32,
}

/// A keyboard.
#[derive(Debug)]
pub struct Board {
    /// Unique ID for the board
    pub id: &'static str,
    /// The board's name.
    pub name: &'static str,
    /// How the board will be recognised over USB, in the order it should be tried.
    /// Listing the cable before the dongle prefers the link that can read.
    pub usb: &'static [UsbId],
    /// Key matrix.
    pub keys: &'static [KeyDesc],
    /// Layer names, ordered by their index.
    pub layers: &'static [&'static str],
    /// Capabilities the board supports.
    pub caps: &'static [Cap],
    /// Firmware specific behaviours for this board.
    pub quirks: Quirks,
}

impl Board {
    /// The board's entry for a key, if it has that key at all.
    pub fn key(&self, key: Key) -> Option<&'static KeyDesc> {
        self.keys.iter().find(|k| k.key == key)
    }

    pub fn supports(&self, id: CapId) -> bool {
        self.caps.iter().any(|cap| cap.id == id)
    }

    pub fn regions(&'static self) -> impl Iterator<Item = Region> {
        let layers = u8::try_from(self.layers.len()).unwrap_or(u8::MAX);

        [Region::DeviceInfo, Region::Macros]
            .into_iter()
            .chain((0..layers).map(Region::Keymap))
            .chain([Region::KeyColors, Region::Lighting, Region::Settings])
            .filter(|region| self.supports(CapId::from(*region)))
    }

    /// The index of the layer with the given name.
    pub fn layer(&self, name: &str) -> Option<u8> {
        self.layers
            .iter()
            .position(|l| l.eq_ignore_ascii_case(name))
            .and_then(|i| u8::try_from(i).ok())
    }
}
