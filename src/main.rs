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
use display::grid::{GridOptions, GridEntry, render};
use display::get_terminal_width;
use unicode_width::UnicodeWidthStr;

fn main() -> Result<()> {
    let args = Cli::parse();

    if !args.path.exists() {
        anyhow::bail!("elx: cannot access '{}': No such file or directory", args.path.display());
    }

    let theme = Theme::new();
    let walker = Walker::new(&args.path, args.all);
    let entries = walker.collect()?;

    if args.long {
        // Long view placeholder
        for entry in &entries {
            println!("{}", entry.name);
        }
    } else {
        let terminal_width = get_terminal_width();
        let grid_entries: Vec<GridEntry> = entries.iter().map(|entry| {
            let icon_str = if args.no_icons {
                "".to_string()
            } else {
                theme.icons.get_icon(&entry.path, entry.metadata.is_dir())
            };

            let icon_width = if icon_str.is_empty() { 0 } else { icon_str.width() + 1 };
            let display_width = icon_width + entry.name.width();

            let icon = if icon_str.is_empty() {
                "".to_string()
            } else {
                format!("{} ", icon_str)
            };

            let name = if args.no_color {
                entry.name.clone()
            } else {
                theme.colors.colorize(&entry.name, &entry.metadata).to_string()
            };

            GridEntry {
                display_name: format!("{}{}", icon, name),
                display_width,
            }
        }).collect();

        let output = render(GridOptions {
            terminal_width,
            entries: grid_entries,
        });
        print!("{}", output);
    }

    Ok(())
}
