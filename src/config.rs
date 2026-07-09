use std::{
    env::{self, VarError},
    fmt::Display,
    fs::read_to_string,
    io,
};

use directories::ProjectDirs;
use log::error;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Config {
    pub icons: IconSetting,
}

impl Config {
    pub fn read_from_config_dir() -> Option<Self> {
        let config_path = match ProjectDirs::from("io", "jeffa", "tiefling") {
            Some(path) => path.config_dir().join("config.toml"),
            None => return None,
        };
        let config_text = if config_path.is_file() {
            match read_to_string(config_path) {
                Ok(text) => text,
                Err(error) => {
                    error.log();

                    return None;
                }
            }
        } else {
            return None;
        };

        match toml::de::from_str(&config_text) {
            Ok(config) => Some(config),
            Err(error) => {
                error.log();

                None
            }
        }
    }

    pub fn read_or_default() -> Self {
        let user_config = Self::read_from_config_dir();

        Config {
            icons: IconSetting::from_env()
                .or(user_config.map(|config| config.icons))
                .unwrap_or_default(),
        }
    }
}

trait ConfigOption: Default {
    const ENVIRONMENT_VARIABLE: &'static str;

    fn from_env() -> Option<Self>;
}

#[derive(Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub enum IconSetting {
    Emoji,
    #[default]
    JetBrains,
}

impl ConfigOption for IconSetting {
    const ENVIRONMENT_VARIABLE: &'static str = "TIEFLING_ICONS";

    fn from_env() -> Option<Self> {
        let env_var = match env::var(Self::ENVIRONMENT_VARIABLE) {
            Ok(var) => var,
            Err(error) => {
                if error != VarError::NotPresent {
                    error.log();
                }

                return None;
            }
        };

        match env_var.as_str() {
            "jetbrains" => Some(Self::JetBrains),
            "emoji" => Some(Self::Emoji),
            _ => None,
        }
    }
}

trait ConfigError: Display {
    fn log(&self) {
        error!("{self}")
    }
}

impl ConfigError for io::Error {}
impl ConfigError for toml::de::Error {}
impl ConfigError for VarError {}
