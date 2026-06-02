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
use theme::Theme;

fn main() -> Result<()> {
    let args = Cli::parse();

    if !args.path.exists() {
        anyhow::bail!("elx: cannot access '{}': No such file or directory", args.path.display());
    }

    let theme = Theme::new();
    let walker = Walker::new(&args.path, args.all);
    let entries = walker.collect()?;

    for entry in entries {
        let icon = if args.no_icons {
            "".to_string()
        } else {
            format!("{} ", theme.icons.get_icon(&entry.path, entry.metadata.is_dir()))
        };

        let name = if args.no_color {
            entry.name.clone().into()
        } else {
            theme.colors.colorize(&entry.name, &entry.metadata).to_string()
        };

        println!("{}{}", icon, name);
    }

    Ok(())
}
