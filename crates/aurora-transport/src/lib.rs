mod dongle;
mod hid;
mod wired;

use std::io::Error as IoError;

use aurora_protocol::{Board, Error, RegionRequest, Result, TransportKind};
pub use dongle::Dongle;
use hidapi::HidApi;
pub use wired::Wired;

#[derive(Debug)]
pub enum Connection {
    Wired(Wired),
    Dongle(Dongle),
}

impl Connection {
    pub fn connect(
        boards: impl IntoIterator<Item = &'static Board>,
    ) -> Result<(Self, &'static Board)> {
        let api = HidApi::new().map_err(IoError::other)?;

        for spec in boards {
            for id in spec.usb {
                let Some(node) = hid::select(&api, id) else { continue };
                let handle = hid::Handle::open(&api, node)?;

                let connection = match id.transport {
                    TransportKind::Wired => Self::Wired(Wired::new(handle, spec.quirks)),
                    TransportKind::Dongle => Self::Dongle(Dongle::new(handle, spec.quirks)),
                };

                return Ok((connection, spec));
            }
        }

        Err(Error::NotFound)
    }

    pub const fn kind(&self) -> TransportKind {
        match self {
            Self::Wired(_) => TransportKind::Wired,
            Self::Dongle(_) => TransportKind::Dongle,
        }
    }

    pub fn supports(&self, req: &RegionRequest) -> bool {
        match self {
            Self::Wired(wired) => wired.supports(req),
            Self::Dongle(dongle) => dongle.supports(req),
        }
    }

    pub fn execute(&mut self, req: RegionRequest) -> Result<Vec<u8>> {
        match self {
            Self::Wired(wired) => wired.execute(req),
            Self::Dongle(dongle) => dongle.execute(req),
        }
    }
}
