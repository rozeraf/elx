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
    /// Global switch to enable/disable hyperlinks
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// Hyperlinks for regular files
    #[serde(default = "default_true")]
    pub files: bool,
    /// Hyperlinks for directories (default false)
    #[serde(default)]
    pub dirs: bool,
    /// Hyperlinks for symlinks
    #[serde(default = "default_true")]
    pub symlinks: bool,
    /// Hyperlinks for executable files
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
    Name,           // case-insensitive алфавит (или с учётом, см. ignore_case)
    NameNatural,    // натуральная: file2 < file10
    Extension,      // по расширению, тайбрейкер — name
    Size,           // metadata.len(), директории → 0
    Modified,       // metadata.modified()
    Created,        // metadata.created()
    GitStatus,      // M > A > D > ? > ! > чистый
    None,           // порядок ФС, без сортировки
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum GroupDirs {
    #[default]
    First,   // директории всегда перед файлами
    Last,    // директории всегда после файлов
    Mixed,   // не группировать — вместе с файлами по основному ключу
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct SortConfig {
    /// Первичный ключ сортировки
    #[serde(default)]
    pub by: SortKey,

    /// Тайбрейкер при равных значениях первичного ключа
    /// По умолчанию None: при равных значениях — порядок ФС
    #[serde(default)]
    pub secondary_by: Option<SortKey>,

    /// Регистронезависимое сравнение имён и расширений
    #[serde(default = "default_true")]
    pub ignore_case: bool,

    /// Обратный порядок (применяется к первичному и вторичному ключам,
    /// но НЕ к group_dirs и dotfiles_first)
    #[serde(default)]
    pub reverse: bool,

    /// Группировка директорий: first, last, mixed
    #[serde(default)]
    pub group_dirs: GroupDirs,

    /// Показывать скрытые файлы (начинающиеся с .) первыми внутри группы
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
    #[serde(default)]
    pub when_not_tty: WhenNotTty,
    #[serde(default)]
    pub sort: SortConfig,
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
            sort: SortConfig::default(),
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, Copy, PartialEq, Default)]
#[serde(rename_all = "snake_case")]
pub enum PermissionsFormat {
    #[default]
    Symbolic,   // drwxr-sr-x
    Octal,      // 4755
    Both,       // drwxr-sr-x 4755
    None,       // Hide permissions
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

#[derive(Deserialize, Serialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct WhenNotTty {
    #[serde(default)]
    pub icons: bool,
    #[serde(default)]
    pub color: bool,
    #[serde(default = "default_true")]
    pub one_per_line: bool,
}

impl Default for WhenNotTty {
    fn default() -> Self {
        Self {
            icons: false,
            color: false,
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

[when_not_tty]
# Configuration for when output is redirected (e.g., to a file or pipe)
icons = false
color = false
one_per_line = true
"#
    }
}
