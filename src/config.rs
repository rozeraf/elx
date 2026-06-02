use serde::Deserialize;
use figment::{Figment, providers::{Format, Toml, Env}};
use std::path::PathBuf;

#[derive(Deserialize, Debug, Default)]
pub struct Config {
    #[serde(default)]
    pub when_not_tty: WhenNotTty,
}

#[derive(Deserialize, Debug)]
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
        let config_path = dirs::config_dir()
            .map(|d| d.join("elx").join("config.toml"))
            .unwrap_or_else(|| PathBuf::from("config.toml"));

        Figment::new()
            .merge(Toml::file(config_path))
            .merge(Env::prefixed("ELX_"))
            .extract()
            .unwrap_or_default()
    }
}
