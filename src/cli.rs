use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "elx")]
#[command(about = "A personal ls-clone in Rust", long_about = None)]
pub struct Cli {
    /// Path to the directory or file
    #[arg(default_value = ".")]
    pub path: PathBuf,

    /// Use long listing format
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
    #[arg(long)]
    pub no_icons: bool,

    /// Do not use colors in output
    #[arg(long)]
    pub no_color: bool,

    /// Display one entry per line
    #[arg(short = '1', long = "one-per-line")]
    pub one_per_line: bool,

    /// Generate a default configuration file
    #[arg(long)]
    pub init_config: bool,
}
