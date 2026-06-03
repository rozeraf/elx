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
use display::DisplayOptions;
use unicode_width::UnicodeWidthStr;
use config::Config;

fn main() -> Result<()> {
    let args = Cli::parse();
    let config = Config::load();

    if args.init_config {
        let config_path = Config::get_config_path();
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        
        std::fs::write(&config_path, Config::get_default_config_template())?;
        println!("Default configuration written to: {}", config_path.display());
        return Ok(());
    }

    let options = DisplayOptions::new(&config, &args);

    if !args.path.exists() {
        anyhow::bail!("elx: cannot access '{}': No such file or directory", args.path.display());
    }

    let theme = Theme::new();
    let max_depth = if args.tree {
        options.depth.unwrap_or(usize::MAX)
    } else {
        1
    };

    let walker = Walker::new(&args.path, args.all, options.git_ignore, max_depth);
    let entries = walker.collect()?;

    if args.tree {
        use display::tree::TreeView;
        let view = TreeView::new(&entries, &theme, &options, config.long.columns, config.long.headers);
        view.render();
    } else if args.long {
        let view = LongView::new(&entries, &theme, &options, config.long.columns, config.long.headers);
        view.render();
    } else {
        let terminal_width = get_terminal_width();
        let grid_entries: Vec<GridEntry> = entries.iter().map(|entry| {
            let icon_str = if !options.icons {
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

            let name = if !options.color {
                entry.name.clone()
            } else {
                theme.colors.colorize(&entry.name, &entry.metadata).to_string()
            };

            let suffix = if options.classify && entry.metadata.is_dir() { "/" } else { "" };
            let suffix_width = suffix.len();

            GridEntry {
                display_name: format!("{}{}{}", icon, name, suffix),
                display_width: icon_width + entry.name.width() + suffix_width,
            }
        }).collect();

        let output = render(GridOptions {
            terminal_width,
            entries: grid_entries,
            one_per_line: options.one_per_line,
        });
        print!("{}", output);
    }

    Ok(())
}
