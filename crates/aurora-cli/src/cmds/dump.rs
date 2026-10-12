use std::fs;
use std::path::PathBuf;

use anyhow::Result;
use aurora::Device;
use aurora::op::{Region, RegionRequest};

use crate::cmds::{CliCommand, CommandMetadata};
use crate::config::AuroraConfig;
use crate::style;

#[derive(clap::Args, Debug, Clone)]
pub struct DumpArgs {
    /// Path to a directory to write the dump to
    #[arg(value_name = "DIR")]
    pub dir: PathBuf,

    #[arg(long, value_name = "NAME", help = "Dump only this region instead of every one")]
    pub region: Option<Region>,
}

impl CommandMetadata for DumpArgs {
    fn about() -> &'static str {
        "Dump the keyboard region contents to a folder"
    }

    fn long_about() -> &'static str {
        "Dump the contents of every region to a folder, or the specified region.
        If the transport does not support reading a region, it will be skipped."
    }
}

impl CliCommand for DumpArgs {
    fn run(&self, dev: &mut Device, _config: &AuroraConfig) -> Result<()> {
        if let Some(region) = self.region {
            dev.has_region(region)?;
        }

        fs::create_dir_all(&self.dir)?;
        style::title(format!("Dumping to \'{}\'", self.dir.display()));

        for region in dev.board().regions() {
            if self.region.is_some_and(|only| only != region) {
                continue;
            }

            let dump_result = dev.run(RegionRequest::Read(region)).and_then(|raw| {
                fs::write(self.dir.join(format!("{region}.bin")), &raw)?;
                Ok(raw.len())
            });

            match dump_result {
                Ok(size) => {
                    style::field(&region.to_string(), format!("{size} bytes"));
                }
                Err(e) => {
                    style::field(&region.to_string(), style::warn(format_args!("skipped ({e})")));
                }
            }
        }

        Ok(())
    }
}
