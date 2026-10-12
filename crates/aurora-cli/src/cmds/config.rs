use anyhow::{Result, bail};
use aurora::Device;
use serde_json::Value;

use crate::cmds::{CliCommand, CommandMetadata};
use crate::config::AuroraConfig;
use crate::style;

#[derive(clap::Args, Debug, Clone)]
pub struct ConfigArgs {
    #[arg(value_name = "KEY")]
    pub key: Option<String>,

    #[arg(value_name = "VALUE")]
    pub value: Option<String>,
}

impl CommandMetadata for ConfigArgs {
    fn aliases() -> &'static [&'static str] {
        &["cfg"]
    }

    fn about() -> &'static str {
        "Show or change stored configuration"
    }
}

impl CliCommand for ConfigArgs {
    fn run(&self, _dev: &mut Device, config: &AuroraConfig) -> Result<()> {
        let Value::Object(mut fields) = serde_json::to_value(config)? else {
            bail!("configuration is invalid");
        };

        let Some(key) = self.key.as_deref() else {
            style::title("Aurora config");
            for (name, value) in &fields {
                style::field(name, show(value));
            }
            return Ok(());
        };

        let Some(current) = fields.get(key) else {
            bail!("unknown config key {key}");
        };

        let Some(value) = self.value.as_deref() else {
            style::field(key, show(current));
            return Ok(());
        };

        let parsed = parse(current, value);
        let previous = show(current);
        fields.insert(key.to_owned(), parsed);

        let updated: AuroraConfig = serde_json::from_value(Value::Object(fields))?;
        updated.save()?;

        style::title("Updated config");
        style::update(key, show(&serde_json::to_value(&updated)?[key]), previous);
        Ok(())
    }
}

fn parse(current: &Value, value: &str) -> Value {
    if value.is_empty() {
        return Value::Null;
    }

    match current {
        Value::String(_) => Value::String(value.to_owned()),
        _ => serde_json::from_str(value).unwrap_or_else(|_| Value::String(value.to_owned())),
    }
}

fn show(value: &Value) -> String {
    match value {
        Value::Null => "(unset)".to_owned(),
        Value::String(text) => text.clone(),
        other => other.to_string(),
    }
}
