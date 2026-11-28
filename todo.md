# TODO

Details and rationale for each feature: `filtering-ux.md` (section numbers in brackets).
Order matters: later items build on earlier ones.

## Cleanup (quick)

- [x] Fix README `[hyperlinks]` example: uses `directories`, `underline_*`, `exclude_env`, which `HyperlinkConfig` doesn't have (`deny_unknown_fields` → config fails to load). Use `dirs`, `executables`.
- [x] Remove unused `toml` dependency from `Cargo.toml`. Check whether `toml_edit` is still used after the config_updater removal.
- [x] Remove redundant `homepage` from `Cargo.toml` (clippy `redundant_homepage`).
- [x] `src/sort.rs:195`: `sort_entries(entries: &mut Vec<Entry>)` → `&mut [Entry]` (clippy `ptr_arg`).
- [x] Run `cargo fmt` once on `cli.rs` and `sort.rs` in a separate `style:` commit so later diffs stay clean.
- [x] Translate remaining Russian comments in `cli.rs`, `config.rs`, `sort.rs`, `formatter.rs` to English.
- [x] `-tr` / `-Sr` should flip the order like `ls` (oldest/smallest first); now `-r` just sets `reverse = true`, which `-t`/`-S` already set, so it does nothing (`build_sort_options` in `main.rs`).
- [x] Optional: restore distinct colors for setuid/setgid (magenta) and sticky (cyan) in `format_permissions_colored` (lost in the v0.1.2 merge; now same green as `x`).
- [x] User side: remove `schema_version` from `~/.config/elx/config.toml` (field was dropped in `8c48690`).

- [x] `elx | head` panics with "failed printing to stdout: Broken pipe". Exit quietly on `BrokenPipe` instead (output goes through `println!`/`print!` in all views).

## Filtering

- [x] **Per-mode overrides `[all]` / `[tree]`** [0]: `ModeOverrides` with `icons`, `color`, `classify`, `git_ignore`, `ignore` (replaces). `collapse` gets added with tree collapse.
  - [x] `[when_not_tty]` moved onto `ModeOverrides` (unset keys now inherit instead of defaulting to `false`).
- [x] **`git_ignore = "hide" | "dim" | "show"`** [1]: old booleans still accepted; `-G` = `show`; dim = `DarkGrey` name, only with colors on.
  - [ ] Optional: `--git-ignore=<mode>` CLI flag (now only via config / `[all]` / `[tree]`).
  - Note: `-T` with `dim` and no `--depth` walks into ignored dirs like `target/`; tree collapse fixes that.
- [x] **Tree collapse** [2]: `collapse` (gitignore-style patterns) in config and modes, `--collapse`, `--no-collapse`; rendered as `name …`.
  - [ ] Optional: entry count in the marker (`target … 1234`); needs a `read_dir` per collapsed dir.
- [ ] **Hidden summary footer** [3]
  - [ ] Walker counts skipped entries by reason (hidden / gitignored / pattern).
  - [ ] Footer `N hidden (…) — use -a / -G to show`, TTY only.
  - [ ] `show_hidden_summary = false` to disable.
- [ ] **Show everything / disable config ignore** [4]
  - [ ] `--no-config-ignore`.
  - [ ] `-aa` (`ArgAction::Count`) or `-u`: hidden + gitignore + config patterns off. Check `--all` still works.
- [ ] **`--changed`** [5]: only `M/A/D/?` entries plus their parent dirs.
- [ ] **`--only <globs>`, `-D/--dirs-only`, `--files-only`** [6]; prune empty dirs in tree.
- [ ] **Negation in `--ignore`** (`!pattern` re-includes) [7]. Check `OverrideBuilder` whitelist semantics first.
- [ ] **`.elxignore`** [8]: `WalkBuilder::add_custom_ignore_filename`.

## UX

- [ ] `--sort` as `clap::ValueEnum` instead of `String` (error with valid values instead of silent fallback).
- [ ] `--print-config`: effective config after all layers for the current flags.
- [ ] `-L <N>` alias for `--depth`; `--depth` without `-T` enables tree.
- [ ] Shell completions via `clap_complete` (new dep) behind hidden `--completions <shell>`.

## Later (from idea.md "not in MVP")

- [ ] Custom icon/color themes via config (`[theme]` section; now hardcoded in `src/theme/`).
- [ ] `-R` recursive listing (ls-style sections per directory, unlike `-T`).
- [ ] Directory size (du-style) in long view, opt-in (`--total-size`), since it walks the whole subtree.

## Release

- [ ] Next release after the Cleanup section + per-mode overrides: bump to 0.2.0 (config format changes), merge `dev` → `main`, tag `v0.2.0`.
