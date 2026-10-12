use std::thread::sleep;
use std::time::Duration;

use aurora_protocol::common::Settings;
use aurora_protocol::keys::Keymap;
use aurora_protocol::leds::LightingConfig;
use aurora_protocol::traits::{FromBytes, ToBytes, bytes};
use aurora_protocol::{Error, Quirks, Region, RegionRequest, Result, TransportKind};

use crate::hid::Handle;

pub const RECORD_LEN: usize = 28;

#[bytes]
#[repr(u8)]
pub enum Opcode {
    Probe = 0x02,
    KeyBase = 0x03,
    Lighting = 0x05,
    Settings = 0x07,
    Macros = 0x09,
    KeyFn = 0x0D,
    KeymapBase = 0x10,
    KeymapFn = 0x12,
    #[allow(dead_code)]
    UserLight = 0x14,
    Battery = 0x20,
}

/// How the firmware places a frame's data bytes.
#[derive(Debug, Clone, Copy)]
enum Layout {
    Whole(usize),
    Chunked,
}

impl Opcode {
    const fn length(self) -> u8 {
        match self {
            Self::Probe => 0x00,
            Self::Battery => 0x01,
            Self::KeyBase | Self::KeyFn => 0x04,
            Self::Lighting | Self::Settings | Self::KeymapFn => 0x10,
            Self::Macros | Self::KeymapBase | Self::UserLight => 0x1C,
        }
    }

    const fn layout(self) -> Layout {
        match self {
            Self::Settings => Layout::Whole(28),
            Self::Lighting => Layout::Whole(16),
            _ => Layout::Chunked,
        }
    }

    // The regions that erase a whole flash page before programming, so a partial write
    // destroys whatever it did not replace.
    const fn erases(self) -> bool {
        matches!(self, Self::Macros | Self::KeymapBase | Self::KeymapFn | Self::UserLight)
    }

    const fn region_len(self) -> usize {
        match self {
            Self::Settings => size_of::<Settings>(),
            Self::Lighting => size_of::<LightingConfig>(),
            Self::KeymapBase | Self::KeymapFn | Self::UserLight => size_of::<Keymap>(),
            Self::Macros => 3072,
            _ => 0,
        }
    }

    // Which opcodes allow to write a single slot (like a single Key or LED cfg) instead of the
    // whole region.
    const fn single_slot(self) -> Option<Self> {
        match self {
            Self::KeymapBase => Some(Self::KeyBase),
            Self::KeymapFn => Some(Self::KeyFn),
            _ => None,
        }
    }
}

impl TryFrom<Region> for Opcode {
    type Error = Error;

    fn try_from(region: Region) -> Result<Self> {
        Ok(match region {
            Region::Settings => Self::Settings,
            Region::Lighting => Self::Lighting,
            Region::Keymap(0) => Self::KeymapBase,
            Region::Keymap(1) => Self::KeymapFn,
            Region::Macros => Self::Macros,
            _ => return Err(Error::UnsupportedOnTransport(TransportKind::Dongle)),
        })
    }
}

#[bytes]
#[derive(ToBytes)]
#[repr(C, packed)]
pub struct Frame {
    opcode: Opcode,
    length: u8,
    index: u8,
    record: [u8; RECORD_LEN],
    sum: u8,
}

impl Frame {
    pub const SIZE: usize = size_of::<Self>();

    pub fn new(opcode: Opcode, index: u8, record: &[u8]) -> Self {
        let mut frame =
            Self { opcode, length: opcode.length(), index, record: [0; RECORD_LEN], sum: 0 };
        let len = record.len().min(RECORD_LEN);
        frame.record[..len].copy_from_slice(&record[..len]);
        frame.sum = frame.checksum();
        frame
    }

    fn checksum(&self) -> u8 {
        zerocopy::IntoBytes::as_bytes(self)[..Self::SIZE - 1]
            .iter()
            .fold(0u8, |sum, &byte| sum.wrapping_add(byte))
    }

    pub fn is_valid(&self) -> bool {
        self.sum == self.checksum()
    }

    pub const fn record(&self) -> &[u8] {
        &self.record
    }
}

impl FromBytes for Frame {
    const SIZE: usize = Self::SIZE;

    fn from_bytes(raw: &[u8]) -> Option<Self> {
        let frame = <Self as zerocopy::TryFromBytes>::try_read_from_bytes(raw).ok()?;
        frame.is_valid().then_some(frame)
    }
}

#[derive(Debug)]
pub struct Dongle {
    hid: Handle,
    quirks: Quirks,
}

impl Dongle {
    const ATTEMPTS: usize = 20;
    const TIMEOUT: Duration = Duration::from_millis(50);

    pub(crate) const fn new(hid: Handle, quirks: Quirks) -> Self {
        Self { hid, quirks }
    }

    fn send(&mut self, frame: &Frame) -> Result<()> {
        sleep(Duration::from_millis(u64::from(self.quirks.inter_transfer_delay_ms)));
        self.hid.write(&frame.to_bytes())
    }

    fn recv(&mut self) -> Result<Frame> {
        let mut raw = [0u8; Frame::SIZE];

        if self.hid.read(&mut raw, Self::TIMEOUT)? != Frame::SIZE {
            return Err(Error::Desync);
        }

        Frame::from_bytes(&raw).ok_or(Error::Desync)
    }

    fn request(&mut self, frame: &Frame) -> Result<Frame> {
        for _ in 0..Self::ATTEMPTS {
            self.send(frame)?;

            if let Ok(reply) = self.recv()
                && reply.opcode == frame.opcode
                && reply.length == frame.length
            {
                return Ok(reply);
            }
        }

        Err(Error::Desync)
    }

    pub(crate) fn device_info(&mut self) -> Result<Vec<u8>> {
        let reply = self.request(&Frame::new(Opcode::Probe, 0, &[]))?;
        Ok(reply.record().to_vec())
    }

    // TODO: Use
    #[allow(dead_code)]
    pub(crate) fn battery(&mut self) -> Result<u8> {
        let reply = self.request(&Frame::new(Opcode::Battery, 0, &[]))?;
        reply.record().first().copied().ok_or(Error::Desync)
    }

    fn write_region(&mut self, region: Region, payload: &[u8]) -> Result<()> {
        let opcode = Opcode::try_from(region)?;

        // Probe the keyboard just to make sure it can receive frames.
        self.request(&Frame::new(Opcode::Probe, 0, &[]))?;

        let limit = opcode.region_len();
        if payload.len() > limit || (opcode.erases() && payload.len() != limit) {
            return Err(Error::RegionOverflow {
                blocks: payload.len().div_ceil(RECORD_LEN) as u8,
                limit: limit.div_ceil(RECORD_LEN) as u8,
            });
        }

        match opcode.layout() {
            Layout::Whole(bytes) => {
                self.send(&Frame::new(opcode, 0, payload.get(..bytes).unwrap_or(payload)))
            }
            Layout::Chunked => {
                for (index, record) in payload.chunks(RECORD_LEN).enumerate() {
                    self.send(&Frame::new(opcode, index as u8, record))?;
                }
                Ok(())
            }
        }
    }

    pub(crate) fn supports(&self, req: &RegionRequest) -> bool {
        match req {
            // Nice protocol you got here, can I have it?
            RegionRequest::Read(region) => *region == Region::DeviceInfo,
            RegionRequest::Write { region, .. } => Opcode::try_from(*region).is_ok(),
            RegionRequest::WriteSlot { region, .. } => {
                Opcode::try_from(*region).is_ok_and(|op| op.single_slot().is_some())
            }
            RegionRequest::Ping => true,
            // Dongle transport will handle this automatically
            RegionRequest::Reload => false,
        }
    }

    pub(crate) fn execute(&mut self, req: RegionRequest) -> Result<Vec<u8>> {
        match req {
            RegionRequest::Read(Region::DeviceInfo) => self.device_info(),
            RegionRequest::Read(_) | RegionRequest::Reload => {
                Err(Error::UnsupportedOnTransport(TransportKind::Dongle))
            }
            RegionRequest::Write { region, data } => {
                self.write_region(region, &data)?;
                Ok(Vec::new())
            }
            RegionRequest::WriteSlot { region, index, slot } => {
                let opcode = Opcode::try_from(region)?
                    .single_slot()
                    .ok_or(Error::UnsupportedOnTransport(TransportKind::Dongle))?;

                self.send(&Frame::new(opcode, index, &slot))?;
                Ok(Vec::new())
            }
            RegionRequest::Ping => {
                self.request(&Frame::new(Opcode::Probe, 0, &[]))?;
                Ok(Vec::new())
            }
        }
    }
}
