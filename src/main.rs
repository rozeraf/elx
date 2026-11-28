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
    if let Some(shell) = args.completions {
        clap_complete::generate(
            shell,
            &mut <Cli as clap::CommandFactory>::command(),
            "elx",
            &mut std::io::stdout(),
        );
        return Ok(());
    }
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
    if args.print_config {
        print!(
            "{}",
            effective_config(
                &options,
                &build_sort_options(&config, &args, &options),
                &args
            )
        );
        return Ok(());
    }
    let theme = Theme::new();

    if !args.path.is_dir() {
        let mut entry = fs::entry::Entry::from_path(args.path.clone())?;
        let status_path = git::status_path(&entry.abs_path);
        if let Some(repo_root) = status_path.parent().and_then(git::find_repo) {
            let statuses =
                git::get_statuses(&repo_root, true, std::slice::from_ref(&entry.abs_path));
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
            cli_ignore_globs: args.ignore.clone(),
            collapse_globs: options.collapse_globs.clone(),
            count_hidden: options.hidden_summary,
        },
    );
    let sort_opts = build_sort_options(&config, &args, &options);
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
    let (entries, stats) = walker.collect()?;
    let mut entries = filter.apply(entries);
    sort_entries(&mut entries, &sort_opts);

    render_entries(entries, &args, &config, &options, &theme);
    if options.hidden_summary
        && let Some(summary) = hidden_summary(&stats.hidden)
    {
        if !std::io::IsTerminal::is_terminal(&std::io::stdout()) {
            eprintln!("{summary}");
        } else if options.color {
            use crossterm::style::{Color, Stylize};
            println!("{}", summary.with(Color::DarkGrey));
        } else {
            println!("{summary}");
        }
    }

    if stats.errors > 0 {
        // process::exit skips destructors, so flush buffered (piped) output first.
        let _ = std::io::Write::flush(&mut std::io::stdout());
        std::process::exit(1);
    }
    Ok(())
}

fn value_name<T: clap::ValueEnum>(value: T) -> String {
    value
        .to_possible_value()
        .map(|v| format!("{:?}", v.get_name()))
        .unwrap_or_default()
}

fn effective_config(options: &DisplayOptions, sort: &SortOptions, args: &Cli) -> String {
    let path = Config::get_config_path();
    let state = if path.exists() { "" } else { " (not found)" };
    let sections: Vec<&str> = [
        (args.show_hidden(), "[all]"),
        (args.tree, "[tree]"),
        (
            !std::io::IsTerminal::is_terminal(&std::io::stdout()),
            "[when_not_tty]",
        ),
    ]
    .into_iter()
    .filter_map(|(on, name)| on.then_some(name))
    .collect();
    let active = if sections.is_empty() {
        "none".to_string()
    } else {
        sections.join(", ")
    };
    let depth = options.depth.map_or_else(
        || "# depth = unlimited".to_string(),
        |d| format!("depth = {d}"),
    );
    let secondary = sort.secondary_by.map_or_else(
        || "# secondary_by = (none)".to_string(),
        |k| format!("secondary_by = {}", value_name(k)),
    );
    format!(
        "# Effective settings for this invocation, not a config file.
# config: {path}{state}
# active sections: {active}
icons = {icons}
color = {color}
hyperlinks = {hyperlinks}
classify = {classify}
one_per_line = {one_per_line}
git_ignore = {git_ignore}
show_hidden = {show_hidden}
ignore_files = {ignore_files}
ignore = {ignore:?}
collapse = {collapse:?}
show_hidden_summary = {summary}
{depth}
permissions_format = {permissions}

[sort]
by = {by}
{secondary}
ignore_case = {ignore_case}
reverse = {reverse}
group_dirs = {group_dirs}
dotfiles_first = {dotfiles_first}
",
        path = path.display(),
        icons = options.icons,
        color = options.color,
        hyperlinks = options.hyperlinks.enabled,
        classify = options.classify,
        one_per_line = options.one_per_line,
        git_ignore = value_name(options.git_ignore),
        show_hidden = options.show_hidden,
        ignore_files = options.ignore_files,
        ignore = options.ignore_globs,
        collapse = options.collapse_globs,
        summary = options.hidden_summary,
        permissions = value_name(options.permissions_format),
        by = value_name(sort.by),
        ignore_case = sort.ignore_case,
        reverse = sort.reverse,
        group_dirs = value_name(sort.group_dirs),
        dotfiles_first = sort.dotfiles_first,
    )
}

fn hidden_summary(hidden: &HiddenCounts) -> Option<String> {
    let parts: Vec<String> = [
        (hidden.dotfiles, "dotfiles", " (-a)"),
        (hidden.gitignored, "gitignored", " (-G)"),
        (hidden.pattern, "by config pattern", " (--no-config-ignore)"),
        (hidden.ignore_files, "by ignore files", " (-u)"),
        (hidden.cli_pattern, "by --ignore", ""),
    ]
    .into_iter()
    .filter(|(count, _, _)| *count > 0)
    .map(|(count, what, hint)| format!("{count} {what}{hint}"))
    .collect();
    (!parts.is_empty()).then(|| format!("{} hidden: {}", hidden.total(), parts.join(", ")))
}

fn build_sort_options(config: &Config, cli: &Cli, options: &DisplayOptions) -> SortOptions {
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

    opts.group_dirs = options.group_dirs;

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
        let options = crate::display::DisplayOptions::new(&config, &cli);
        build_sort_options(&config, &cli, &options).reverse
    }

    #[test]
    fn hidden_summary_lists_only_nonzero_reasons() {
        use super::{HiddenCounts, hidden_summary};
        assert_eq!(hidden_summary(&HiddenCounts::default()), None);
        let counts = HiddenCounts {
            dotfiles: 3,
            pattern: 2,
            cli_pattern: 1,
            ..Default::default()
        };
        assert_eq!(
            hidden_summary(&counts).as_deref(),
            Some(
                "6 hidden: 3 dotfiles (-a), 2 by config pattern (--no-config-ignore), 1 by --ignore"
            )
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
