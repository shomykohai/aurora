use std::thread::sleep;
use std::time::Duration;

use aurora_protocol::board::{Board, KeyDesc, TransportKind};
use aurora_protocol::cap::{Cap, CapId, Params};
use aurora_protocol::common::{DeviceInfo, Settings};
use aurora_protocol::error::{Error, Result};
use aurora_protocol::keys::{Key, KeyAction, KeySlot, Keymap};
use aurora_protocol::leds::{LedTable, LightMode, LightingConfig};
use aurora_protocol::op::{Region, RegionRequest};
use aurora_protocol::traits::{FromBytes, ToBytes};
use aurora_transport::Connection;
use rgb::RGB8;

#[derive(Debug)]
pub struct Device {
    connection: Connection,
    board: &'static Board,
}

impl Device {
    pub fn find() -> Result<Self> {
        Self::probe(aurora_boards::ALL.iter().copied())
    }

    /// Opens one board, and fails if that particular board is not attached.
    pub fn find_board(board: &'static Board) -> Result<Self> {
        Self::probe([board])
    }

    fn probe(boards: impl IntoIterator<Item = &'static Board>) -> Result<Self> {
        let (connection, board) = Connection::connect(boards)?;
        Ok(Self { connection, board })
    }

    /// The transport kind used to interact with the board.
    pub const fn transport(&self) -> TransportKind {
        self.connection.kind()
    }

    pub const fn board(&self) -> &'static Board {
        self.board
    }

    /// Returns the board's matching KeyDesc for the given Key
    pub fn key(&self, key: Key) -> Result<&'static KeyDesc> {
        self.board.key(key).ok_or(Error::UnknownKey(key))
    }

    /// Returns all board's defined keys
    pub const fn keys(&self) -> &'static [KeyDesc] {
        self.board.keys
    }

    /// Returns all board's defined layers
    pub const fn layers(&self) -> &'static [&'static str] {
        self.board.layers
    }

    /// Returns the index of the layer with the given name, or None if not found
    pub fn layer(&self, name: &str) -> Option<u8> {
        self.board.layer(name)
    }

    /// Return's whether the board supports the given Capability.
    pub fn supports(&self, id: CapId) -> bool {
        self.board.supports(id)
    }

    pub fn has_region(&self, region: Region) -> Result<Region> {
        self.cap(region.into())?;

        match region {
            Region::Keymap(layer) if usize::from(layer) >= self.board.layers.len() => {
                Err(Error::UnknownLayer(layer))
            }
            _ => Ok(region),
        }
    }

    /// Returns the board's Cap for the given CapId, with the defined parameters if present.
    pub fn cap(&self, id: CapId) -> Result<&'static Cap> {
        self.board.caps.iter().find(|c| c.id == id).ok_or(Error::UnsupportedCap(id))
    }

    /// Sets the colour of a single key. The board must support the KeyColors capability.
    pub fn set_key_color(&mut self, key: Key, color: RGB8) -> Result<()> {
        let index = self.key(key)?.light_index;
        let mut table = self.key_colors()?;
        let slot = Self::slot_at(table.slots_mut(), key, index)?;
        slot.index = index;
        slot.set_color(color);
        self.set_key_colors(&table)
    }

    /// Sets a single colour for all keys. The board must support the Lighting capability.
    pub fn set_all_colors(&mut self, color: RGB8) -> Result<()> {
        self.edit_lighting(|config| {
            config.set_color(color);
            config.set_mode(LightMode::Static);
        })
    }

    /// Sets the action of a single key. The board must support the Keymap capability.
    pub fn set_key(&mut self, layer: u8, key: Key, action: KeyAction) -> Result<()> {
        let index = self.key(key)?.key_index;
        let mut map = self.keymap(layer)?;
        *Self::slot_at(map.slots_mut(), key, index)? = KeySlot::from(action);
        self.set_keymap(layer, &map)
    }

    /// Restore the function of a single key to its default
    pub fn reset_key(&mut self, layer: u8, key: Key) -> Result<()> {
        self.set_key(layer, key, KeyAction::Default)
    }

    /// Returns the device info. The board must support the DeviceInfo capability.
    pub fn device_info(&mut self) -> Result<DeviceInfo> {
        self.read_region(Region::DeviceInfo)
    }

    /// Returns the board settings. The board must support the Settings capability.
    pub fn settings(&mut self) -> Result<Settings> {
        self.read_region(Region::Settings)
    }

    /// Sets the board settings. The board must support the Settings capability.
    pub fn set_settings(&mut self, value: &Settings) -> Result<()> {
        self.write_region(Region::Settings, value)
    }

    /// Edits the board settings. The board must support the Settings capability.
    pub fn edit_settings(&mut self, f: impl FnOnce(&mut Settings)) -> Result<()> {
        self.edit_region(Region::Settings, f)
    }

    /// Returns the lighting configuration. The board must support the Lighting capability.
    pub fn lighting(&mut self) -> Result<LightingConfig> {
        self.read_region(Region::Lighting)
    }

    /// Sets the lighting configuration. The board must support the Lighting capability.
    pub fn set_lighting(&mut self, value: &LightingConfig) -> Result<()> {
        self.write_region(Region::Lighting, value)
    }

    /// Edits the lighting configuration. The board must support the Lighting capability.
    pub fn edit_lighting(&mut self, f: impl FnOnce(&mut LightingConfig)) -> Result<()> {
        self.edit_region(Region::Lighting, f)
    }

    /// Returns the keymap for the given layer. The board must support the Keymap capability.
    pub fn keymap(&mut self, layer: u8) -> Result<Keymap> {
        self.read_region(self.has_region(Region::Keymap(layer))?)
    }

    /// Sets the keymap for the given layer. The board must support the Keymap capability.
    pub fn set_keymap(&mut self, layer: u8, value: &Keymap) -> Result<()> {
        self.write_region(self.has_region(Region::Keymap(layer))?, value)
    }

    /// Edits the keymap for the given layer. The board must support the Keymap capability.
    pub fn edit_keymap(&mut self, layer: u8, f: impl FnOnce(&mut Keymap)) -> Result<()> {
        self.edit_region(self.has_region(Region::Keymap(layer))?, f)
    }

    /// Returns the key colours (each key with its own). The board must support the KeyColors
    /// capability.
    pub fn key_colors(&mut self) -> Result<LedTable> {
        self.read_region(Region::KeyColors)
    }

    /// Sets the key colours (each key with its own). The board must support the KeyColors
    /// capability.
    pub fn set_key_colors(&mut self, value: &LedTable) -> Result<()> {
        self.write_region(Region::KeyColors, value)?;
        self.show_stored_colors()
    }

    /// Edits the key colours (each key with its own). The board must support the KeyColors
    /// capability.
    pub fn edit_key_colors(&mut self, f: impl FnOnce(&mut LedTable)) -> Result<()> {
        let mut value = self.key_colors()?;
        f(&mut value);
        self.set_key_colors(&value)
    }

    /// Pings the device to check if it is alive.
    pub fn ping(&mut self) -> Result<()> {
        self.run(RegionRequest::Ping)?;
        Ok(())
    }

    /// Reloads the device to apply any changes made. This is usually called automatically after a
    /// write operation, but can be called manually if needed.
    pub fn reload(&mut self) -> Result<()> {
        self.run(RegionRequest::Reload)?;
        Ok(())
    }

    pub fn validate_region(&self, region: Region, data: &[u8]) -> Result<Vec<u8>> {
        match self.has_region(region)? {
            Region::DeviceInfo => Err(Error::ReadOnly(region)),
            Region::Settings => Self::validate_as::<Settings>(region, data),
            Region::Lighting => Self::validate_as::<LightingConfig>(region, data),
            Region::KeyColors => Self::validate_as::<LedTable>(region, data),
            Region::Keymap(_) => Self::validate_as::<Keymap>(region, data),
            Region::Macros => Err(Error::Unverified(region)),
        }
    }

    pub fn restore(&mut self, region: Region, data: &[u8]) -> Result<bool> {
        if self.transport() == TransportKind::Dongle {
            return Err(Error::UnsupportedOnTransport(TransportKind::Dongle));
        }

        let data = self.validate_region(region, data)?;

        if self.run(RegionRequest::Read(region))?.get(..data.len()) == Some(data.as_slice()) {
            return Ok(false);
        }

        self.run(RegionRequest::Write { region, data: data.clone() })?;
        sleep(Duration::from_millis(u64::from(self.board.quirks.flash_program_delay_ms)));

        if self.run(RegionRequest::Read(region))?.get(..data.len()) == Some(data.as_slice()) {
            Ok(true)
        } else {
            Err(Error::WriteLost(region))
        }
    }

    /// Executes a region request.
    pub fn run(&mut self, req: RegionRequest) -> Result<Vec<u8>> {
        if !self.connection.supports(&req) {
            return Err(Error::UnsupportedOnTransport(self.connection.kind()));
        }

        self.connection.execute(req)
    }

    /* Behold, the comment to visually separate pub and private methods!! */

    // Set the light mode needed for showing custom colours.
    fn show_stored_colors(&mut self) -> Result<()> {
        let Params::Rgb(rgb) = self.cap(CapId::KeyColors)?.params else { return Ok(()) };
        let Some(wanted) = rgb.needs_mode else { return Ok(()) };

        let mut lighting: LightingConfig = self.read_region(Region::Lighting)?;

        if lighting.mode() == wanted {
            return Ok(());
        }

        lighting.set_mode(wanted);
        self.write_region(Region::Lighting, &lighting)
    }

    fn validate_as<S: FromBytes + ToBytes>(region: Region, raw: &[u8]) -> Result<Vec<u8>> {
        let (record, padding) =
            raw.split_at_checked(<S as FromBytes>::SIZE).ok_or(Error::InvalidRecord(region))?;

        if !padding.iter().all(|&byte| byte == 0) {
            return Err(Error::InvalidRecord(region));
        }

        let value = S::from_bytes(record).ok_or(Error::InvalidRecord(region))?;
        Ok(value.to_bytes().as_ref().to_vec())
    }

    fn slot_at<S>(slots: &mut [S], key: Key, index: u8) -> Result<&mut S> {
        slots.get_mut(usize::from(index)).ok_or(Error::SlotOutOfRange { key, index })
    }

    fn read_region<S: FromBytes>(&mut self, region: Region) -> Result<S> {
        self.cap(region.into())?;
        let raw = self.run(RegionRequest::Read(region))?;
        let raw = raw.get(..S::SIZE).ok_or(Error::Desync)?;
        S::from_bytes(raw).ok_or(Error::InvalidRegion(region))
    }

    fn write_region<S: ToBytes>(&mut self, region: Region, value: &S) -> Result<()> {
        self.cap(region.into())?;
        let data = value.to_bytes().as_ref().to_vec();
        self.run(RegionRequest::Write { region, data })?;
        Ok(())
    }

    fn edit_region<S: FromBytes + ToBytes>(
        &mut self,
        region: Region,
        f: impl FnOnce(&mut S),
    ) -> Result<()> {
        let mut value: S = self.read_region(region)?;
        f(&mut value);
        self.write_region(region, &value)
    }
}
