use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "elx")]
#[command(about = "A personal ls-clone in Rust with Git integration and Tree view", long_about = None)]
pub struct Cli {
    /// Path to the directory or file
    #[arg(default_value = ".")]
    pub path: PathBuf,

    /// Use long listing format (includes Git status)
    #[arg(short = 'l', long)]
    pub long: bool,

    /// Show hidden files
    #[arg(short = 'a', long)]
    pub all: bool,

    /// Display output as a tree
    #[arg(short = 'T', long)]
    pub tree: bool,

    /// Maximum recursion depth for tree view
    #[arg(long)]
    pub depth: Option<usize>,

    /// Do not display icons
    #[arg(long, overrides_with = "icons")]
    pub no_icons: bool,

    /// Display icons (overrides config)
    #[arg(long, overrides_with = "no_icons", hide = true)]
    pub icons: bool,

    /// Do not use colors in output
    #[arg(long, overrides_with = "color")]
    pub no_color: bool,

    /// Use colors in output (overrides config)
    #[arg(long, overrides_with = "no_color", hide = true)]
    pub color: bool,

    /// Do not use hyperlinks in output
    #[arg(long, overrides_with = "hyperlinks")]
    pub no_hyperlinks: bool,

    /// Use hyperlinks in output (overrides config)
    #[arg(long, overrides_with = "no_hyperlinks", hide = true)]
    pub hyperlinks: bool,

    /// Append indicator (one of /) to entries
    #[arg(short = 'F', long)]
    pub classify: bool,

    /// Do not respect .gitignore files
    #[arg(short = 'G', long, visible_alias = "nogit")]
    pub no_git_ignore: bool,

    /// Display one entry per line
    #[arg(short = '1', long = "one-per-line")]
    pub one_per_line: bool,

    /// Ignore patterns (glob), e.g., "*.json,node_modules,bin"
    #[arg(long, value_delimiter = ',')]
    pub ignore: Vec<String>,

    /// Show permissions in octal format (e.g. 4755). Overrides config.
    #[arg(long, short = 'o')]
    pub octal: bool,

    /// Sort by key: name, name_natural, extension, size, modified, created, git_status, none
    #[arg(long, value_name = "KEY")]
    pub sort: Option<String>,

    /// Sort by modification time, newest first (shorthand: --sort=modified --reverse)
    #[arg(short = 't', conflicts_with = "sort")]
    pub sort_time: bool,

    /// Sort by size, largest first (shorthand: --sort=size --reverse)
    #[arg(short = 'S', conflicts_with = "sort")]
    pub sort_size: bool,

    /// Sort by file extension (shorthand: --sort=extension)
    #[arg(short = 'X', conflicts_with = "sort")]
    pub sort_extension: bool,

    /// Natural sort of version numbers (shorthand: --sort=name_natural)
    #[arg(short = 'v', conflicts_with = "sort")]
    pub sort_natural: bool,

    /// No sorting, filesystem order (shorthand: --sort=none)
    #[arg(short = 'U', conflicts_with = "sort")]
    pub sort_none: bool,

    /// Reverse sort order
    #[arg(short = 'r', long)]
    pub reverse: bool,

    /// List directories before files (overrides config)
    #[arg(long, conflicts_with_all = ["dirs_last", "dirs_mixed"])]
    pub dirs_first: bool,

    /// List directories after files (overrides config)
    #[arg(long, conflicts_with_all = ["dirs_first", "dirs_mixed"])]
    pub dirs_last: bool,

    /// Do not group directories (overrides config)
    #[arg(long, conflicts_with_all = ["dirs_first", "dirs_last"])]
    pub dirs_mixed: bool,

    /// Show dotfiles before regular entries within their group
    #[arg(long)]
    pub dotfiles_first: bool,

    /// Case-sensitive name sorting (overrides config ignore_case = true)
    #[arg(long)]
    pub case_sensitive: bool,

    /// Generate a default configuration file
    #[arg(long)]
    pub init_config: bool,
}

#[cfg(test)]
mod tests {
    use super::Cli;
    use clap::Parser;

    #[test]
    fn git_ignore_can_be_disabled_with_all_supported_flags() {
        for flag in ["-G", "--nogit", "--no-git-ignore"] {
            let cli = Cli::try_parse_from(["elx", flag]).unwrap();
            assert!(
                cli.no_git_ignore,
                "flag {flag} should disable git-ignore filtering"
            );
        }
    }
}

impl Cli {
    pub fn icons_overridden(&self) -> Option<bool> {
        if self.icons { Some(true) }
        else if self.no_icons { Some(false) }
        else { None }
    }

    pub fn color_overridden(&self) -> Option<bool> {
        if self.color { Some(true) }
        else if self.no_color { Some(false) }
        else { None }
    }

    pub fn hyperlinks_overridden(&self) -> Option<bool> {
        if self.hyperlinks { Some(true) }
        else if self.no_hyperlinks { Some(false) }
        else { None }
    }

    /// Возвращает (ключ, нужно ли форсировать reverse из шортхенда).
    /// Шортхенды -t и -S сами устанавливают reverse=true.
    pub fn resolve_sort_key(&self) -> (Option<crate::config::SortKey>, bool) {
        use crate::config::SortKey;
        if self.sort_time      { return (Some(SortKey::Modified), true); }
        if self.sort_size      { return (Some(SortKey::Size), true); }
        if self.sort_extension { return (Some(SortKey::Extension), false); }
        if self.sort_natural   { return (Some(SortKey::NameNatural), false); }
        if self.sort_none      { return (Some(SortKey::None), false); }
        if let Some(ref s) = self.sort {
            let key = match s.as_str() {
                "name"         => SortKey::Name,
                "name_natural" => SortKey::NameNatural,
                "extension"    => SortKey::Extension,
                "size"         => SortKey::Size,
                "modified"     => SortKey::Modified,
                "created"      => SortKey::Created,
                "git_status"   => SortKey::GitStatus,
                "none"         => SortKey::None,
                unknown => {
                    eprintln!("elx: unknown sort key '{}', falling back to 'name'", unknown);
                    SortKey::Name
                }
            };
            return (Some(key), false);
        }
        (None, false)
    }
}
