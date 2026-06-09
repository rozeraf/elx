pub mod grid;
pub mod long;
pub mod tree;
pub mod formatter;

pub use formatter::{Column, Cell, ColumnFormatter};

use terminal_size::{terminal_size, Width};
use crate::config::{Config, HyperlinkConfig};
use crate::cli::Cli;
use std::io::{stdout, IsTerminal};

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
    pub git_ignore: bool,
    pub one_per_line: bool,
    pub long_view: bool,
    pub depth: Option<usize>,
}

impl DisplayOptions {
    pub fn new(config: &Config, cli: &Cli) -> Self {
        let is_tty = stdout().is_terminal();
        
        // 1. Start with global config
        let mut icons = config.icons;
        let mut color = config.color;
        let mut hyperlinks = config.hyperlinks.clone();
        let mut classify = config.classify;
        let mut git_ignore = config.git_ignore;
        let mut depth = config.depth;
        let mut one_per_line = false;
        let long_view = cli.long;

        // 2. Apply TTY overrides if not a terminal
        if !is_tty {
            icons = config.when_not_tty.icons;
            color = config.when_not_tty.color;
            hyperlinks.enabled = config.when_not_tty.hyperlinks;
            one_per_line = config.when_not_tty.one_per_line;
        }

        // 3. Exclude by environment variables
        for env_var in &hyperlinks.exclude_env {
            if std::env::var_os(env_var).is_some() {
                hyperlinks.enabled = false;
                break;
            }
        }

        // 4. CLI overrides
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
            git_ignore = false;
        }
        if cli.one_per_line {
            one_per_line = true;
        }
        if cli.depth.is_some() {
            depth = cli.depth;
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
        }
    }
}
