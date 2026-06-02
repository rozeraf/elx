# elx — personal ls-clone in Rust

`elx` (explore + ls + x) is a personal replacement for `ls`, built from scratch in Rust. It's designed to be fast, highly customizable, and tailored for a specific workflow, rather than being a universal tool for everyone.

## Why elx?

While tools like `eza` and `lsd` are excellent, they often come with trade-offs:
- **Configuration:** Limited or non-existent configuration files (relying on env vars or flags).
- **Git Integration:** Can be superficial or missing entirely.
- **Filtering:** Lacking advanced glob-pattern ignoring.
- **Customization:** Limited control over icons and colors without forking.

`elx` aims to solve these by providing a layered configuration system and deep integration with tools like `ignore` (used in ripgrep) and `gix`.

## Key Features (MVP)

- **Display Modes:** Grid (default), Long (`-l`), and Tree (`-T`).
- **Git Status:** Integrated file status (`M`, `A`, `?`, `!`) powered by `gix`.
- **Smart Ignoring:** Respects `.gitignore` and custom glob patterns via the `ignore` crate.
- **Rich Visuals:** Nerd Font icons and color-coded output by file type.
- **Layered Config:** Configuration via `config.toml`, environment variables, and CLI flags.

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

### Phase 1: Skeleton (Current)
- [x] Project initialization and dependency setup.
- [ ] Basic CLI with `clap`.
- [ ] Directory listing implementation.

### Phase 2: Core Views
- [ ] Grid view with column alignment.
- [ ] Long view with metadata (permissions, owner, size, date).
- [ ] Nerd Font icons and basic coloring.

### Phase 3: Advanced Features
- [ ] Tree view implementation.
- [ ] Full `.gitignore` support.
- [ ] Git status enrichment.

### Phase 4: Configuration
- [ ] Figment-based configuration system.
- [ ] `~/.config/elx/config.toml` support.

## Installation

*Note: `elx` is currently in early development.*

```bash
cargo install --path .
```

## Requirements

- **Rust:** Latest stable version.
- **Terminal:** A terminal with [Nerd Fonts](https://www.nerdfonts.com/) installed for icons.
- **OS:** Linux (primary target).

---

Built with ❤️ and Rust.
