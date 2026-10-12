use std::fs;
use std::path::PathBuf;

use anyhow::{Result, anyhow, bail};
use aurora::op::Region;
use aurora::{Device, Error};

use crate::cmds::{CliCommand, CommandMetadata};
use crate::config::AuroraConfig;
use crate::style;

#[derive(clap::Args, Debug, Clone)]
pub struct RestoreArgs {
    /// Path to a directory containing a dump
    #[arg(value_name = "DIR")]
    pub dir: PathBuf,

    #[arg(long, value_name = "NAME", help = "Restore only this region instead of every one")]
    pub region: Option<Region>,
}

impl CommandMetadata for RestoreArgs {
    fn about() -> &'static str {
        "Write regions back from a folder made by dump"
    }

    fn long_about() -> &'static str {
        "Restore a dump made by the dump command, and validate each region before writing it."
    }
}

impl CliCommand for RestoreArgs {
    fn run(&self, dev: &mut Device, _config: &AuroraConfig) -> Result<()> {
        if let Some(region) = self.region {
            dev.has_region(region)?;
        }

        style::title(format!("Restoring from {}", self.dir.display()));

        let mut to_restore = Vec::new();

        for region in
            dev.board().regions().filter(|region| self.region.is_none_or(|only| only == *region))
        {
            let checked = fs::read(self.dir.join(format!("{region}.bin")))
                .map_err(Error::Io)
                .and_then(|raw| dev.validate_region(region, &raw));

            match checked {
                Ok(data) => to_restore.push((region, data)),
                Err(e) => {
                    style::field(&region.to_string(), style::warn(format_args!("skipped ({e})")));
                }
            }
        }

        if to_restore.is_empty() {
            bail!("nothing in {} could be restored", self.dir.display());
        }

        for (region, data) in to_restore {
            let written = dev
                .restore(region, &data)
                .map(|changed| if changed { "written" } else { "unchanged" });

            match &written {
                Ok(state) => style::field(&region.to_string(), state),
                Err(e) => {
                    style::field(&region.to_string(), style::warn(format_args!("failed ({e})")));
                }
            }

            written.map_err(|e| anyhow!("failed to restore {region}: {e}"))?;
        }

        Ok(())
    }
}
