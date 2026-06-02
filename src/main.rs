mod cli;
mod config;
mod display;
mod fs;
mod git;
mod theme;

use clap::Parser;
use cli::Cli;
use anyhow::Result;
use fs::walker::Walker;

fn main() -> Result<()> {
    let args = Cli::parse();

    let walker = Walker::new(&args.path, args.all);
    let entries = walker.collect()?;

    for entry in entries {
        println!("{}", entry.name);
    }

    Ok(())
}
