mod cli;
mod config;
mod display;
mod fs;
mod git;
mod sort;
mod theme;

use anyhow::Result;
use clap::Parser;
use cli::Cli;
use config::Config;
use display::DisplayOptions;
use display::get_terminal_width;
use display::grid::{GridEntry, GridOptions, render};
use display::long::LongView;
use fs::filter::EntryFilter;
use fs::walker::{HiddenCounts, WalkOptions, Walker};
use sort::{SortOptions, sort_entries};
use theme::Theme;
use unicode_width::UnicodeWidthStr;

fn main() -> Result<()> {
    // Rust ignores SIGPIPE, so `elx | head` would panic in println!. Die quietly like ls instead.
    // SAFETY: called before any other thread exists; SIG_DFL is a valid handler.
    unsafe {
        libc::signal(libc::SIGPIPE, libc::SIG_DFL);
    }

    let mut args = Cli::parse();
    if args.depth.is_some() {
        args.tree = true;
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
        let mut entry = fs::entry::Entry::from_path(args.path.clone())?;
        let status_path = git::status_path(&entry.abs_path);
        if let Some(repo_root) = status_path.parent().and_then(git::find_repo) {
            let statuses = git::get_statuses(&repo_root, true);
            entry.git_status = git::status_for_path(&statuses, &entry.abs_path);
        }
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
        WalkOptions {
            show_hidden: options.show_hidden,
            git_ignore: options.git_ignore,
            ignore_files: options.ignore_files,
            max_depth,
            ignore_globs: options.ignore_globs.clone(),
            collapse_globs: options.collapse_globs.clone(),
            count_hidden: options.hidden_summary,
        },
    );
    let sort_opts = build_sort_options(&config, &args);
    if args.changed && git::find_repo(&args.path).is_none() {
        eprintln!(
            "elx: --changed: {} is not inside a git repository",
            args.path.display()
        );
    }
    let filter = EntryFilter {
        changed: args.changed,
        only: if args.only.is_empty() {
            None
        } else {
            Some(fs::walker::build_matcher(&args.path, &args.only)?)
        },
        dirs_only: args.dirs_only,
        files_only: args.files_only,
    };
    let (entries, hidden) = walker.collect()?;
    let mut entries = filter.apply(entries);
    sort_entries(&mut entries, &sort_opts);

    render_entries(entries, &args, &config, &options, &theme);
    if options.hidden_summary
        && let Some(summary) = hidden_summary(&hidden)
    {
        if options.color {
            use crossterm::style::{Color, Stylize};
            println!("{}", summary.with(Color::DarkGrey));
        } else {
            println!("{summary}");
        }
    }

    Ok(())
}

fn hidden_summary(hidden: &HiddenCounts) -> Option<String> {
    let parts: Vec<String> = [
        (hidden.dotfiles, "dotfiles", "-a"),
        (hidden.gitignored, "gitignored", "-G"),
        (hidden.pattern, "by pattern", "-u"),
    ]
    .into_iter()
    .filter(|(count, _, _)| *count > 0)
    .map(|(count, what, flag)| format!("{count} {what} ({flag})"))
    .collect();
    (!parts.is_empty()).then(|| format!("{} hidden: {}", hidden.total(), parts.join(", ")))
}

fn build_sort_options(config: &Config, cli: &Cli) -> SortOptions {
    use crate::config::GroupDirs;

    let mut opts = SortOptions::from(&config.sort);

    let (cli_key, shorthand_forces_reverse) = cli.resolve_sort_key();
    if let Some(key) = cli_key {
        opts.by = key;
    }
    if shorthand_forces_reverse {
        opts.reverse = true;
    }
    // Like ls: -r flips whatever order is in effect, so -tr lists oldest first.
    if cli.reverse {
        opts.reverse = !opts.reverse;
    }

    if cli.dirs_first {
        opts.group_dirs = GroupDirs::First;
    } else if cli.dirs_last {
        opts.group_dirs = GroupDirs::Last;
    } else if cli.dirs_mixed {
        opts.group_dirs = GroupDirs::Mixed;
    }

    if cli.dotfiles_first {
        opts.dotfiles_first = true;
    }
    if cli.case_sensitive {
        opts.ignore_case = false;
    }

    opts
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

                let safe_name = display::formatter::escape_terminal_text(&entry.name);
                let name = display::formatter::colored_name(theme, options, entry, &safe_name);

                let suffix = if options.classify && entry.metadata.is_dir() {
                    "/"
                } else {
                    ""
                };
                let suffix_width = suffix.len();

                use display::formatter::{file_uri, should_hyperlink, wrap_hyperlink};

                let display_name = if should_hyperlink(options, entry) {
                    let uri = file_uri(&entry.abs_path);
                    wrap_hyperlink(&uri, &format!("{}{}{}", icon, name, suffix))
                } else {
                    format!("{}{}{}", icon, name, suffix)
                };

                GridEntry {
                    display_name,
                    display_width: icon_width + safe_name.width() + suffix_width,
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

#[cfg(test)]
mod tests {
    use super::build_sort_options;
    use crate::cli::Cli;
    use crate::config::Config;
    use clap::Parser;

    fn reverse(args: &[&str], config_reverse: bool) -> bool {
        let cli = Cli::try_parse_from(std::iter::once("elx").chain(args.iter().copied())).unwrap();
        let mut config = Config::default();
        config.sort.reverse = config_reverse;
        build_sort_options(&config, &cli).reverse
    }

    #[test]
    fn hidden_summary_lists_only_nonzero_reasons() {
        use super::{HiddenCounts, hidden_summary};
        assert_eq!(hidden_summary(&HiddenCounts::default()), None);
        let counts = HiddenCounts {
            dotfiles: 3,
            gitignored: 0,
            pattern: 2,
        };
        assert_eq!(
            hidden_summary(&counts).as_deref(),
            Some("5 hidden: 3 dotfiles (-a), 2 by pattern (-u)")
        );
    }

    #[test]
    fn reverse_flag_flips_the_effective_order() {
        assert!(!reverse(&[], false));
        assert!(reverse(&["-r"], false));
        assert!(reverse(&["-t"], false));
        assert!(!reverse(&["-tr"], false));
        assert!(!reverse(&["-Sr"], false));
        assert!(!reverse(&["-r"], true));
    }
}
