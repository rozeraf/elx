# elx — personal ls-clone in Rust

`elx` (explore + ls + x) is a personal replacement for `ls`, built from scratch in Rust. It's designed to be fast, highly customizable, and tailored for a specific workflow, rather than being a universal tool for everyone.

## Why elx?

While tools like `eza` and `lsd` are excellent, they often come with trade-offs:
- **Configuration:** Limited or non-existent configuration files (relying on env vars or flags).
- **Git Integration:** Can be superficial or missing entirely.
- **Filtering:** Lacking advanced glob-pattern ignoring.
- **Customization:** Limited control over icons and colors without forking.

`elx` aims to solve these by providing a layered configuration system and deep integration with tools like `ignore` (used in ripgrep) and `gix`.

## Key Features

- **Display Modes:** Grid (default), Long (`-l`), and Tree (`-T`).
- **Tree Mode:** Recursive directory visualization with configurable depth (`--depth`).
- **Metadata:** Integrated metadata display (permissions, owner, size, date) in both Long and Tree views.
- **Git Status:** Integrated file status (`M`, `A`, `D`, `?`, `!`) powered by `gix`.
- **Smart Ignoring:** Respects `.gitignore` by default via the `ignore` crate.
- **Rich Visuals:** Nerd Font icons and color-coded output by file type.
- **Layered Config:** Robust configuration via `config.toml`, environment variables, and CLI flags.
- **Auto-Config:** Easy setup with `--init-config` to generate a default configuration file.
- **Classify:** Automatic indicator (`/`) for directories via `-F` or config.

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

## Requirements

- **Rust:** Latest stable version.
- **Terminal:** A terminal with [Nerd Fonts](https://www.nerdfonts.com/) installed for icons.
- **OS:** Linux (primary target).

---

Built with ❤️ and Rust.
