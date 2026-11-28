# elx — personal ls-clone in Rust

`elx` is a personal replacement for `ls`, built from scratch in Rust. It's designed to be fast, highly customizable, and tailored for a specific workflow, rather than being a universal tool for everyone.

## Why elx?

While tools like `eza` and `lsd` are excellent, they often come with trade-offs:
- **Configuration:** Limited or non-existent configuration files (relying on env vars or flags).
- **Git Integration:** Can be superficial or missing entirely.
- **Filtering:** Lacking advanced glob-pattern ignoring.
- **Customization:** Limited control over icons and colors without forking.

`elx` aims to solve these by providing a layered configuration system and deep integration with tools like `ignore` (used in ripgrep) and `gix`.

## Key Features

- **Display Modes:** Grid (default), Long (`-l`), and Tree (`-T`).
- **Several paths:** `elx *.toml` or `elx src tests` work like `ls`: files first, then each directory under a `path:` header.
- **Tree Mode:** Recursive directory visualization with configurable depth (`--depth`/`-L`, which also turns tree mode on).
- **Metadata:** Integrated metadata display (permissions, owner, size, date) in both Long and Tree views.
- **Git Status:** Integrated file status (`U` conflict, `M`, `A`, `D`, `?`, `!`) powered by `gix`, computed only for the listed path.
- **Terminal Hyperlinks (OSC 8):** Clickable file names in supported terminals with granular control.
- **Smart Ignoring:** Respects `.gitignore` by default and supports custom glob patterns via `--ignore`.
- **Rich Visuals:** Nerd Font icons and color-coded output by file type.
- **Layered Config:** Robust configuration via `config.toml`, environment variables, and CLI flags.
- **Auto-Config:** Easy setup with `--init-config` to generate a default configuration file.
- **Classify:** `ls -F` indicators (`/` directory, `@` symlink, `*` executable, `|` FIFO, `=` socket) via `-F` or config.

In long listings, directory Git summaries show `M` when tracked descendants have
changes (including staged changes and deletions), or `?` when only untracked
descendants are present. Ignored files do not mark their parent directories as
modified. Use `-G` to include ignored entries and show their `!` status. Git status
also works for an explicitly listed file and through directory symlinks.

## Filtering

`elx` provides powerful filtering capabilities:

- **Git Ignore:** Entries matched by `.gitignore` are hidden by default. Set `git_ignore = "dim"` to show them
  greyed out, or `"show"` to list them normally (same as `-G`, `--nogit`, `--no-git-ignore`).
  `--git-ignore <hide|dim|show>` overrides the config for one run.
  The old `true`/`false` values still work (`true` = hide, `false` = show). `.git/info/exclude` and the global
  git excludes file follow the same setting.
- **Custom Patterns:** `--ignore` skips files or directories using gitignore-style patterns, comma-separated.
  Patterns containing a `/` are anchored to the listed directory. Config patterns come first, CLI patterns after,
  and the last match wins, so `!pattern` re-includes something the config hides:
  ```bash
  elx --ignore "*.json,node_modules,target"
  elx -a --ignore '!.env.example'   # config has ignore = [".env*"]
  ```
  `--no-config-ignore` drops the config patterns for one run (`--ignore` still applies).
- **`.elxignore`:** Per-project ignore file with gitignore syntax, read in every directory like `.gitignore`
  (`.ignore` files work too). Unlike `.gitignore`, it is not affected by `git_ignore`.
- **Narrowing:** `--only <globs>` shows only matching files, `-D`/`--dirs-only` only directories, `--files-only`
  only non-directories, `--changed` only entries with git changes (`U`, `M`, `A`, `D`, `?`; deleted files are no longer
  on disk, so only their directories show up). In tree view the directories
  leading to a match stay, empty branches are dropped. `elx -T --changed` gives an overview of what changed in a repo.
- **Hidden Summary:** In a terminal, a line below the listing tells what was hidden and how to show it, e.g.
  `4 hidden: 3 dotfiles (-a), 1 gitignored (-G)`. Other reasons: config patterns (`--no-config-ignore`),
  ignore files such as `.elxignore` (`-u`) and your own `--ignore`. When the listing is piped or redirected, the
  line goes to stderr instead (only if stderr is a terminal). Disable with `show_hidden_summary = false`.
- **Show Everything:** `-u`/`--unrestricted` (or `-aa`) turns off all filtering: dotfiles, `.gitignore`,
  `.elxignore`/`.ignore` and config patterns. Explicit `--ignore` and `--git-ignore` still apply.
- **Tree Collapse:** In tree view, directories matching `collapse` patterns (gitignore syntax) are shown as
  `name … N` (N = number of direct entries) without listing their contents. `--collapse <globs>` adds patterns, `--no-collapse` expands everything.
- **Persistent Ignore:** Patterns can also be defined in your `config.toml`:
  ```toml
  ignore = ["*.tmp", ".cache", "bin"]
  ```

## Configuration

`elx` uses a layered configuration system. You can generate a default config file using:

```bash
elx --init-config
```

The config file is located at `~/.config/elx/config.toml`.

To check what a given command line actually uses, run `elx --print-config [flags]`: it prints the settings after
merging the config file, the active sections and the flags.

### Per-mode overrides

Sections `[long]` (active with `-l`), `[all]` (active with `-a`), `[tree]` (active with `-T`) and `[when_not_tty]` (output redirected to a file or pipe) override top-level keys while the mode is on. `[long]` also holds the long-view settings (`columns`, `headers`, ...).
Unset keys inherit the top-level value; lists (`ignore`, `collapse`) replace the global list instead of extending it.
Supported keys: `icons`, `color`, `classify`, `git_ignore`, `ignore`, `collapse`, `one_per_line`, `group_dirs`.
Without a `[when_not_tty]` section, redirected output has no icons, no colors and one entry per line; hyperlinks are always off there.

```toml
classify = false

[all]
classify = true
color = false

[tree]
git_ignore = "dim"
collapse = ["target", "node_modules", ".git"]
```

Order: top-level (`group_dirs` from `[sort]`) → `[long]` → `[all]` → `[tree]` → `[when_not_tty]` → CLI flags (CLI always wins). For example, directory grouping per view:

```toml
[sort]
group_dirs = "first"

[long]
group_dirs = "mixed"

[tree]
group_dirs = "last"
```

### Hyperlinks (OSC 8)

One of the unique features of `elx` is configurable support for terminal hyperlinks. You can control which entry types become links:

```toml
[hyperlinks]
# Global toggle
enabled = true

# Toggle links for specific entry types
files = true
dirs = false
symlinks = true
executables = true
```

## Architecture

```
elx/
├── src/
│   ├── main.rs          — Entry point, CLI orchestration
│   ├── cli.rs           — Command-line interface definitions (clap)
│   ├── config.rs        — Layered configuration logic (figment)
│   ├── fs/              — Filesystem traversal and metadata
│   ├── display/         — Output formatting (Grid, Long, Tree)
│   ├── theme/           — Icons and color logic
│   └── git.rs           — Git status integration (gix)
└── config/              — Default configuration templates
```

## Development Roadmap

### Phase 1: Skeleton (Completed)
- [x] Project initialization and dependency setup.
- [x] Basic CLI with `clap`.
- [x] Directory listing implementation.

### Phase 2: Core Views (Completed)
- [x] Grid view with variable column alignment.
- [x] Long view with metadata (permissions, owner, size, date).
- [x] Nerd Font icons and terminal-aware coloring.
- [x] Case-insensitive sorting by name.

### Phase 3: Advanced Features (Completed)
- [x] Tree view implementation with recursive traversal.
- [x] Configurable depth control (`--depth`).
- [x] Integrated metadata formatting in Tree view (`-Tl`).
- [x] Directory classification (`-F/--classify`).

### Phase 4: Git Integration (Completed)
- [x] Connect `gix` for Git repository detection.
- [x] Implement file status enrichment (`M`, `A`, `D`, `?`, `!`).
- [x] Display Git status in Long and Tree views.

### Phase 5: Configuration (Completed)
- [x] Figment-based layered configuration system.
- [x] XDG-compliant config path (`~/.config/elx/config.toml`).
- [x] TTY detection with automatic overrides for pipes and redirects.
- [x] Default config generation via `--init-config`.

## Installation

```bash
make install
```

Shell completions (bash, zsh, fish, elvish, powershell):

```bash
elx --completions zsh > ~/.local/share/zsh/site-functions/_elx
```

## Requirements

- **Rust:** Latest stable version.
- **Terminal:** A terminal with [Nerd Fonts](https://www.nerdfonts.com/) installed for icons.
- **OS:** Linux (primary target).

---

Built with ❤️ and Rust.
