use serde::{Deserialize, Serialize};
use figment::{Figment, providers::{Format, Toml, Env}};
use std::path::PathBuf;
use crate::display::Column;

#[derive(Deserialize, Serialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Config {
    #[serde(default = "default_true")]
    pub icons: bool,
    #[serde(default = "default_true")]
    pub color: bool,
    #[serde(default = "default_true")]
    pub hyperlinks: bool,
    #[serde(default = "default_true")]
    pub git_ignore: bool,
    #[serde(default)]
    pub classify: bool,
    #[serde(default)]
    pub depth: Option<usize>,
    #[serde(default)]
    pub long: LongConfig,
    #[serde(default)]
    pub when_not_tty: WhenNotTty,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            icons: true,
            color: true,
            hyperlinks: true,
            git_ignore: true,
            classify: false,
            depth: None,
            long: LongConfig::default(),
            when_not_tty: WhenNotTty::default(),
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct LongConfig {
    #[serde(default = "default_columns")]
    pub columns: Vec<Column>,
    #[serde(default)]
    pub headers: bool,
    #[serde(default)]
    pub autohide_columns: bool,
}

impl Default for LongConfig {
    fn default() -> Self {
        Self {
            columns: default_columns(),
            headers: false,
            autohide_columns: false,
        }
    }
}

fn default_columns() -> Vec<Column> {
    vec![
        Column::Git,
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
#[serde(deny_unknown_fields)]
pub struct WhenNotTty {
    #[serde(default)]
    pub icons: bool,
    #[serde(default)]
    pub color: bool,
    #[serde(default)]
    pub hyperlinks: bool,
    #[serde(default = "default_true")]
    pub one_per_line: bool,
}

impl Default for WhenNotTty {
    fn default() -> Self {
        Self {
            icons: false,
            color: false,
            hyperlinks: false,
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
            .unwrap_or_else(|e| {
                eprintln!("elx: config error: {}", e);
                Config::default()
            })
    }

    pub fn get_config_path() -> PathBuf {
        dirs::config_dir()
            .map(|d| d.join("elx").join("config.toml"))
            .unwrap_or_else(|| PathBuf::from("config.toml"))
    }

    pub fn get_default_config_template() -> &'static str {
        r#"# elx configuration file

# Display icons next to file names (requires Nerd Font)
icons = true

# Use colors in output
color = true

# Enable clickable hyperlinks in supported terminals (OSC 8)
hyperlinks = true

# Respect .gitignore files
git_ignore = true

# Append indicator (one of /) to directories
classify = false

# Default recursion depth for tree view (uncomment to set)
# depth = 3

[long]
# Display column headers in long listing format
headers = true

# Automatically hide columns that are empty for all shown entries (e.g., Git column if no changes)
autohide_columns = true

# Columns to display in long listing format.
# Available columns: git, permissions, links, owner, group, size, date, name
columns = [
    "git",
    "permissions",
    "links",
    "owner",
    "group",
    "size",
    "date",
    "name",
]

[when_not_tty]
# Configuration for when output is redirected (e.g., to a file or pipe)
icons = false
color = false
hyperlinks = false
one_per_line = true
"#
    }
}
