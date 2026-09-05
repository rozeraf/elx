use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "elx", version)]
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
    #[arg(short = 'G', long)]
    pub no_git_ignore: bool,

    /// Display one entry per line
    #[arg(short = '1', long = "one-per-line")]
    pub one_per_line: bool,

    /// Ignore patterns (glob), e.g., "*.json,node_modules,bin"
    #[arg(long, value_delimiter = ',')]
    pub ignore: Vec<String>,

    /// Generate a default configuration file
    #[arg(long)]
    pub init_config: bool,

    /// Analyze and update config file to current schema
    #[arg(long)]
    pub update_config: bool,

    /// Show what --update-config would do without making changes
    #[arg(long, requires = "update_config")]
    pub dry_run: bool,
}

impl Cli {
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
}
