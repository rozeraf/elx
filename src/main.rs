mod cli;
mod config;
mod config_updater;
mod display;
mod fs;
mod git;
mod theme;

use anyhow::Result;
use clap::Parser;
use cli::Cli;
use config::Config;
use display::DisplayOptions;
use display::get_terminal_width;
use display::grid::{GridEntry, GridOptions, render};
use display::long::LongView;
use fs::walker::Walker;
use theme::Theme;
use unicode_width::UnicodeWidthStr;

fn main() -> Result<()> {
    let args = Cli::parse();

    if args.update_config {
        use config_updater::ConfigUpdater;
        let config_path = Config::get_config_path();
        if !config_path.exists() {
            anyhow::bail!(
                "No config file found at {}. Run --init-config first.",
                config_path.display()
            );
        }
        use std::io::IsTerminal;
        let interactive = !args.dry_run && std::io::stdin().is_terminal();
        let mut updater = ConfigUpdater::new(config_path, interactive)?;
        updater.dry_run = args.dry_run;
        updater.run()?;
        return Ok(());
    }

    let config = Config::load();

    if args.init_config {
        let config_path = Config::get_config_path();
        if config_path.exists() {
            anyhow::bail!(
                "elx: configuration already exists at {}",
                config_path.display()
            );
        }
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        std::fs::write(&config_path, Config::get_default_config_template())?;
        println!(
            "Default configuration written to: {}",
            config_path.display()
        );
        return Ok(());
    }

    if !args.path.exists() && !args.path.is_symlink() {
        anyhow::bail!(
            "elx: cannot access '{}': No such file or directory",
            args.path.display()
        );
    }

    let options = DisplayOptions::new(&config, &args);
    let theme = Theme::new();

    if !args.path.is_dir() {
        let entry = fs::entry::Entry::from_path(args.path.clone())?;
        render_entries(vec![entry], &args, &config, &options, &theme);
        return Ok(());
    }

    let max_depth = if args.tree {
        options.depth.unwrap_or(usize::MAX)
    } else {
        1
    };

    let walker = Walker::new(
        &args.path,
        args.all,
        options.git_ignore,
        max_depth,
        options.ignore_globs.clone(),
    );
    let entries = walker.collect()?;

    render_entries(entries, &args, &config, &options, &theme);

    Ok(())
}

fn render_entries(
    entries: Vec<fs::entry::Entry>,
    args: &Cli,
    config: &Config,
    options: &DisplayOptions,
    theme: &Theme,
) {
    if args.tree {
        use display::tree::TreeView;
        let view = TreeView::new(
            &entries,
            theme,
            options,
            config.long.columns.clone(),
            config.long.headers,
            config.long.autohide_columns,
        );
        view.render();
    } else if args.long {
        let view = LongView::new(
            &entries,
            theme,
            options,
            config.long.columns.clone(),
            config.long.headers,
            config.long.autohide_columns,
        );
        view.render();
    } else {
        let terminal_width = get_terminal_width();
        let grid_entries: Vec<GridEntry> = entries
            .iter()
            .map(|entry| {
                let icon_str = if !options.icons {
                    "".to_string()
                } else {
                    theme.icons.get_icon(&entry.path, entry.metadata.is_dir())
                };

                let icon_width = if icon_str.is_empty() {
                    0
                } else {
                    icon_str.width() + 1
                };

                let icon = if icon_str.is_empty() {
                    "".to_string()
                } else {
                    format!("{} ", icon_str)
                };

                let name = if !options.color {
                    entry.name.clone()
                } else {
                    theme
                        .colors
                        .colorize(&entry.name, &entry.metadata)
                        .to_string()
                };

                let suffix = if options.classify && entry.metadata.is_dir() {
                    "/"
                } else {
                    ""
                };
                let suffix_width = suffix.len();

                use display::formatter::{file_uri, should_hyperlink, wrap_hyperlink};

                // should_hyperlink принимает &DisplayOptions и &Entry
                let display_name = if should_hyperlink(options, entry) {
                    let uri = file_uri(&entry.abs_path);
                    wrap_hyperlink(&uri, &format!("{}{}{}", icon, name, suffix))
                } else {
                    format!("{}{}{}", icon, name, suffix)
                };

                GridEntry {
                    display_name,
                    display_width: icon_width + entry.name.width() + suffix_width,
                }
            })
            .collect();

        let output = render(GridOptions {
            terminal_width,
            entries: grid_entries,
            one_per_line: options.one_per_line,
        });
        print!("{}", output);
    }
}
