use std::thread::sleep;
use std::time::{Duration, Instant};

use aurora_protocol::traits::{FromBytes, ToBytes, bytes};
use aurora_protocol::{Error, Quirks, Region, RegionRequest, Result, TransportKind};
use zerocopy::little_endian::U32;

use crate::hid::Handle;

pub const REPORT_LEN: usize = 64;

type Report = [u8; REPORT_LEN];

#[bytes]
#[repr(u8)]
pub enum Opcode {
    Commit = 0x02,
    ReadDeviceInfo = 0x05,
    ReadKeymapBase = 0x10,
    WriteKeymapBase = 0x11,
    ReadLighting = 0x12,
    WriteLighting = 0x13,
    ReadMacros = 0x14,
    WriteMacros = 0x15,
    ReadSettings = 0x16,
    WriteSettings = 0x17,
    SessionOpen = 0x18,
    ReadKeyColors = 0x22,
    WriteKeyColors = 0x23,
    ReadKeymapFn = 0x26,
    WriteKeymapFn = 0x27,
    Ping = 0xE0,
    ApplyReload = 0xF0,
}

#[bytes]
#[repr(u8)]
pub enum Status {
    Pending = 0x00,
    Accepted = 0x01,
    #[expect(dead_code, reason = "we never send this, only receive it from firmware")]
    Rejected = 0xFF,
}

#[bytes]
#[derive(ToBytes)]
#[repr(C, packed)]
pub struct Command {
    magic: u8,
    opcode: Opcode,
    sub: u8,
    status: Status,
    sum: U32,
    blocks: u8,
    _reserved: [u8; 55],
}

impl Command {
    pub const MAGIC: u8 = 0x04;
    pub const SIZE: usize = size_of::<Self>();

    pub const fn new(opcode: Opcode, blocks: u8) -> Self {
        Self {
            magic: Self::MAGIC,
            opcode,
            sub: 0,
            status: Status::Pending,
            sum: U32::ZERO,
            blocks,
            _reserved: [0; 55],
        }
    }

    pub const fn is_valid(&self) -> bool {
        self.magic == Self::MAGIC
    }

    pub fn is_ack(&self, opcode: Opcode) -> bool {
        self.answers(opcode) && self.status == Status::Accepted
    }

    pub fn answers(&self, opcode: Opcode) -> bool {
        self.is_valid() && self.opcode == opcode
    }

    pub const fn sum(&self) -> u32 {
        self.sum.get()
    }
}

impl FromBytes for Command {
    const SIZE: usize = Self::SIZE;

    fn from_bytes(raw: &[u8]) -> Option<Self> {
        let command = <Self as zerocopy::TryFromBytes>::try_read_from_bytes(raw).ok()?;
        command.is_valid().then_some(command)
    }
}

#[derive(Debug, Clone, Copy)]
struct RegionMap {
    /// The opcode to read the region
    read: Opcode,
    /// The opcode to write the region, if any. Some regions are RO.
    write: Option<Opcode>,
    blocks: u8,
    /// Whether the device must be told to reload this region after writing it,
    /// to make the changes take effect.
    reloads: bool,
}

impl TryFrom<Region> for RegionMap {
    type Error = Error;

    fn try_from(region: Region) -> Result<Self> {
        let (read, write, blocks, reloads) = match region {
            Region::DeviceInfo => (Opcode::ReadDeviceInfo, None, 1, false),
            Region::Settings => (Opcode::ReadSettings, Some(Opcode::WriteSettings), 1, false),
            Region::Lighting => (Opcode::ReadLighting, Some(Opcode::WriteLighting), 1, true),
            Region::KeyColors => (Opcode::ReadKeyColors, Some(Opcode::WriteKeyColors), 9, true),
            Region::Keymap(0) => (Opcode::ReadKeymapBase, Some(Opcode::WriteKeymapBase), 9, true),
            Region::Keymap(1) => (Opcode::ReadKeymapFn, Some(Opcode::WriteKeymapFn), 9, true),
            Region::Macros => (Opcode::ReadMacros, Some(Opcode::WriteMacros), 48, false),
            Region::Keymap(_) => return Err(Error::UnsupportedOnTransport(TransportKind::Wired)),
        };

        Ok(Self { read, write, blocks, reloads })
    }
}

#[derive(Debug)]
pub struct Wired {
    hid: Handle,
    quirks: Quirks,
}

impl Wired {
    /// The firmware keeps sending the same thing over and over, until we acknowledge it.
    const MAX_EQUAL_PACKETS: usize = 1;
    const MAX_READS: usize = 16;
    const REPLY_TIMEOUT: Duration = Duration::from_millis(150);
    const SYNC_PINGS: usize = 1;

    pub(crate) const fn new(hid: Handle, quirks: Quirks) -> Self {
        Self { hid, quirks }
    }

    fn send(&mut self, report: &Report) -> Result<()> {
        sleep(Duration::from_millis(u64::from(self.quirks.inter_transfer_delay_ms)));
        self.hid.set_feature(report)
    }

    fn recv(&mut self) -> Result<Report> {
        sleep(Duration::from_millis(u64::from(self.quirks.inter_transfer_delay_ms)));
        let mut report = [0u8; REPORT_LEN];
        self.hid.get_feature(&mut report)?;
        Ok(report)
    }

    fn send_command(&mut self, opcode: Opcode, blocks: u8) -> Result<()> {
        self.send(&Command::new(opcode, blocks).to_bytes())
    }

    fn recv_ack(&mut self, opcode: Opcode) -> Result<Command> {
        let timeout = Instant::now() + Self::REPLY_TIMEOUT;

        while Instant::now() < timeout {
            let report = self.recv()?;

            let Some(reply) = Command::from_bytes(&report).filter(|r| r.answers(opcode)) else {
                continue;
            };

            match reply.status {
                Status::Accepted => return Ok(reply),
                Status::Rejected => return Err(Error::Nak),
                Status::Pending => {}
            }
        }

        Err(Error::Desync)
    }

    fn request(&mut self, opcode: Opcode, blocks: u8) -> Result<Command> {
        self.send_command(opcode, blocks)?;

        match self.recv_ack(opcode) {
            Err(Error::Nak) => {
                self.send_command(opcode, blocks)?;
                self.recv_ack(opcode)
            }
            other => other,
        }
    }

    fn resync(&mut self) -> Result<()> {
        let mut previous = None;
        let mut repeats = 0;

        for _ in 0..Self::MAX_READS {
            let report = self.recv()?;

            if previous == Some(report) {
                repeats += 1;
            } else {
                repeats = 1;
                previous = Some(report);
            }

            if repeats >= Self::MAX_EQUAL_PACKETS {
                break;
            }
        }

        for _ in 0..Self::SYNC_PINGS {
            self.request(Opcode::Ping, 0)?;
        }

        Ok(())
    }

    fn begin(&mut self) -> Result<()> {
        self.resync()?;
        self.request(Opcode::SessionOpen, 0)?;
        Ok(())
    }

    fn read_region(&mut self, region: Region) -> Result<Vec<u8>> {
        let map = RegionMap::try_from(region)?;
        self.begin()?;
        self.send_command(map.read, map.blocks)?;

        let mut remaining = usize::from(map.blocks);
        let mut data = Vec::with_capacity(remaining * REPORT_LEN);

        let first = self.recv()?;

        if !Command::from_bytes(&first).is_some_and(|reply| reply.is_ack(map.read)) {
            data.extend_from_slice(&first);
            remaining -= 1;
        }

        for _ in 0..remaining {
            data.extend_from_slice(&self.recv()?);
        }

        Ok(data)
    }

    fn write_region(&mut self, region: Region, payload: &[u8]) -> Result<()> {
        let map = RegionMap::try_from(region)?;
        let Some(opcode) = map.write else {
            return Err(Error::UnsupportedOnTransport(TransportKind::Wired));
        };

        let blocks = u8::try_from(payload.len().div_ceil(REPORT_LEN)).unwrap_or(u8::MAX);
        if blocks > map.blocks {
            return Err(Error::RegionOverflow { blocks, limit: map.blocks });
        }

        self.begin()?;
        self.request(opcode, blocks)?;

        for chunk in payload.chunks(REPORT_LEN) {
            let mut block = [0u8; REPORT_LEN];
            block[..chunk.len()].copy_from_slice(chunk);
            self.send(&block)?;
        }

        self.commit(payload)?;

        if map.reloads {
            self.request(Opcode::ApplyReload, 0)?;
        }
        Ok(())
    }

    fn commit(&mut self, payload: &[u8]) -> Result<()> {
        let reported = self.request(Opcode::Commit, 0)?.sum();
        let expected: u32 = payload.iter().copied().map(u32::from).sum();

        if reported == expected { Ok(()) } else { Err(Error::Checksum { reported, expected }) }
    }

    pub(crate) fn supports(&self, req: &RegionRequest) -> bool {
        match req {
            RegionRequest::Read(region) => RegionMap::try_from(*region).is_ok(),
            RegionRequest::Write { region, .. } => {
                RegionMap::try_from(*region).is_ok_and(|map| map.write.is_some())
            }
            RegionRequest::WriteSlot { .. } => false,
            RegionRequest::Reload | RegionRequest::Ping => true,
        }
    }

    pub(crate) fn execute(&mut self, req: RegionRequest) -> Result<Vec<u8>> {
        match req {
            RegionRequest::Read(region) => self.read_region(region),
            RegionRequest::Write { region, data } => {
                self.write_region(region, &data).map(|()| Vec::new())
            }
            RegionRequest::WriteSlot { .. } => {
                Err(Error::UnsupportedOnTransport(TransportKind::Wired))
            }
            RegionRequest::Reload => self.request(Opcode::ApplyReload, 0).map(|_| Vec::new()),
            RegionRequest::Ping => self.request(Opcode::Ping, 0).map(|_| Vec::new()),
        }
    }
}
