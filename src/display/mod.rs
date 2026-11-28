pub mod formatter;
pub mod grid;
pub mod long;
pub mod tree;

pub use formatter::{Cell, Column, ColumnFormatter};

use crate::cli::Cli;
use crate::config::{
    Config, GitIgnoreMode, GroupDirs, HyperlinkConfig, ModeOverrides, PermissionsFormat,
};
use std::io::{IsTerminal, stderr, stdout};
use terminal_size::{Width, terminal_size};

pub fn get_terminal_width() -> usize {
    if let Some((Width(w), _)) = terminal_size() {
        w as usize
    } else {
        80
    }
}

#[derive(Debug, Clone)]
pub struct DisplayOptions {
    pub icons: bool,
    pub color: bool,
    pub hyperlinks: HyperlinkConfig,
    pub classify: bool,
    pub git_ignore: GitIgnoreMode,
    pub show_hidden: bool,
    pub ignore_files: bool,
    pub one_per_line: bool,
    pub long_view: bool,
    pub depth: Option<usize>,
    pub ignore_globs: Vec<String>,
    pub collapse_globs: Vec<String>,
    pub hidden_summary: bool,
    pub permissions_format: PermissionsFormat,
    pub group_dirs: GroupDirs,
}

impl DisplayOptions {
    pub fn new(config: &Config, cli: &Cli) -> Self {
        let mut options = Self::build(config, cli, stdout().is_terminal());
        // Piped output still says what it left out, on stderr, as long as someone is watching it.
        if config.show_hidden_summary && !stdout().is_terminal() && stderr().is_terminal() {
            options.hidden_summary = true;
        }
        options
    }

    fn build(config: &Config, cli: &Cli, is_tty: bool) -> Self {
        // Layers: config, [long], [all], [tree], [when_not_tty], then explicit CLI flags (which always win).
        let mut icons = config.icons;
        let mut color = config.color;
        let mut hyperlinks = config.hyperlinks.clone();
        let mut classify = config.classify;
        let mut git_ignore = config.git_ignore;
        let mut depth = config.depth.map(std::num::NonZeroUsize::get);
        let mut one_per_line = false;
        let long_view = cli.long;
        let mut ignore_globs = config.ignore.clone();
        let mut collapse_globs = config.collapse.clone();
        let mut group_dirs = config.sort.group_dirs;

        let long_mode = config.long.mode();
        let active_modes: [(bool, &ModeOverrides); 4] = [
            (cli.long, &long_mode),
            (cli.show_hidden(), &config.all),
            (cli.tree, &config.tree),
            (!is_tty, &config.when_not_tty),
        ];
        for (_, mode) in active_modes.into_iter().filter(|(on, _)| *on) {
            icons = mode.icons.unwrap_or(icons);
            color = mode.color.unwrap_or(color);
            classify = mode.classify.unwrap_or(classify);
            git_ignore = mode.git_ignore.unwrap_or(git_ignore);
            one_per_line = mode.one_per_line.unwrap_or(one_per_line);
            group_dirs = mode.group_dirs.unwrap_or(group_dirs);
            if let Some(globs) = &mode.ignore {
                ignore_globs = globs.clone();
            }
            if let Some(globs) = &mode.collapse {
                collapse_globs = globs.clone();
            }
        }

        if !is_tty {
            hyperlinks.enabled = false;
        }

        if let Some(i) = cli.icons_overridden() {
            icons = i;
        }
        if let Some(c) = cli.color_overridden() {
            color = c;
        }
        if let Some(h) = cli.hyperlinks_overridden() {
            hyperlinks.enabled = h;
        }
        if cli.classify {
            classify = true;
        }
        let unrestricted = cli.unrestricted();
        if unrestricted {
            git_ignore = GitIgnoreMode::Show;
        }
        if cli.no_git_ignore {
            git_ignore = GitIgnoreMode::Show;
        }
        if let Some(mode) = cli.git_ignore {
            git_ignore = mode;
        }
        if cli.one_per_line {
            one_per_line = true;
        }
        if cli.depth.is_some() {
            depth = cli.depth;
        }

        if cli.no_config_ignore || unrestricted {
            ignore_globs.clear();
        }
        ignore_globs.extend(cli.ignore.clone());
        if cli.no_collapse {
            collapse_globs.clear();
        }
        collapse_globs.extend(cli.collapse.clone());

        if cli.dirs_first {
            group_dirs = GroupDirs::First;
        } else if cli.dirs_last {
            group_dirs = GroupDirs::Last;
        } else if cli.dirs_mixed {
            group_dirs = GroupDirs::Mixed;
        }

        let mut permissions_format = config.long.permissions_format;
        if cli.octal {
            permissions_format = PermissionsFormat::Octal;
        }

        Self {
            icons,
            color,
            hyperlinks,
            classify,
            git_ignore,
            show_hidden: cli.show_hidden(),
            ignore_files: !unrestricted,
            one_per_line,
            long_view,
            depth,
            ignore_globs,
            collapse_globs,
            hidden_summary: config.show_hidden_summary && is_tty,
            permissions_format,
            group_dirs,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::DisplayOptions;
    use crate::cli::Cli;
    use crate::config::{Config, GitIgnoreMode, ModeOverrides};
    use clap::Parser;

    fn config() -> Config {
        Config {
            classify: false,
            ignore: vec!["*.log".into()],
            all: ModeOverrides {
                classify: Some(true),
                color: Some(false),
                ..Default::default()
            },
            tree: ModeOverrides {
                color: Some(true),
                git_ignore: Some(GitIgnoreMode::Dim),
                ignore: Some(vec!["target".into()]),
                ..Default::default()
            },
            ..Default::default()
        }
    }

    fn opts(args: &[&str], is_tty: bool) -> DisplayOptions {
        let cli = Cli::try_parse_from(std::iter::once("elx").chain(args.iter().copied())).unwrap();
        DisplayOptions::build(&config(), &cli, is_tty)
    }

    #[test]
    fn no_mode_uses_base_config() {
        let o = opts(&[], true);
        assert!(!o.classify && o.color);
        assert_eq!(o.git_ignore, GitIgnoreMode::Hide);
        assert_eq!(o.ignore_globs, ["*.log"]);
    }

    #[test]
    fn all_mode_overrides_only_its_keys() {
        let o = opts(&["-a"], true);
        assert!(o.classify && !o.color);
        assert_eq!(o.git_ignore, GitIgnoreMode::Hide);
        assert_eq!(o.ignore_globs, ["*.log"]);
    }

    #[test]
    fn tree_mode_replaces_ignore_list() {
        let o = opts(&["-T"], true);
        assert_eq!(o.git_ignore, GitIgnoreMode::Dim);
        assert_eq!(o.ignore_globs, ["target"]);
    }

    #[test]
    fn tree_applies_after_all() {
        let o = opts(&["-aT"], true);
        assert!(o.classify && o.color);
        assert_eq!(o.git_ignore, GitIgnoreMode::Dim);
    }

    #[test]
    fn non_tty_overrides_modes() {
        let o = opts(&["-T"], false);
        assert!(!o.color);
    }

    #[test]
    fn non_tty_defaults_apply_without_section() {
        let cli = Cli::try_parse_from(["elx"]).unwrap();
        let o = DisplayOptions::build(&Config::default(), &cli, false);
        assert!(!o.icons && !o.color && !o.hyperlinks.enabled && o.one_per_line);
    }

    #[test]
    fn non_tty_section_inherits_unset_keys() {
        let mut cfg = config();
        cfg.when_not_tty = ModeOverrides {
            icons: Some(false),
            ..Default::default()
        };
        let cli = Cli::try_parse_from(["elx"]).unwrap();
        let o = DisplayOptions::build(&cfg, &cli, false);
        assert!(!o.icons && o.color && !o.one_per_line);
    }

    #[test]
    fn no_git_ignore_flag_overrides_dim() {
        assert_eq!(opts(&["-T", "-G"], true).git_ignore, GitIgnoreMode::Show);
    }

    #[test]
    fn collapse_follows_modes_and_cli() {
        let mut cfg = config();
        cfg.collapse = vec![".git".into()];
        cfg.tree.collapse = Some(vec!["target".into()]);
        let build = |args: &[&str]| {
            let cli =
                Cli::try_parse_from(std::iter::once("elx").chain(args.iter().copied())).unwrap();
            DisplayOptions::build(&cfg, &cli, true).collapse_globs
        };
        assert_eq!(build(&[]), [".git"]);
        assert_eq!(build(&["-T"]), ["target"]);
        assert_eq!(build(&["-T", "--collapse", "dist"]), ["target", "dist"]);
        assert!(build(&["-T", "--no-collapse"]).is_empty());
    }

    #[test]
    fn unrestricted_disables_all_filters_but_keeps_explicit_flags() {
        for args in [&["-u"][..], &["-aa"], &["--all", "--all"]] {
            let o = opts(args, true);
            assert!(o.show_hidden && !o.ignore_files, "{args:?}");
            assert_eq!(o.git_ignore, GitIgnoreMode::Show);
            assert!(o.ignore_globs.is_empty());
            assert!(o.classify, "[all] applies with {args:?}");
        }
        let o = opts(&["-u", "--git-ignore", "dim", "--ignore", "*.tmp"], true);
        assert_eq!(o.git_ignore, GitIgnoreMode::Dim);
        assert_eq!(o.ignore_globs, ["*.tmp"]);
        let o = opts(&["-a"], true);
        assert!(o.show_hidden && o.ignore_files);
        assert_eq!(o.git_ignore, GitIgnoreMode::Hide);
    }

    #[test]
    fn no_config_ignore_keeps_cli_patterns() {
        let o = opts(&["--no-config-ignore", "--ignore", "*.tmp"], true);
        assert_eq!(o.ignore_globs, ["*.tmp"]);
        assert!(o.ignore_files);
    }

    #[test]
    fn cli_flags_override_modes() {
        let o = opts(&["-a", "--color", "--ignore", "*.tmp"], true);
        assert!(o.color);
        assert_eq!(o.ignore_globs, ["*.log", "*.tmp"]);
    }

    #[test]
    fn group_dirs_per_view() {
        use crate::config::GroupDirs;
        let mut cfg = config();
        cfg.sort.group_dirs = GroupDirs::First;
        cfg.long.group_dirs = Some(GroupDirs::Mixed);
        cfg.tree.group_dirs = Some(GroupDirs::Last);
        let build = |args: &[&str]| {
            let cli =
                Cli::try_parse_from(std::iter::once("elx").chain(args.iter().copied())).unwrap();
            DisplayOptions::build(&cfg, &cli, true).group_dirs
        };
        assert_eq!(build(&[]), GroupDirs::First);
        assert_eq!(build(&["-l"]), GroupDirs::Mixed);
        assert_eq!(build(&["-T"]), GroupDirs::Last);
        assert_eq!(build(&["-lT"]), GroupDirs::Last);
        assert_eq!(build(&["-lT", "--dirs-first"]), GroupDirs::First);
        assert_eq!(build(&["-l", "--dirs-last"]), GroupDirs::Last);
    }

    #[test]
    fn long_section_is_a_mode_section() {
        let mut cfg = config();
        cfg.long.classify = Some(true);
        cfg.long.color = Some(false);
        let cli = Cli::try_parse_from(["elx", "-l"]).unwrap();
        let o = DisplayOptions::build(&cfg, &cli, true);
        assert!(o.classify && !o.color);
        let cli = Cli::try_parse_from(["elx", "-la"]).unwrap();
        assert!(DisplayOptions::build(&cfg, &cli, true).classify);
        let cli = Cli::try_parse_from(["elx"]).unwrap();
        assert!(!DisplayOptions::build(&cfg, &cli, true).classify);
    }
}
