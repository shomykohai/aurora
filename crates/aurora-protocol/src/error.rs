use thiserror::Error;

use crate::TransportKind;
use crate::cap::CapId;
use crate::keys::Key;
use crate::op::Region;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("no supported keyboard attached")]
    NotFound,

    #[error("device rejected the command")]
    Nak,

    #[error("device out of sync (stale transfer?)")]
    Desync,

    #[error("could not decode the {0} region")]
    InvalidRegion(Region),

    #[error("these bytes are not a valid {0} record")]
    InvalidRecord(Region),

    #[error("the {0} region is read only")]
    ReadOnly(Region),

    #[error("the {0} region did not complete the write")]
    WriteLost(Region),

    // TODO: Remove this eventually
    #[error("restoring the {0} region is not verified on this hardware yet")]
    Unverified(Region),

    #[error("unknown region {0}")]
    UnknownRegion(String),

    #[error("checksum mismatch: got {reported:#010x}, expected {expected:#010x}")]
    Checksum { reported: u32, expected: u32 },

    #[error("invalid debounce level {0}")]
    InvalidDebounce(u8),

    #[error("invalid sleep timeout {0}")]
    InvalidSleepTimeout(u8),

    #[error("invalid light level {0}")]
    InvalidLightLevel(u8),

    #[error("invalid light direction {0}")]
    InvalidLightDirection(u8),

    #[error("invalid light mode {0}")]
    InvalidLightMode(u8),

    #[error("capability {0:?} not supported by this board")]
    UnsupportedCap(CapId),

    #[error("operation not available on {0:?} transport")]
    UnsupportedOnTransport(TransportKind),

    #[error("this board has no {0:?} key")]
    UnknownKey(Key),

    #[error("this board puts {key:?} at slot {index}, which is past the end of the table")]
    SlotOutOfRange { key: Key, index: u8 },

    #[error("this board has no keymap layer {0}")]
    UnknownLayer(u8),

    #[error("payload is {blocks} blocks, {limit} is the maximum allowed for this region")]
    RegionOverflow { blocks: u8, limit: u8 },

    #[error("transport: {0}")]
    Io(#[from] std::io::Error),
}
