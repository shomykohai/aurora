use crate::cli_commands;

mod config;
mod dump;
mod info;
mod restore;
mod settings;

pub use config::ConfigArgs;
pub use dump::DumpArgs;
pub use info::InfoArgs;
pub use restore::RestoreArgs;
pub use settings::SettingsArgs;

pub trait CommandMetadata {
    fn aliases() -> &'static [&'static str] {
        &[]
    }

    fn visible_aliases() -> &'static [&'static str] {
        &[]
    }

    fn about() -> &'static str {
        ""
    }

    fn long_about() -> &'static str {
        ""
    }

    fn hide() -> bool {
        false
    }
}

cli_commands!(
    Config(ConfigArgs),
    Info(InfoArgs),
    Settings(SettingsArgs),
    Dump(DumpArgs),
    Restore(RestoreArgs),
);

pub trait CliCommand {
    fn run(
        &self,
        dev: &mut aurora::Device,
        config: &crate::config::AuroraConfig,
    ) -> anyhow::Result<()>;
}
