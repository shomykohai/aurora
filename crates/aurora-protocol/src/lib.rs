// Needed for the proc macro :D
extern crate self as aurora_protocol;

pub mod board;
pub mod cap;
pub mod common;
pub mod error;
pub mod keys;
pub mod leds;
pub mod op;
pub mod traits;

pub use board::{Board, KeyDesc, Quirks, TransportKind, UsbId};
pub use cap::{Cap, CapId};
pub use error::{Error, Result};
pub use op::{Region, RegionRequest};
pub use traits::{FromBytes, ToBytes};
