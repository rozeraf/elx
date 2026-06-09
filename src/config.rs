use serde::{Deserialize, Serialize};
use figment::{Figment, providers::{Format, Toml, Env}};
use std::path::PathBuf;
use crate::display::Column;

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct HyperlinkConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_true")]
    pub files: bool,
    #[serde(default = "default_true")]
    pub directories: bool,
    #[serde(default = "default_true")]
    pub symlinks: bool,
    #[serde(default)]
    pub underline_files: bool,
    #[serde(default)]
    pub underline_directories: bool,
    #[serde(default)]
    pub underline_symlinks: bool,
    #[serde(default)]
    pub exclude_env: Vec<String>,
}

impl Default for HyperlinkConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            files: true,
            directories: true,
            symlinks: true,
            underline_files: false,
            underline_directories: false,
            underline_symlinks: false,
            exclude_env: Vec::new(),
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Config {
    #[serde(default = "default_true")]
    pub icons: bool,
    #[serde(default = "default_true")]
    pub color: bool,
    #[serde(default)]
    pub hyperlinks: HyperlinkConfig,
    #[serde(default = "default_true")]
    pub git_ignore: bool,
    #[serde(default)]
    pub ignore: Vec<String>,
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
            hyperlinks: HyperlinkConfig::default(),
            git_ignore: true,
            ignore: Vec::new(),
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

# Respect .gitignore files
git_ignore = true

# Global ignore patterns (glob)
# Example: ignore = ["*.json", "node_modules", "bin"]
ignore = []

# Append indicator (one of /) to directories
classify = false

# Default recursion depth for tree view (uncomment to set)
# depth = 3

[hyperlinks]
# Enable clickable hyperlinks in supported terminals (OSC 8)
enabled = true

# Enable hyperlinks for specific types
files = true
directories = true
symlinks = true

# Visually underline (ANSI underline) entries that are links
underline_files = false
underline_directories = false
underline_symlinks = false

# Disable hyperlinks if any of these environment variables are set
# Example: exclude_env = ["KITTY_WINDOW_ID"] to avoid issues in Kitty
exclude_env = []

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
