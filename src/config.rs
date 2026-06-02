use serde::{Deserialize, Serialize};
use figment::{Figment, providers::{Format, Toml, Env}};
use std::path::PathBuf;
use crate::display::long::Column;

#[derive(Deserialize, Serialize, Debug, Default)]
pub struct Config {
    #[serde(default)]
    pub long: LongConfig,
    #[serde(default)]
    pub when_not_tty: WhenNotTty,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct LongConfig {
    #[serde(default = "default_columns")]
    pub columns: Vec<Column>,
}

impl Default for LongConfig {
    fn default() -> Self {
        Self {
            columns: default_columns(),
        }
    }
}

fn default_columns() -> Vec<Column> {
    vec![
        Column::Permissions,
        Column::Links,
        Column::Owner,
        Column::Group,
        Column::Size,
        Column::Date,
        Column::Name,
    ]
}

#[derive(Deserialize, Serialize, Debug)]
pub struct WhenNotTty {
    #[serde(default = "default_true")]
    pub no_icons: bool,
    #[serde(default = "default_true")]
    pub no_color: bool,
    #[serde(default = "default_true")]
    pub one_per_line: bool,
}

impl Default for WhenNotTty {
    fn default() -> Self {
        Self {
            no_icons: true,
            no_color: true,
            one_per_line: true,
        }
    }
}

fn default_true() -> bool {
    true
}

impl Config {
    pub fn load() -> Self {
        let config_path = Self::get_config_path();

        Figment::new()
            .merge(Toml::file(config_path))
            .merge(Env::prefixed("ELX_"))
            .extract()
            .unwrap_or_default()
    }

    pub fn get_config_path() -> PathBuf {
        dirs::config_dir()
            .map(|d| d.join("elx").join("config.toml"))
            .unwrap_or_else(|| PathBuf::from("config.toml"))
    }
}
