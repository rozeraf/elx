use crate::display::Column;
use figment::{
    Figment,
    providers::{Env, Format, Toml},
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct HyperlinkConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_true")]
    pub files: bool,
    #[serde(default)]
    pub dirs: bool,
    #[serde(default = "default_true")]
    pub symlinks: bool,
    #[serde(default = "default_true")]
    pub executables: bool,
}

impl Default for HyperlinkConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            files: true,
            dirs: false,
            symlinks: true,
            executables: true,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SortKey {
    #[default]
    Name,
    NameNatural,
    Extension,
    Size,
    Modified,
    Created,
    GitStatus,
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum GroupDirs {
    #[default]
    First,
    Last,
    Mixed,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct SortConfig {
    #[serde(default)]
    pub by: SortKey,

    /// Without a secondary key, ties keep filesystem order.
    #[serde(default)]
    pub secondary_by: Option<SortKey>,

    #[serde(default = "default_true")]
    pub ignore_case: bool,

    /// Applies to the sort keys only, not to `group_dirs` or `dotfiles_first`.
    #[serde(default)]
    pub reverse: bool,

    #[serde(default)]
    pub group_dirs: GroupDirs,

    #[serde(default)]
    pub dotfiles_first: bool,
}

impl Default for SortConfig {
    fn default() -> Self {
        Self {
            by: SortKey::Name,
            secondary_by: None,
            ignore_case: true,
            reverse: false,
            group_dirs: GroupDirs::First,
            dotfiles_first: false,
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
    #[serde(default = "default_when_not_tty")]
    pub when_not_tty: ModeOverrides,
    #[serde(default)]
    pub sort: SortConfig,
    #[serde(default)]
    pub all: ModeOverrides,
    #[serde(default)]
    pub tree: ModeOverrides,
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
            when_not_tty: default_when_not_tty(),
            sort: SortConfig::default(),
            all: ModeOverrides::default(),
            tree: ModeOverrides::default(),
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, Copy, PartialEq, Default)]
#[serde(rename_all = "snake_case")]
pub enum PermissionsFormat {
    #[default]
    Symbolic,
    Octal,
    Both,
    None,
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
    #[serde(default)]
    pub permissions_format: PermissionsFormat,
}

impl Default for LongConfig {
    fn default() -> Self {
        Self {
            columns: default_columns(),
            headers: false,
            autohide_columns: false,
            permissions_format: PermissionsFormat::Symbolic,
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

/// Keys left unset inherit the base value. `ignore` replaces the base list instead of extending it,
/// so a mode can drop a pattern.
#[derive(Deserialize, Serialize, Debug, Default)]
#[serde(deny_unknown_fields)]
pub struct ModeOverrides {
    pub icons: Option<bool>,
    pub color: Option<bool>,
    pub classify: Option<bool>,
    pub git_ignore: Option<bool>,
    pub ignore: Option<Vec<String>>,
    pub one_per_line: Option<bool>,
}

fn default_when_not_tty() -> ModeOverrides {
    ModeOverrides {
        icons: Some(false),
        color: Some(false),
        one_per_line: Some(true),
        ..Default::default()
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
# Master switch: enable clickable OSC 8 hyperlinks in supported terminals
enabled = true
# Control which entry types get hyperlinks
files = true
dirs = false        # disabled: clicking a dir opens it in a file manager
symlinks = true
executables = true

[long]
# Display column headers in long listing format
headers = true

# Automatically hide columns that are empty for all shown entries (e.g., Git column if no changes)
autohide_columns = true

# Permissions display format: symbolic (drwxr-sr-x), octal (4755), both (drwxr-sr-x 4755), none
permissions_format = "symbolic"

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

[sort]
# Primary sort key:
#   name, name_natural, extension, size, modified, created, git_status, none
by = "name"

# Tiebreaker when primary values are equal (same keys available, or omit)
# secondary_by = "name"

# Case-insensitive comparison for name/extension keys
ignore_case = true

# Reverse sort order (applies to sort keys, not to dir/dotfile grouping)
reverse = false

# Directory grouping: first, last, mixed
group_dirs = "first"

# Show dotfiles (names starting with .) before regular entries in their group
dotfiles_first = false

# Per-mode overrides: keys set here replace the values above while the mode is active.
# Applied in order: [all] (-a), [tree] (-T), [when_not_tty] (output is not a terminal), then CLI flags.
# Supported keys: icons, color, classify, git_ignore, ignore (replaces the global list), one_per_line.
[all]
# classify = true

[tree]
# ignore = ["target", "node_modules"]

[when_not_tty]
# Output redirected to a file or pipe. Hyperlinks are always disabled here.
icons = false
color = false
one_per_line = true
"#
    }
}
