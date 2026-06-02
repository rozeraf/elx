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

    if !args.path.exists() {
        anyhow::bail!("elx: cannot access '{}': No such file or directory", args.path.display());
    }

    let walker = Walker::new(&args.path, args.all);
    let entries = walker.collect()?;

    // В будущем здесь будет логика выбора Display режима (Grid, Long, Tree)
    for entry in entries {
        println!("{}", entry.name);
    }

    Ok(())
}
