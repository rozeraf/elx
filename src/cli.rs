use crate::config::{GitIgnoreMode, SortKey};
use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "elx", version)]
#[command(about = "A personal ls-clone in Rust with Git integration and Tree view", long_about = None)]
pub struct Cli {
    /// Directories or files to list
    // OsString, not PathBuf: clap rejects an empty PathBuf, while ls reports `ls ''` as missing.
    #[arg(default_value = ".", value_name = "PATH")]
    pub paths: Vec<std::ffi::OsString>,

    /// Use long listing format (includes Git status)
    #[arg(short = 'l', long)]
    pub long: bool,

    /// Show hidden files; repeat (-aa) to disable all filtering like -u
    #[arg(short = 'a', long, action = clap::ArgAction::Count)]
    pub all: u8,

    /// Disable all filtering: hidden files, .gitignore, ignore files and config ignore patterns
    #[arg(short = 'u', long)]
    pub unrestricted: bool,

    /// Display output as a tree
    #[arg(short = 'T', long)]
    pub tree: bool,

    /// Maximum recursion depth for tree view, at least 1 (implies --tree)
    #[arg(short = 'L', long, value_parser = parse_depth)]
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

    /// Append indicator (one of /@*|=) to entries, like ls -F
    #[arg(short = 'F', long)]
    pub classify: bool,

    /// Do not respect .gitignore files (same as --git-ignore=show)
    #[arg(
        short = 'G',
        long,
        visible_alias = "nogit",
        conflicts_with = "git_ignore"
    )]
    pub no_git_ignore: bool,

    /// What to do with entries matched by .gitignore (overrides config)
    #[arg(long, value_name = "MODE", value_enum)]
    pub git_ignore: Option<GitIgnoreMode>,

    /// Display one entry per line
    #[arg(short = '1', long = "one-per-line")]
    pub one_per_line: bool,

    /// Show only files matching these patterns (gitignore syntax); with -D, match directories instead
    #[arg(long, value_delimiter = ',', value_name = "GLOBS")]
    pub only: Vec<String>,

    /// Show only directories
    #[arg(short = 'D', long, conflicts_with = "files_only")]
    pub dirs_only: bool,

    /// Show only non-directories (in tree view, directories that contain them stay)
    #[arg(long)]
    pub files_only: bool,

    /// Show only entries with git changes (U, M, A, D, ?) and the directories containing them.
    /// Deleted files are gone from disk, so only their directories show up
    #[arg(long)]
    pub changed: bool,

    /// Do not apply ignore patterns from the config file (--ignore still applies)
    #[arg(long)]
    pub no_config_ignore: bool,

    /// Ignore patterns (gitignore syntax, later wins, !pattern re-includes), e.g. "*.json,node_modules"
    #[arg(long, value_delimiter = ',')]
    pub ignore: Vec<String>,

    /// Show these directories in tree view without descending into them (glob), e.g. "target,node_modules"
    #[arg(long, value_delimiter = ',', conflicts_with = "no_collapse")]
    pub collapse: Vec<String>,

    /// Expand every directory in tree view, ignoring configured collapse patterns
    #[arg(long)]
    pub no_collapse: bool,

    /// Show permissions in octal format (e.g. 4755). Overrides config.
    #[arg(long, short = 'o')]
    pub octal: bool,

    /// Sort by key
    #[arg(long, value_name = "KEY", value_enum, overrides_with_all = ["sort_time", "sort_size", "sort_extension", "sort_natural", "sort_none"])]
    pub sort: Option<SortKey>,

    /// Sort by modification time, newest first (shorthand: --sort=modified --reverse)
    #[arg(short = 't', overrides_with_all = ["sort", "sort_size", "sort_extension", "sort_natural", "sort_none"])]
    pub sort_time: bool,

    /// Sort by size, largest first (shorthand: --sort=size --reverse)
    #[arg(short = 'S', overrides_with_all = ["sort", "sort_time", "sort_extension", "sort_natural", "sort_none"])]
    pub sort_size: bool,

    /// Sort by file extension (shorthand: --sort=extension)
    #[arg(short = 'X', overrides_with_all = ["sort", "sort_time", "sort_size", "sort_natural", "sort_none"])]
    pub sort_extension: bool,

    /// Natural sort of version numbers (shorthand: --sort=name_natural)
    #[arg(short = 'v', overrides_with_all = ["sort", "sort_time", "sort_size", "sort_extension", "sort_none"])]
    pub sort_natural: bool,

    /// No sorting, filesystem order (shorthand: --sort=none)
    #[arg(short = 'U', overrides_with_all = ["sort", "sort_time", "sort_size", "sort_extension", "sort_natural"])]
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

    /// Print a shell completion script (bash, zsh, fish, elvish, powershell) and exit
    #[arg(long, value_name = "SHELL", hide = true)]
    pub completions: Option<clap_complete::Shell>,

    /// Print the settings in effect for this invocation (config, sections and flags merged) and exit
    #[arg(long)]
    pub print_config: bool,
}

fn parse_depth(value: &str) -> Result<usize, String> {
    match value.parse::<usize>() {
        Ok(0) => Err("must be at least 1".to_string()),
        Ok(depth) => Ok(depth),
        Err(err) => Err(err.to_string()),
    }
}

impl Cli {
    pub fn unrestricted(&self) -> bool {
        self.unrestricted || self.all >= 2
    }

    pub fn show_hidden(&self) -> bool {
        self.all > 0 || self.unrestricted()
    }

    pub fn icons_overridden(&self) -> Option<bool> {
        if self.icons {
            Some(true)
        } else if self.no_icons {
            Some(false)
        } else {
            None
        }
    }

    pub fn color_overridden(&self) -> Option<bool> {
        if self.color {
            Some(true)
        } else if self.no_color {
            Some(false)
        } else {
            None
        }
    }

    pub fn hyperlinks_overridden(&self) -> Option<bool> {
        if self.hyperlinks {
            Some(true)
        } else if self.no_hyperlinks {
            Some(false)
        } else {
            None
        }
    }

    /// Returns the sort key and whether it forces reverse (`-t`, `-S` list newest/largest first).
    pub fn resolve_sort_key(&self) -> (Option<SortKey>, bool) {
        if self.sort_time {
            return (Some(SortKey::Modified), true);
        }
        if self.sort_size {
            return (Some(SortKey::Size), true);
        }
        if self.sort_extension {
            return (Some(SortKey::Extension), false);
        }
        if self.sort_natural {
            return (Some(SortKey::NameNatural), false);
        }
        if self.sort_none {
            return (Some(SortKey::None), false);
        }
        if let Some(key) = self.sort {
            return (Some(key), false);
        }
        (None, false)
    }
}

#[cfg(test)]
mod tests {
    use super::Cli;
    use clap::Parser;

    #[test]
    fn last_sort_option_wins_like_ls() {
        let key = |args: &[&str]| {
            Cli::try_parse_from(std::iter::once("elx").chain(args.iter().copied()))
                .unwrap()
                .resolve_sort_key()
        };
        use super::SortKey;
        assert_eq!(key(&["-t", "-S"]), (Some(SortKey::Size), true));
        assert_eq!(key(&["-S", "-t"]), (Some(SortKey::Modified), true));
        assert_eq!(key(&["-t", "--sort", "name"]), (Some(SortKey::Name), false));
        assert_eq!(
            key(&["--sort", "size", "-X"]),
            (Some(SortKey::Extension), false)
        );
    }

    #[test]
    fn depth_must_be_positive() {
        assert!(Cli::try_parse_from(["elx", "-L", "0"]).is_err());
        assert_eq!(
            Cli::try_parse_from(["elx", "-L", "2"]).unwrap().depth,
            Some(2)
        );
    }

    #[test]
    fn sort_rejects_unknown_keys_and_accepts_snake_case() {
        assert!(Cli::try_parse_from(["elx", "--sort", "bogus"]).is_err());
        let cli = Cli::try_parse_from(["elx", "--sort", "name_natural"]).unwrap();
        assert_eq!(cli.sort, Some(super::SortKey::NameNatural));
    }

    #[test]
    fn git_ignore_mode_flag_parses_and_conflicts_with_g() {
        let cli = Cli::try_parse_from(["elx", "--git-ignore", "dim"]).unwrap();
        assert_eq!(cli.git_ignore, Some(super::GitIgnoreMode::Dim));
        assert!(Cli::try_parse_from(["elx", "-G", "--git-ignore", "hide"]).is_err());
    }

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
