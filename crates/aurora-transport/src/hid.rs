use std::io::{Error as IoError, ErrorKind};
use std::time::Duration;

use aurora_protocol::{Error, Result, UsbId};
use hidapi::{DeviceInfo, HidApi, HidDevice};

// Unused by the original software
const REPORT_ID: u8 = 0x00;

/// Find the matching HID device to the given ID from the board.
pub fn select<'a>(api: &'a HidApi, id: &UsbId) -> Option<&'a DeviceInfo> {
    api.device_list()
        .filter(|node| node.vendor_id() == id.vid && node.product_id() == id.pid)
        .filter(|node| node.usage_page() == id.usage_page)
        .filter(|node| {
            id.bcd.is_none_or(|(low, high)| (low..=high).contains(&node.release_number()))
        })
        .filter(|node| id.interface.is_none_or(|n| node.interface_number() == n))
        .min_by_key(|node| (node.interface_number(), node.path()))
}

#[derive(Debug)]
pub struct Handle {
    device: HidDevice,
    buffer: Vec<u8>,
}

impl Handle {
    pub(crate) fn open(api: &HidApi, node: &DeviceInfo) -> Result<Self> {
        let device = api.open_path(node.path()).map_err(IoError::other)?;
        Ok(Self { device, buffer: Vec::new() })
    }

    /// Sends a feature report.
    pub(crate) fn set_feature(&mut self, report: &[u8]) -> Result<()> {
        self.frame(report);
        self.device.send_feature_report(&self.buffer).map_err(IoError::other)?;
        Ok(())
    }

    /// Gets a feature report
    pub(crate) fn get_feature(&mut self, report: &mut [u8]) -> Result<()> {
        self.buffer.clear();
        self.buffer.resize(report.len() + 1, REPORT_ID);

        let read = self.device.get_feature_report(&mut self.buffer).map_err(IoError::other)?;
        if read < self.buffer.len() {
            let short = format!("device returned {read} of {} bytes", self.buffer.len());
            return Err(Error::Io(IoError::new(ErrorKind::UnexpectedEof, short)));
        }

        report.copy_from_slice(&self.buffer[1..]);
        Ok(())
    }

    /// Interrupt write
    pub(crate) fn write(&mut self, report: &[u8]) -> Result<()> {
        self.frame(report);
        self.device.write(&self.buffer).map_err(IoError::other)?;
        Ok(())
    }

    /// Interrupt read
    pub(crate) fn read(&self, report: &mut [u8], timeout: Duration) -> Result<usize> {
        let millis = i32::try_from(timeout.as_millis()).unwrap_or(i32::MAX);
        Ok(self.device.read_timeout(report, millis).map_err(IoError::other)?)
    }

    fn frame(&mut self, report: &[u8]) {
        self.buffer.clear();
        self.buffer.push(REPORT_ID);
        self.buffer.extend_from_slice(report);
    }
}
