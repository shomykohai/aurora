use anyhow::Result;
use aurora::{Device, TransportKind};

use crate::cmds::{CliCommand, CommandMetadata};
use crate::config::AuroraConfig;
use crate::style;

#[derive(clap::Args, Debug, Clone)]
pub struct InfoArgs {}

impl CommandMetadata for InfoArgs {
    fn about() -> &'static str {
        "Show general keyboard info"
    }
}

impl CliCommand for InfoArgs {
    fn run(&self, dev: &mut Device, _config: &AuroraConfig) -> Result<()> {
        let board = dev.board();

        style::title(board.name);
        style::field("id", board.id);
        style::field("connection", match dev.transport() {
            TransportKind::Wired => "USB cable",
            TransportKind::Dongle => "2.4GHz dongle",
        });

        match dev.device_info() {
            Ok(info) => {
                style::field("usb", format!("{:04X}:{:04X}", info.vid.get(), info.pid.get()));
                style::field("firmware", format!("{:04X}", info.bcd_device.get()));
                style::field(
                    "transfer",
                    format!(
                        "{} bytes payload, {} max blocks",
                        info.payload_size.get(),
                        info.max_blocks.get()
                    ),
                );
            }
            Err(e) => style::field("firmware", style::warn(format_args!("unavailable ({e})"))),
        }

        style::field("layers", board.layers.join(", "));
        style::field("keys", board.keys.len());
        style::field(
            "capabilities",
            board.caps.iter().map(|c| c.to_string()).collect::<Vec<_>>().join(", "),
        );

        Ok(())
    }
}
