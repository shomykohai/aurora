use rgb::RGB8;

use crate::common::{MAGIC, Slot, Table};
use crate::error::Error;
use crate::traits::{FromBytes, ToBytes, bytes};

/// Shared by brightness and speed
#[bytes]
#[repr(u8)]
pub enum Level {
    Off = 0,
    L1 = 1,
    L2 = 2,
    L3 = 3,
    L4 = 4,
    L5 = 5,
}

impl TryFrom<u8> for Level {
    type Error = Error;

    fn try_from(v: u8) -> Result<Self, Self::Error> {
        Ok(match v {
            0 => Self::Off,
            1 => Self::L1,
            2 => Self::L2,
            3 => Self::L3,
            4 => Self::L4,
            5 => Self::L5,
            _ => return Err(Error::InvalidLightLevel(v)),
        })
    }
}

#[bytes]
#[repr(u8)]
pub enum Direction {
    Left = 0,
    Right = 1,
    Top = 2,
    Bottom = 3,
}

impl TryFrom<u8> for Direction {
    type Error = Error;

    fn try_from(v: u8) -> Result<Self, Self::Error> {
        Ok(match v {
            0 => Self::Left,
            1 => Self::Right,
            2 => Self::Top,
            3 => Self::Bottom,
            _ => return Err(Error::InvalidLightDirection(v)),
        })
    }
}

/// Cute LED mode names from the firmware language files
#[bytes]
#[repr(u8)]
#[derive(Default)]
pub enum LightMode {
    /// All LEDs off
    Off = 0x00,
    #[default]
    /// All LEDs on, no effect, with the configured colour
    Static = 0x01,
    /// All LEDs off by default, but the configured colour is used on keypresses
    SingleOn = 0x02,
    /// All LEDs on by default with the configured colour, but turn off the LED of the pressed key
    SingleOff = 0x03,
    /// LEDs turn on randomly at full brightness with the configured colour, then fade out their
    /// brightness
    Glittering = 0x04,
    /// LEDs fall down like rain with the configured colour, then fade out their brightness
    Falling = 0x05,
    /// All LEDs on, with random colours, changing over time, ignoring the configured colour
    Colourful = 0x06,
    /// All LEDs fade in and out with the configured colour
    Breath = 0x07,
    /// All LEDs change through the colour spectrum, ignoring the configured colour
    Spectrum = 0x08,
    /// All LEDs on, with the configured colour, but the brightness of the LEDs changes from the
    /// center outwards
    Outward = 0x09,
    /// A row or column of LEDs moves across the keyboard with the configured colour
    Scrolling = 0x0A,
    /// Like scrolling but more "wavy", with the configured colour
    Rolling = 0x0B,
    /// SPIN! (looks like a fan) With the configured colour
    Rotating = 0x0C,
    /// All LEDs off. On keypress, the pressed key's LED turns on with the configured colour
    /// propagating outwards on the column or row.
    Explode = 0x0D,
    /// Like `Explode`, but it propagates on more than one row and column, with the configured
    /// colour
    Launch = 0x0E,
    /// Like `Explode`, but it propagates in a circular pattern, with the configured colour
    Ripples = 0x0F,
    /// All LEDs off, with the configured colour propagating across the keyboard starting from the
    /// top left corner, lighting up each key of the row and then moving to the next row, until
    /// the last key is lit up, then it starts over again
    Flowing = 0x10,
    /// All LEDs on doing a sort of traingle wave pattern with the configured colour, looks like
    /// a heartbeat?
    Pulsating = 0x11,
    /// All LEDs with the configured colour, doing a sort of repeated `\` pattern
    Tilt = 0x12,
    /// All LEDs with the configured colour, with lights coming from both sides metting in the
    /// center
    Shuttle = 0x13,

    // On M75, these are handled by the firmware
    // but the side LEDs do not light up :(
    SideFlowing = 0x20,
    SideRed = 0x21,
    SideYellow = 0x22,
    SideGreen = 0x23,
    SideIceBlue = 0x24,
    SideBlue = 0x25,
    SidePink = 0x26,
    SideWhite = 0x27,
    SideNeon = 0x28,
    SideOff = 0x29,

    UserLayer = 0x80,
}

impl TryFrom<u8> for LightMode {
    type Error = Error;

    fn try_from(v: u8) -> Result<Self, Self::Error> {
        zerocopy::TryFromBytes::try_read_from_bytes(&[v]).map_err(|_| Error::InvalidLightMode(v))
    }
}

impl TryFrom<String> for LightMode {
    type Error = Error;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        Ok(match s.to_ascii_lowercase().as_str() {
            "off" => Self::Off,
            "static" => Self::Static,
            "single_on" => Self::SingleOn,
            "single_off" => Self::SingleOff,
            "glittering" => Self::Glittering,
            "falling" => Self::Falling,
            "colourful" => Self::Colourful,
            "breath" => Self::Breath,
            "spectrum" => Self::Spectrum,
            "outward" => Self::Outward,
            "scrolling" => Self::Scrolling,
            "rolling" => Self::Rolling,
            "rotating" => Self::Rotating,
            "explode" => Self::Explode,
            "launch" => Self::Launch,
            "ripples" => Self::Ripples,
            "flowing" => Self::Flowing,
            "pulsating" => Self::Pulsating,
            "tilt" => Self::Tilt,
            "shuttle" => Self::Shuttle,

            // Side modes
            "side_flowing" => Self::SideFlowing,
            "side_red" => Self::SideRed,
            "side_yellow" => Self::SideYellow,
            "side_green" => Self::SideGreen,
            "side_ice_blue" => Self::SideIceBlue,
            "side_blue" => Self::SideBlue,
            "side_pink" => Self::SidePink,
            "side_white" => Self::SideWhite,
            "side_neon" => Self::SideNeon,
            "side_off" => Self::SideOff,

            _ => return Err(Error::InvalidLightMode(0xFF)),
        })
    }
}

impl TryFrom<&str> for LightMode {
    type Error = Error;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        Self::try_from(s.to_string())
    }
}

impl LightMode {
    pub const fn is_keyboard(&self) -> bool {
        (*self as u8) <= 0x13
    }

    pub const fn is_side(&self) -> bool {
        matches!(*self as u8, 0x20..=0x29)
    }
}

/// LED config with predefined modes and colours.
/// This contrasts with `LedTable`, which sets a colour for each LED individually.
#[bytes]
#[repr(C, packed)]
pub struct LightingConfig {
    /// The light mode
    mode: LightMode,
    /// Red component of the colour
    r: u8,
    /// Green component of the colour
    g: u8,
    /// Blue component of the colour
    b: u8,
    _reserved: [u8; 4],
    /// Whether the effect is colourful (ignores the configured colour) or not
    colorful: bool,
    /// The brightness of the LEDs
    brightness: Level,
    /// The speed of the effect, if applicable
    speed: Level,
    /// The direction of the effect, if applicable
    direction: Direction,
    magic: [u8; 4],
}

impl FromBytes for LightingConfig {
    const SIZE: usize = size_of::<Self>();

    fn from_bytes(raw: &[u8]) -> Option<Self> {
        let c = <Self as zerocopy::TryFromBytes>::try_read_from_bytes(raw).ok()?;
        (c.magic == MAGIC).then_some(c)
    }
}

impl ToBytes for LightingConfig {
    type Output = [u8; size_of::<Self>()];

    const SIZE: usize = size_of::<Self>();

    fn to_bytes(&self) -> Self::Output {
        let mut buf = [0u8; size_of::<Self>()];
        buf.copy_from_slice(zerocopy::IntoBytes::as_bytes(self));
        buf[12..16].copy_from_slice(&MAGIC);
        buf
    }
}

impl LightingConfig {
    pub const fn mode(&self) -> LightMode {
        self.mode
    }

    pub const fn color(&self) -> RGB8 {
        RGB8::new(self.r, self.g, self.b)
    }

    pub const fn colorful(&self) -> bool {
        self.colorful
    }

    pub const fn brightness(&self) -> Level {
        self.brightness
    }

    pub const fn speed(&self) -> Level {
        self.speed
    }

    pub const fn direction(&self) -> Direction {
        self.direction
    }

    pub const fn set_mode(&mut self, v: LightMode) {
        self.mode = v
    }

    pub const fn set_color(&mut self, c: RGB8) {
        self.r = c.r;
        self.g = c.g;
        self.b = c.b
    }

    pub const fn set_colorful(&mut self, v: bool) {
        self.colorful = v
    }

    pub const fn set_brightness(&mut self, v: Level) {
        self.brightness = v
    }

    pub const fn set_speed(&mut self, v: Level) {
        self.speed = v
    }

    pub const fn set_direction(&mut self, v: Direction) {
        self.direction = v
    }
}

#[bytes]
#[derive(FromBytes, ToBytes, Default)]
#[repr(C, packed)]
pub struct LedSlot {
    pub index: u8,
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl LedSlot {
    pub const UNUSED: Self = Self { index: 0, r: 0, g: 0, b: 0 };

    pub fn is_blank(&self) -> bool {
        (self.index, self.r, self.g, self.b) == (0, 0, 0, 0)
    }

    pub fn is_dark(&self) -> bool {
        (self.r, self.g, self.b) == (0, 0, 0)
    }

    pub const fn color(&self) -> RGB8 {
        RGB8::new(self.r, self.g, self.b)
    }

    pub const fn set_color(&mut self, c: RGB8) {
        self.r = c.r;
        self.g = c.g;
        self.b = c.b
    }
}

/// A table of LEDs, each with its own colour.
/// This contrasts with `LightingConfig`, which sets a single colour for all LEDs.
pub type LedTable = Table<LedSlot>;

impl Slot for LedSlot {
    fn valid_at(&self, position: usize) -> bool {
        // The firmware rejects any slot whose index does not
        // match the position in the table if populated.
        self.is_blank() || usize::from(self.index) == position
    }
}
