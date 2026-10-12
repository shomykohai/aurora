use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Result;
use config::{Config, Environment, File};
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Deserialize, Clone, Serialize)]
pub struct AuroraConfig {
    pub default_board: Option<String>,
}

impl AuroraConfig {
    pub fn load() -> Result<Arc<Self>> {
        let mut builder = Config::builder();

        builder = builder.set_default("default_board", None::<String>)?;

        if let Some(path) = Self::get_path() {
            builder = builder.add_source(File::from(path).required(false));
        }

        builder = builder.add_source(Environment::with_prefix("AURORA"));
        let cfg = match builder.build().and_then(|c| c.try_deserialize::<Self>()) {
            Ok(cfg) => {
                cfg.save()?;
                cfg
            }
            Err(e) => {
                println!("Could not read config ({e})");
                Self::default()
            }
        };

        Ok(Arc::new(cfg))
    }

    pub fn save(&self) -> Result<()> {
        if let Some(path) = Self::get_path() {
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)?;
            }

            fs::write(path, serde_json::to_string_pretty(self)?)?;
        }

        Ok(())
    }

    fn get_path() -> Option<PathBuf> {
        dirs::config_dir().map(|p| p.join("aurora/config.json"))
    }
}
