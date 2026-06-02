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
use display::long::LongView;
use display::get_terminal_width;
use unicode_width::UnicodeWidthStr;
use std::io::IsTerminal;
use config::Config;

fn main() -> Result<()> {
    let mut args = Cli::parse();
    let config = Config::load();

    if args.init_config {
        let config_path = Config::get_config_path();
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        
        let toml_string = toml::to_string_pretty(&Config::default())?;
        std::fs::write(&config_path, toml_string)?;
        println!("Default configuration written to: {}", config_path.display());
        return Ok(());
    }

    // Apply global config defaults if not overridden by CLI flags
    if !config.icons {
        args.no_icons = true;
    }
    if !config.color {
        args.no_color = true;
    }
    if config.classify {
        args.classify = true;
    }

    // TTY detection and overrides
    if !std::io::stdout().is_terminal() {
        if config.when_not_tty.no_icons {
            args.no_icons = true;
        }
        if config.when_not_tty.no_color {
            args.no_color = true;
        }
        if config.when_not_tty.one_per_line {
            args.one_per_line = true;
        }
    }

    if !args.path.exists() {
        anyhow::bail!("elx: cannot access '{}': No such file or directory", args.path.display());
    }

    let theme = Theme::new();
    let walker = Walker::new(&args.path, args.all);
    let entries = walker.collect()?;

    if args.long {
        let view = LongView::new(&entries, &theme, &args, config.long.columns, args.classify);
        view.render();
    } else {
        let terminal_width = get_terminal_width();
        let grid_entries: Vec<GridEntry> = entries.iter().map(|entry| {
            let icon_str = if args.no_icons {
                "".to_string()
            } else {
                theme.icons.get_icon(&entry.path, entry.metadata.is_dir())
            };

            let icon_width = if icon_str.is_empty() { 0 } else { icon_str.width() + 1 };
            
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

            let suffix = if args.classify && entry.metadata.is_dir() { "/" } else { "" };
            let suffix_width = suffix.len();

            GridEntry {
                display_name: format!("{}{}{}", icon, name, suffix),
                display_width: icon_width + entry.name.width() + suffix_width,
            }
        }).collect();

        let output = render(GridOptions {
            terminal_width,
            entries: grid_entries,
            one_per_line: args.one_per_line,
        });
        print!("{}", output);
    }

    Ok(())
}
