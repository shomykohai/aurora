use bitflags::bitflags;

use crate::common::{Slot, Table};
use crate::traits::{FromBytes, ToBytes, bytes};

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct Mods: u8 {
        const LCTRL  = 1 << 0;
        const LSHIFT = 1 << 1;
        const LALT   = 1 << 2;
        /// Windows / Super / Whatever
        const LGUI   = 1 << 3;
        const RCTRL  = 1 << 4;
        const RSHIFT = 1 << 5;
        const RALT   = 1 << 6;
        const RGUI   = 1 << 7;
    }
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct SystemMods: u8 {
        const POWER = 1 << 0;
        const SLEEP = 1 << 1;
        const WAKE  = 1 << 2;
    }
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct MouseButtons: u8 {
        const LEFT    = 1 << 0;
        const RIGHT   = 1 << 1;
        const MIDDLE  = 1 << 2;
        const BUTTON4 = 1 << 3;
        const BUTTON5 = 1 << 4;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseAction {
    Buttons(MouseButtons),
    Wheel(i8),
}

#[bytes]
#[derive(PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum Key {
    A = 0x04,
    B = 0x05,
    C = 0x06,
    D = 0x07,
    E = 0x08,
    F = 0x09,
    G = 0x0A,
    H = 0x0B,
    I = 0x0C,
    J = 0x0D,
    K = 0x0E,
    L = 0x0F,
    M = 0x10,
    N = 0x11,
    O = 0x12,
    P = 0x13,
    Q = 0x14,
    R = 0x15,
    S = 0x16,
    T = 0x17,
    U = 0x18,
    V = 0x19,
    W = 0x1A,
    X = 0x1B,
    Y = 0x1C,
    Z = 0x1D,
    Digit1 = 0x1E,
    Digit2 = 0x1F,
    Digit3 = 0x20,
    Digit4 = 0x21,
    Digit5 = 0x22,
    Digit6 = 0x23,
    Digit7 = 0x24,
    Digit8 = 0x25,
    Digit9 = 0x26,
    Digit0 = 0x27,
    Enter = 0x28,
    Escape = 0x29,
    Backspace = 0x2A,
    Tab = 0x2B,
    Space = 0x2C,
    Minus = 0x2D,
    Equal = 0x2E,
    BracketLeft = 0x2F,
    BracketRight = 0x30,
    Backslash = 0x31,
    IntlHash = 0x32,
    Semicolon = 0x33,
    Quote = 0x34,
    Backquote = 0x35,
    Comma = 0x36,
    Period = 0x37,
    Slash = 0x38,
    CapsLock = 0x39,
    F1 = 0x3A,
    F2 = 0x3B,
    F3 = 0x3C,
    F4 = 0x3D,
    F5 = 0x3E,
    F6 = 0x3F,
    F7 = 0x40,
    F8 = 0x41,
    F9 = 0x42,
    F10 = 0x43,
    F11 = 0x44,
    F12 = 0x45,
    PrintScreen = 0x46,
    ScrollLock = 0x47,
    Pause = 0x48,
    Insert = 0x49,
    Home = 0x4A,
    PageUp = 0x4B,
    Delete = 0x4C,
    End = 0x4D,
    PageDown = 0x4E,
    ArrowRight = 0x4F,
    ArrowLeft = 0x50,
    ArrowDown = 0x51,
    ArrowUp = 0x52,
    NumLock = 0x53,
    NumpadDivide = 0x54,
    NumpadMultiply = 0x55,
    NumpadSubtract = 0x56,
    NumpadAdd = 0x57,
    NumpadEnter = 0x58,
    Numpad1 = 0x59,
    Numpad2 = 0x5A,
    Numpad3 = 0x5B,
    Numpad4 = 0x5C,
    Numpad5 = 0x5D,
    Numpad6 = 0x5E,
    Numpad7 = 0x5F,
    Numpad8 = 0x60,
    Numpad9 = 0x61,
    Numpad0 = 0x62,
    NumpadDecimal = 0x63,
    IntlBackslash = 0x64,
    ContextMenu = 0x65,
    F13 = 0x68,
    F14 = 0x69,
    F15 = 0x6A,
    F16 = 0x6B,
    F17 = 0x6C,
    F18 = 0x6D,
    F19 = 0x6E,
    F20 = 0x6F,
    F21 = 0x70,
    F22 = 0x71,
    F23 = 0x72,
    F24 = 0x73,
    IntlRo = 0x87,
    IntlYen = 0x89,
    ControlLeft = 0xE0,
    ShiftLeft = 0xE1,
    AltLeft = 0xE2,
    MetaLeft = 0xE3,
    ControlRight = 0xE4,
    ShiftRight = 0xE5,
    AltRight = 0xE6,
    MetaRight = 0xE7,
    Fn = 0xAF,
}

impl Key {
    pub const fn hid(self) -> u8 {
        self as u8
    }

    pub fn from_hid(hid: u8) -> Option<Self> {
        zerocopy::TryFromBytes::try_read_from_bytes(&[hid]).ok()
    }
}

/// Fn key, which the firmware treats differently :D
pub const HID_FN: u8 = Key::Fn.hid();

/// `[type][modifiers][keycode][param]`.
#[bytes]
#[derive(FromBytes, ToBytes, Default)]
#[repr(C, packed)]
pub struct KeySlot(pub [u8; 4]);

impl From<KeySlot> for KeyAction {
    fn from(slot: KeySlot) -> Self {
        let [ty, mods, code, param] = slot.0;
        match ty {
            0 => Self::Default,
            // Apparently this is a thing
            1 if mods == 1 => {
                Self::Mouse(MouseAction::Buttons(MouseButtons::from_bits_truncate(code)))
            }
            1 if mods == 3 => Self::Mouse(MouseAction::Wheel(code.cast_signed())),
            2 if code == HID_FN => Self::Fn,
            2 if code == 0 => Self::Modifier(Mods::from_bits_truncate(mods)),
            2 => Self::Key { mods: Mods::from_bits_truncate(mods), code },
            // Media keys are 16bit, the fw interprets them as [type][low][high][0]
            3 => Self::Consumer(u16::from_le_bytes([mods, code])),
            4 => Self::System(SystemMods::from_bits_truncate(mods)),
            5 => Self::Special { kind: mods, id: code },
            6 => Self::Macro { slot: mods, mode: code, repeat: param },
            7 => Self::MultiKey([mods, code, param]),
            _ => Self::Default,
        }
    }
}

impl From<KeyAction> for KeySlot {
    fn from(a: KeyAction) -> Self {
        Self(match a {
            KeyAction::Default => [0, 0, 0, 0],
            KeyAction::Mouse(MouseAction::Buttons(b)) => [1, 1, b.bits(), 0],
            KeyAction::Mouse(MouseAction::Wheel(d)) => [1, 3, d.cast_unsigned(), 0],
            KeyAction::Fn => [2, 0, HID_FN, 0],
            KeyAction::Modifier(m) => [2, m.bits(), 0, 0],
            KeyAction::Key { mods, code } => [2, mods.bits(), code, 0],
            KeyAction::Consumer(u) => {
                let [low, high] = u.to_le_bytes();
                [3, low, high, 0]
            }
            KeyAction::System(m) => [4, m.bits(), 0, 0],
            KeyAction::Special { kind, id } => [5, kind, id, 0],
            KeyAction::Macro { slot, mode, repeat } => [6, slot, mode, repeat],
            KeyAction::MultiKey(k) => [7, k[0], k[1], k[2]],
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyAction {
    Default,
    Key {
        mods: Mods,
        code: u8,
    },
    /// Mouse keys mapped onto the keyboard?
    Mouse(MouseAction),
    /// Modifier keys
    Modifier(Mods),
    Fn,
    /// Media keys
    Consumer(u16),
    System(SystemMods),
    /// Perform actions on the host?
    Special {
        kind: u8,
        id: u8,
    },
    Macro {
        slot: u8,
        mode: u8,
        repeat: u8,
    },
    /// Map multiple keys to a single key.
    MultiKey([u8; 3]),
}

/// A keymap of a single layer
pub type Keymap = Table<KeySlot>;

impl Slot for KeySlot {}
