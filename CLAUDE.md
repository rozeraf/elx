# elx

Personal `ls` replacement in Rust: grid, long (`-l`) and tree (`-T`) views, git status via `gix`,
filtering via `ignore`, layered config via `figment`.

## Language
- Talk to the user in Russian.
- Everything inside files is English: code, comments, CLI help, error messages, docs.
  Some existing comments are in Russian; leave them unless you are editing that code anyway.

## Commands
- `cargo build` / `cargo test` / `cargo clippy`
- `make install`: release build into `~/.local/bin/elx`.
- Manual check: `./target/debug/elx -l`, `-T --depth 2`, `--no-color --no-icons` (no TTY changes the defaults, see `[when_not_tty]`).
- Integration tests are in `tests/` (`git_status.rs`, `terminal_output.rs`); unit tests sit next to the code.
- `cargo fmt --check` and `cargo clippy --all-targets` are clean; keep them that way.

## Git workflow
- `main` = latest release, tagged `vX.Y.Z`. `dev` = integration branch; work there or in a feature branch off it.
- Release: bump `version` in `Cargo.toml` + `Cargo.lock`, commit `chore: release X.Y.Z`, merge into `main`, tag.
- Conventional Commits. Commit on your own after each verified, logically complete step (split by topic). Push, merge and tag only when the user asks.

## Architecture
- `src/cli.rs`: clap definitions. Short flags in use: `l a T F G 1 o t S X v U r u D`.
- `src/config.rs`: `Config` + `get_default_config_template()` (the `--init-config` output).
- `src/display/mod.rs`: `DisplayOptions::build` merges config → `[all]` → `[tree]` → `when_not_tty` → CLI flags (unit-tested there, `is_tty` passed in). Put new override logic here.
- `src/fs/walker.rs`: one `WalkBuilder` per directory level (depth 1), recursion done manually. `ignore`/`collapse` patterns are `Gitignore` matchers rooted at the listed path (`filter_entry`, never applied to the root itself).
- `src/sort.rs`: all sorting. The walker does not sort.
- `src/git.rs`: status map keyed by canonical absolute path.

## Gotchas
- Config structs use `#[serde(deny_unknown_fields)]`. A removed or renamed field breaks existing user configs
  (`Config::load` prints an error and falls back to defaults). When changing config fields, update the template
  in `config.rs` and the README examples in the same change.
- Config sources: `~/.config/elx/config.toml` then `ELX_*` env vars, then CLI flags.
- `rtk` rewrites shell commands and can trim output (`git show`, `git diff`). Use `rtk proxy <cmd>` when you need exact file content.

## Project constraints
- Linux only, Nerd Fonts assumed installed.
- Correctness first, performance second.
- Architectural changes are discussed with the user first, not done on the fly.
- Behaviour references: eza, lsd, GNU coreutils `ls.c` (look up specific parts, don't port wholesale).

## Context files
- `todo.md`: task checklist.
- `filtering-ux.md`: planned filtering/UX work (per-mode overrides `[all]`/`[tree]`, `git_ignore = dim`, tree collapse, etc.).
