use std::fmt::Display;

use anyhow::{Result, anyhow};
use aurora::Device;
use aurora::common::{Debounce, Settings, SleepTimeout};
use clap::builder::{PossibleValuesParser, TypedValueParser};
use clap_num::number_range;

use crate::cmds::{CliCommand, CommandMetadata};
use crate::config::AuroraConfig;
use crate::style;

#[derive(clap::Args, Debug, Clone)]
pub struct SettingsArgs {
    /// Disable alt-tab
    #[arg(long = "alt-tab-dis", value_name = "BOOL")]
    pub alt_tab_disabled: Option<bool>,

    /// Disable alt-f4
    #[arg(long = "alt-f4-dis", value_name = "BOOL")]
    pub alt_f4_disabled: Option<bool>,

    /// Disable the Windows/Super key
    #[arg(long = "win-dis", value_name = "BOOL")]
    pub win_disabled: Option<bool>,

    /// Set the Fn key to toggle or hold mode
    #[arg(long, value_name = "BOOL")]
    pub fn_toggle: Option<bool>,

    /// Set the keyboard sleep timeout
    #[arg(long, value_parser = sleep_timeout())]
    pub sleep: Option<SleepTimeout>,

    /// Set the debounce level
    #[arg(
        long,
        value_name = "1-5",
        value_parser = |raw: &str| number_range(raw, 1, 5)
            .and_then(|level| Debounce::try_from(level).map_err(|e| e.to_string()))
    )]
    pub debounce: Option<Debounce>,

    #[arg(
        long,
        help = "Force writing settings even when the current ones cannot be read (resets non specified fields)"
    )]
    pub force: bool,
}

impl CommandMetadata for SettingsArgs {
    fn about() -> &'static str {
        "Show or change keyboard settings"
    }

    fn long_about() -> &'static str {
        "Show the current keyboard settings, or change them by providing the corresponding flags."
    }
}

impl SettingsArgs {
    const fn changes(&self) -> bool {
        self.alt_tab_disabled.is_some()
            || self.alt_f4_disabled.is_some()
            || self.win_disabled.is_some()
            || self.fn_toggle.is_some()
            || self.sleep.is_some()
            || self.debounce.is_some()
    }

    const fn apply(&self, settings: &mut Settings) {
        if let Some(value) = self.alt_tab_disabled {
            settings.set_alt_tab_disabled(value);
        }
        if let Some(value) = self.alt_f4_disabled {
            settings.set_alt_f4_disabled(value);
        }
        if let Some(value) = self.win_disabled {
            settings.set_win_disabled(value);
        }
        if let Some(value) = self.fn_toggle {
            settings.set_fn_toggle(value);
        }
        if let Some(value) = self.sleep {
            settings.set_sleep(value);
        }
        if let Some(value) = self.debounce {
            settings.set_debounce(value);
        }
    }
}

impl CliCommand for SettingsArgs {
    fn run(&self, dev: &mut Device, _config: &AuroraConfig) -> Result<()> {
        let before = match dev.settings() {
            Ok(settings) => Some(settings),
            Err(_) if self.force && self.changes() => None,
            Err(e) if self.changes() => {
                return Err(anyhow!(
                    "{e}\n\tpass --force to write anyway (resets the non specified fields)"
                ));
            }
            Err(e) => return Err(e.into()),
        };

        let base = before.unwrap_or_default();
        let mut after = base;
        self.apply(&mut after);

        if before.is_some() && after == base {
            style::title("Keyboard settings");
            report(&base, &base);
            return Ok(());
        }

        dev.set_settings(&after)?;

        style::title("Updated settings");
        report(&dev.settings().unwrap_or(after), &before.unwrap_or(after));
        Ok(())
    }
}

fn report(now: &Settings, before: &Settings) {
    row("alt-tab", now, before, |s| style::switch(!s.alt_tab_disabled()));
    row("alt-f4", now, before, |s| style::switch(!s.alt_f4_disabled()));
    row("win key", now, before, |s| style::switch(!s.win_disabled()));
    row("fn key", now, before, |s| fn_mode(s.fn_toggle()));
    row("sleep", now, before, Settings::sleep);
    row("debounce", now, before, |s| s.debounce() as u8);
}

fn row<T: Display>(label: &str, now: &Settings, before: &Settings, show: impl Fn(&Settings) -> T) {
    style::update(label, show(now), show(before));
}

const fn fn_mode(toggle: bool) -> &'static str {
    if toggle { "toggle" } else { "hold" }
}

fn sleep_timeout() -> impl TypedValueParser<Value = SleepTimeout> {
    PossibleValuesParser::new(["never", "1m", "5m", "30m"]).map(|raw| match raw.as_str() {
        "never" => SleepTimeout::Never,
        "1m" => SleepTimeout::OneMinute,
        "5m" => SleepTimeout::FiveMinutes,
        _ => SleepTimeout::ThirtyMinutes,
    })
}
