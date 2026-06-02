use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "elx")]
#[command(about = "A personal ls-clone in Rust", long_about = None)]
pub struct Cli {
    /// Directory to list
    #[arg(default_value = ".")]
    pub path: PathBuf,

    /// Use long listing format
    #[arg(short = 'l', long)]
    pub long: bool,

    /// Show hidden files
    #[arg(short = 'a', long)]
    pub all: bool,

    /// Recurse into directories as a tree
    #[arg(short = 'T', long)]
    pub tree: bool,

    /// Maximum depth for tree view
    #[arg(long)]
    pub depth: Option<usize>,
}
