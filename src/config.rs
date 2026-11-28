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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize, Serialize, clap::ValueEnum)]
#[serde(rename_all = "snake_case")]
#[value(rename_all = "snake_case")]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize, Serialize, clap::ValueEnum)]
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
    /// Written by v0.1.x `--init-config`; accepted and ignored so those configs keep loading.
    #[serde(default, rename = "schema_version", skip_serializing)]
    pub _schema_version: Option<u32>,
    #[serde(default = "default_true")]
    pub icons: bool,
    #[serde(default = "default_true")]
    pub color: bool,
    #[serde(default)]
    pub hyperlinks: HyperlinkConfig,
    #[serde(default)]
    pub git_ignore: GitIgnoreMode,
    #[serde(default)]
    pub ignore: Vec<String>,
    #[serde(default)]
    pub collapse: Vec<String>,
    #[serde(default = "default_true")]
    pub show_hidden_summary: bool,
    #[serde(default)]
    pub classify: bool,
    #[serde(default)]
    pub depth: Option<std::num::NonZeroUsize>,
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
            _schema_version: None,
            icons: true,
            color: true,
            hyperlinks: HyperlinkConfig::default(),
            git_ignore: GitIgnoreMode::Hide,
            ignore: Vec::new(),
            collapse: Vec::new(),
            show_hidden_summary: true,
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

/// What to do with entries matched by `.gitignore`.
#[derive(Deserialize, Serialize, Debug, Clone, Copy, PartialEq, Eq, Default, clap::ValueEnum)]
#[serde(rename_all = "snake_case", from = "GitIgnoreValue")]
pub enum GitIgnoreMode {
    #[default]
    Hide,
    /// Show greyed out
    Dim,
    Show,
}

/// Also accepts the old boolean form: `true` = hide, `false` = show.
#[derive(Deserialize)]
#[serde(untagged)]
enum GitIgnoreValue {
    Bool(bool),
    Name(GitIgnoreName),
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
enum GitIgnoreName {
    Hide,
    Dim,
    Show,
}

impl From<GitIgnoreValue> for GitIgnoreMode {
    fn from(value: GitIgnoreValue) -> Self {
        match value {
            GitIgnoreValue::Bool(true) | GitIgnoreValue::Name(GitIgnoreName::Hide) => Self::Hide,
            GitIgnoreValue::Name(GitIgnoreName::Dim) => Self::Dim,
            GitIgnoreValue::Bool(false) | GitIgnoreValue::Name(GitIgnoreName::Show) => Self::Show,
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, Copy, PartialEq, Default, clap::ValueEnum)]
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
    #[serde(default = "default_true")]
    pub headers: bool,
    #[serde(default = "default_true")]
    pub autohide_columns: bool,
    #[serde(default)]
    pub permissions_format: PermissionsFormat,
    /// Replaces `[sort] group_dirs` in long view; `[tree] group_dirs` wins for `-lT`.
    #[serde(default)]
    pub group_dirs: Option<GroupDirs>,
}

impl Default for LongConfig {
    fn default() -> Self {
        Self {
            columns: default_columns(),
            headers: true,
            autohide_columns: true,
            permissions_format: PermissionsFormat::Symbolic,
            group_dirs: None,
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

/// Keys left unset inherit the base value. Lists (`ignore`, `collapse`) replace the base list
/// instead of extending it, so a mode can drop a pattern.
#[derive(Deserialize, Serialize, Debug, Default)]
#[serde(deny_unknown_fields)]
pub struct ModeOverrides {
    pub icons: Option<bool>,
    pub color: Option<bool>,
    pub classify: Option<bool>,
    pub git_ignore: Option<GitIgnoreMode>,
    pub ignore: Option<Vec<String>>,
    pub collapse: Option<Vec<String>>,
    pub one_per_line: Option<bool>,
    pub group_dirs: Option<GroupDirs>,
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

# Entries matched by .gitignore: "hide", "dim" (shown greyed out) or "show"
git_ignore = "hide"

# Global ignore patterns (gitignore syntax; "!pattern" re-includes, the last match wins)
# Example: ignore = ["*.json", "node_modules", "bin"]
ignore = []

# Append indicator (one of /) to directories
classify = false

# Directories shown in tree view without descending into them (gitignore-style patterns)
# Example: collapse = ["target", "node_modules", ".git"]
collapse = []

# Print how many entries were hidden and why below the listing
# (on stderr when the output is piped; never when neither stdout nor stderr is a terminal)
show_hidden_summary = true

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

# Directory grouping in long view (first, last, mixed); unset uses [sort] group_dirs
# group_dirs = "mixed"

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
# Supported keys: icons, color, classify, git_ignore, ignore, collapse, one_per_line, group_dirs.
# Lists (ignore, collapse) replace the global list.
[all]
# classify = true

[tree]
# git_ignore = "dim"
# collapse = ["target", "node_modules", ".git"]
# group_dirs = "first"

[when_not_tty]
# Output redirected to a file or pipe. Hyperlinks are always disabled here.
icons = false
color = false
one_per_line = true
"#
    }
}

#[cfg(test)]
mod tests {
    use super::{Config, GitIgnoreMode};
    use figment::{
        Figment,
        providers::{Format, Toml},
    };

    fn parse(toml: &str) -> Config {
        Figment::from(Toml::string(toml)).extract().unwrap()
    }

    #[test]
    fn git_ignore_accepts_modes_and_old_booleans() {
        for (value, mode) in [
            ("\"hide\"", GitIgnoreMode::Hide),
            ("\"dim\"", GitIgnoreMode::Dim),
            ("\"show\"", GitIgnoreMode::Show),
            ("true", GitIgnoreMode::Hide),
            ("false", GitIgnoreMode::Show),
        ] {
            assert_eq!(parse(&format!("git_ignore = {value}")).git_ignore, mode);
        }
        assert_eq!(
            parse("[tree]\ngit_ignore = \"dim\"").tree.git_ignore,
            Some(GitIgnoreMode::Dim)
        );
    }

    #[test]
    fn v0_1_configs_with_schema_version_still_load() {
        let config = parse("schema_version = 3\nicons = false\n");
        assert!(!config.icons);
    }

    #[test]
    fn depth_zero_is_rejected() {
        let result: Result<Config, _> = Figment::from(Toml::string("depth = 0")).extract();
        assert!(result.is_err());
        assert_eq!(parse("depth = 2").depth.map(|d| d.get()), Some(2));
    }

    #[test]
    fn default_template_parses() {
        parse(Config::get_default_config_template());
    }
}
