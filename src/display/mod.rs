pub mod formatter;
pub mod grid;
pub mod long;
pub mod tree;

pub use formatter::{Cell, Column, ColumnFormatter};

use crate::cli::Cli;
use crate::config::{Config, GitIgnoreMode, HyperlinkConfig, ModeOverrides, PermissionsFormat};
use std::io::{IsTerminal, stdout};
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
    pub one_per_line: bool,
    pub long_view: bool,
    pub depth: Option<usize>,
    pub ignore_globs: Vec<String>,
    pub permissions_format: PermissionsFormat,
}

impl DisplayOptions {
    pub fn new(config: &Config, cli: &Cli) -> Self {
        Self::build(config, cli, stdout().is_terminal())
    }

    fn build(config: &Config, cli: &Cli, is_tty: bool) -> Self {
        // Layers: config, [all], [tree], [when_not_tty], then explicit CLI flags (which always win).
        let mut icons = config.icons;
        let mut color = config.color;
        let mut hyperlinks = config.hyperlinks.clone();
        let mut classify = config.classify;
        let mut git_ignore = config.git_ignore;
        let mut depth = config.depth;
        let mut one_per_line = false;
        let long_view = cli.long;
        let mut ignore_globs = config.ignore.clone();

        let active_modes: [(bool, &ModeOverrides); 3] = [
            (cli.all, &config.all),
            (cli.tree, &config.tree),
            (!is_tty, &config.when_not_tty),
        ];
        for (_, mode) in active_modes.into_iter().filter(|(on, _)| *on) {
            icons = mode.icons.unwrap_or(icons);
            color = mode.color.unwrap_or(color);
            classify = mode.classify.unwrap_or(classify);
            git_ignore = mode.git_ignore.unwrap_or(git_ignore);
            one_per_line = mode.one_per_line.unwrap_or(one_per_line);
            if let Some(globs) = &mode.ignore {
                ignore_globs = globs.clone();
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
        if cli.no_git_ignore {
            git_ignore = GitIgnoreMode::Show;
        }
        if cli.one_per_line {
            one_per_line = true;
        }
        if cli.depth.is_some() {
            depth = cli.depth;
        }

        if !cli.ignore.is_empty() {
            ignore_globs.extend(cli.ignore.clone());
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
            one_per_line,
            long_view,
            depth,
            ignore_globs,
            permissions_format,
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
    fn cli_flags_override_modes() {
        let o = opts(&["-a", "--color", "--ignore", "*.tmp"], true);
        assert!(o.color);
        assert_eq!(o.ignore_globs, ["*.log", "*.tmp"]);
    }
}
