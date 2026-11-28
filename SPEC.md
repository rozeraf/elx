# elx specification

Version 0.2.0. This document describes the behaviour of the current code; where it disagrees with
README.md or the config template, the code (and this file) is what actually happens.

## 1. Scope

elx is a personal `ls` replacement for Linux with three views (grid, long, tree), git status,
gitignore-aware filtering and a layered TOML config.

- Platform: Linux only. Icons assume a Nerd Font.
- Priorities: correctness first, performance second.
- Behaviour references: GNU coreutils `ls`, eza, lsd.
- Out of scope for now (see `todo.md`, section Later): icon/color themes in config, `-R` recursive
  listing, directory sizes.

## 2. Invocation

```
elx [OPTIONS] [PATH]
```

`PATH` defaults to `.`. Exactly one path is accepted.

- Missing path (and not a dangling symlink): error `elx: cannot access '<path>': No such file or directory`, exit 1.
- Path is not a directory: that single entry is shown in the selected view, with its git status.
  `--only`, `--changed`, `-D` and `--files-only` do not apply to it (like `ls file`).
- Path is a directory: its contents are listed; in tree view, recursively.

### 2.1 Exit status

| Code | Meaning |
|---|---|
| 0 | Success. |
| 1 | Path not found, invalid config generation, or at least one entry/directory could not be read. Unreadable entries are reported on stderr as `elx: <error>` and the rest of the listing is still printed. |
| 2 | Invalid command line (clap): unknown flag, bad value, `--depth 0`, conflicting flags. |
| SIGPIPE | When stdout is closed (`elx \| head`), elx is killed by SIGPIPE silently, like `ls`. |

## 3. Command-line options

### 3.1 Views

| Flag | Effect |
|---|---|
| (none) | Grid view. |
| `-l`, `--long` | Long view (one entry per line with metadata columns). |
| `-T`, `--tree` | Tree view. Combined with `-l`, metadata columns are printed before the tree branches. |
| `-L`, `--depth <N>` | Maximum tree depth, N ≥ 1. Implies `--tree`. `-L 1` shows only the top level. |
| `-1`, `--one-per-line` | Grid view prints one entry per line. |

### 3.2 Appearance

| Flag | Effect |
|---|---|
| `--no-icons` | No icons. |
| `--no-color` | No colors (no ANSI styling at all). |
| `--no-hyperlinks` | No OSC 8 hyperlinks. |
| `-F`, `--classify` | Append `/` to directories. No other indicators. |
| `-o`, `--octal` | Permissions column in octal (`0755`). Overrides `long.permissions_format`. |

### 3.3 Filtering

| Flag | Effect |
|---|---|
| `-a`, `--all` | Show dotfiles. Activates the `[all]` config section. |
| `-aa`, `-u`, `--unrestricted` | Disable all filtering: dotfiles shown, gitignore `show`, `.ignore`/`.elxignore` off, config ignore patterns off. `--ignore` from the command line still applies. Also activates `[all]`. |
| `-G`, `--no-git-ignore` (alias `--nogit`) | Same as `--git-ignore show`. Conflicts with `--git-ignore`. |
| `--git-ignore <hide\|dim\|show>` | What to do with gitignored entries (section 5.2). |
| `--ignore <GLOBS>` | Extra ignore patterns, comma-separated or repeated, gitignore syntax. Appended after config patterns, so they win over them. |
| `--no-config-ignore` | Drop `ignore` patterns that come from the config (including mode sections). |
| `--only <GLOBS>` | Show only files matching the patterns (gitignore syntax). With `-D`, match directories instead. |
| `-D`, `--dirs-only` | Show only directories. Symlinks to directories are not directories here (like `find -type d`). |
| `--files-only` | Show only non-directories. |
| `--changed` | Show only entries with git status M, A, D or ?, plus the directories leading to them. Deleted files no longer exist on disk, so only their parent directories appear. Outside a git repository a warning is printed on stderr and nothing matches. |
| `--collapse <GLOBS>` | Tree view: show matching directories without descending into them. Appended to config patterns. |
| `--no-collapse` | Ignore configured collapse patterns. Conflicts with `--collapse`. |

### 3.4 Sorting

| Flag | Equivalent |
|---|---|
| `--sort <KEY>` | `name`, `name_natural`, `extension`, `size`, `modified`, `created`, `git_status`, `none`. Unknown keys are an error listing the valid ones. |
| `-t` | `--sort modified`, newest first. |
| `-S` | `--sort size`, largest first. |
| `-X` | `--sort extension`. |
| `-v` | `--sort name_natural`. |
| `-U` | `--sort none` (filesystem order). |
| `-r`, `--reverse` | Flip the order in effect: `-tr` is oldest first, `-r` alone is Z→A. |
| `--dirs-first` / `--dirs-last` / `--dirs-mixed` | Directory grouping. |
| `--dotfiles-first` | Dotfiles before other entries within their group. |
| `--case-sensitive` | Case-sensitive name/extension comparison. |

Among `-t -S -X -v -U --sort`, the last one on the command line wins.

### 3.5 Utility

| Flag | Effect |
|---|---|
| `--init-config` | Write the default config template (section 4.4) to the config path. Fails if the file exists. |
| `--print-config` | Print the effective settings for this invocation (config + active sections + flags) and exit. The first lines name the config file and the active sections. The output is informational, not a valid config file. |
| `--completions <bash\|zsh\|fish\|elvish\|powershell>` | Hidden. Print a shell completion script. |
| `-h`, `--help` / `-V`, `--version` | Help / `elx 0.2.0`. |

## 4. Configuration

### 4.1 Sources and precedence

1. Built-in defaults.
2. `$XDG_CONFIG_HOME/elx/config.toml` (usually `~/.config/elx/config.toml`).
3. Environment variables `ELX_<KEY>` for top-level keys only, e.g. `ELX_ICONS=false`,
   `ELX_SHOW_HIDDEN_SUMMARY=false`. Nested tables cannot be set this way.
4. Mode sections, in this order, each only when its mode is active:
   `[all]` (`-a`/`-u`), `[tree]` (`-T`/`-L`), `[when_not_tty]` (stdout is not a terminal).
   A key set in a section replaces the value from the previous layer; unset keys inherit.
5. Command-line flags, which always win.

A config that fails to parse (syntax error, unknown key, wrong type) prints
`elx: config error: ...` on stderr and elx continues with built-in defaults.
All tables reject unknown keys.

### 4.2 Top-level keys

| Key | Type | Default | Meaning |
|---|---|---|---|
| `icons` | bool | `true` | Nerd Font icons before names. |
| `color` | bool | `true` | ANSI colors. |
| `classify` | bool | `false` | `/` after directories. |
| `git_ignore` | `"hide"\|"dim"\|"show"` or bool | `"hide"` | Gitignored entries; `true` = `hide`, `false` = `show` (pre-0.2 format). |
| `ignore` | list of patterns | `[]` | Ignore patterns, gitignore syntax (section 5.3). |
| `collapse` | list of patterns | `[]` | Tree collapse patterns (section 6.3). |
| `show_hidden_summary` | bool | `true` | Hidden-entries footer (section 5.6). |
| `depth` | integer ≥ 1 | unset (unlimited) | Default tree depth. `0` is a config error. |
| `schema_version` | integer | — | Accepted and ignored, so configs from 0.1.x still load. |

### 4.3 Tables

`[hyperlinks]`: `enabled` (true), `files` (true), `dirs` (false), `symlinks` (true),
`executables` (true). Which entries get an OSC 8 `file://` link; checked in the order
directory → symlink → executable (any x bit) → other file.

`[long]`:

| Key | Default | Meaning |
|---|---|---|
| `columns` | `["git", "permissions", "links", "owner", "group", "size", "date", "name"]` | Column order. |
| `headers` | `true` | Underlined header row (`Git Mode Link User Group Size Date Name`). |
| `autohide_columns` | `true` | Drop columns that are empty for every shown entry (in practice the git column outside a repo or with no changes). |
| `permissions_format` | `"symbolic"` | `symbolic`, `octal`, `both`, `none`. |

`[sort]`: `by` (`name`), `secondary_by` (unset), `ignore_case` (true), `reverse` (false),
`group_dirs` (`first` / `last` / `mixed`, default `first`), `dotfiles_first` (false).

`[all]`, `[tree]`, `[when_not_tty]`: mode sections, all keys optional:
`icons`, `color`, `classify`, `git_ignore`, `ignore`, `collapse`, `one_per_line`.
Lists replace the global list rather than extending it.
`[when_not_tty]` defaults to `icons = false`, `color = false`, `one_per_line = true` when the
section is absent. Hyperlinks are always disabled when stdout is not a terminal.

### 4.4 Default template

`--init-config` writes a commented template with every key, set to the built-in defaults
(except `[when_not_tty]`, which it spells out, and commented examples).

## 5. Filtering

### 5.1 Pipeline

For each listed directory, entries are dropped in this order:

1. Dotfiles (unless `-a`/`-u`).
2. Gitignore rules: `.gitignore`, `.git/info/exclude` and the global git excludes file, only
   when `git_ignore = "hide"` and only inside a git repository (like git itself).
3. `.ignore` and `.elxignore` files (unless `-u`).
4. `ignore` patterns (config, then `--ignore`).

An ignored directory is not descended into. Ignore patterns never apply to the listed root itself.
After the walk, the post-filters `--changed`, `--only`, `-D`, `--files-only` are applied
(section 5.5), then entries are sorted.

### 5.2 `git_ignore` modes

- `hide`: gitignored entries are not listed.
- `dim`: listed, name in dark grey (only when colors are on; otherwise they look normal).
- `show`: listed normally.

In `dim`/`show`, the long view git column shows `!` for them.

### 5.3 Ignore pattern syntax

`ignore`, `--ignore`, `--only` and `collapse` use gitignore syntax rooted at the listed path:

- `*.log` matches at any depth; `/build` or `src/gen` (a pattern containing `/`) is anchored to
  the listed directory.
- A trailing `/` matches directories only.
- The last matching pattern wins; `!pattern` re-includes. Example:
  `--ignore '*.log' --ignore '!keep.log'`.
- As in git, a file inside an excluded directory cannot be re-included.

### 5.4 `.elxignore`

Same syntax and placement rules as `.gitignore` (per directory, applies to the subtree), honoured
regardless of `git_ignore` and of whether the directory is a git repository. Disabled by `-u`.

### 5.5 Post-filters

An entry passes when it satisfies every active filter:

- type: `-D` → directory, `--files-only` → not a directory;
- `--only`: files (or directories with `-D`) matching a pattern; without `-D`, directories
  never match `--only` themselves;
- `--changed`: git status M, A, D or ?.

A directory whose children were read is kept if it passes itself or if any child is kept, so the
tree always shows the path to every match; directories left empty are pruned.
Directories whose children were not read (the `--depth` limit, collapsed directories, all
directories in grid/long view) are kept only if they pass themselves. So
`elx --files-only` without `-T` shows no directories, and `--only` drops a collapsed `target`.

### 5.6 Hidden-entries footer

When stdout is a terminal and `show_hidden_summary = true`, a dark grey line follows the listing:

```
4 hidden: 3 dotfiles (-a), 1 gitignored (-G)
```

Reasons, with the flag that reveals them: `dotfiles (-a)`, `gitignored (-G)`,
`by config pattern (--no-config-ignore)`, `by ignore files (-u)`, `by --ignore`.
Only non-zero reasons appear; nothing is printed when nothing was hidden. Each entry is counted
once, under the first reason in that order. Counted per listed directory, including every
directory walked in tree view, but not inside collapsed directories or beyond `--depth`.
Entries removed by post-filters (section 5.5) are not counted.
"Ignore files" also covers tracked files matched by `.gitignore` (git does not report them as
ignored). Disable the line with `show_hidden_summary = false` or `ELX_SHOW_HIDDEN_SUMMARY=false`.

## 6. Views

### 6.1 Grid

- Names (with icon and classify suffix) are laid out in columns, filled top to bottom then left to
  right, with two spaces between columns, like `ls -C`.
- All entries on one line if they fit; otherwise the largest column count that fits the terminal.
- Width comes from the terminal, 80 when unknown. Display width is Unicode-aware.
- `one_per_line` (or `-1`, or non-TTY default) prints one name per line.

### 6.2 Long

One line per entry: configured columns separated by a space, each padded to the widest value
(the header width counts when headers are on). `links` and `size` are right-aligned, the rest
left-aligned.

| Column | Content |
|---|---|
| `git` | One character: `M` modified (yellow), `A` added/staged (green), `D` deleted (red), `?` untracked (blue), `!` ignored (grey), blank otherwise. |
| `permissions` | `symbolic`: `drwxr-xr-x` with type letter `d - l p s c b`, `s/S` for setuid/setgid, `t/T` for sticky. Colors: type blue (dir), grey (file), cyan (other); `r` yellow, `w` red, `x` green, `s/S` magenta, `t/T` cyan. `octal`: `0755` (4 digits including special bits). `both`: symbolic then octal. `none`: empty. |
| `links` | Hard link count. |
| `owner`, `group` | Name, or numeric id when unknown. Yellow. |
| `size` | Bytes `n B` below 1024, otherwise one decimal with `K M G T` (1024-based). Green. |
| `date` | Modification time, local, `%b %d %H:%M` (e.g. `Nov 28 20:42`). Cyan. |
| `name` | Icon, colored name, classify suffix, ` -> target` for symlinks (target cyan, not resolved). |

### 6.3 Tree

```
├── link -> src
├── src
│   └── main.rs
├── target … 1
└── README.md
```

- The root itself is not printed; its children are the top level.
- Branches use `├── └── │`.
- With `-l`, the long columns are printed in front of each tree line.
- Collapsed directory: `name … N`, where N is the number of direct entries in it (counted with
  one `read_dir`, unfiltered). Its contents are not read. The marker appears only in tree view.
- Depth: `--depth` / `depth`, default unlimited.

### 6.4 Names and safety

- Control characters and bidirectional formatting characters in names and symlink targets are
  escaped (`\n`, `\u{1b}`, ...), and `\` becomes `\\`, so names cannot inject terminal
  sequences and escaped output is unambiguous.
- Name colors: directory bold blue, symlink cyan, executable bold green, other white.
  Gitignored in `dim` mode: dark grey.
- Hyperlinks: `file://` URI of the absolute path (symlinks are not resolved), percent-encoded
  except `A-Z a-z 0-9 - . _ ~ /`.
- Icons: a small built-in table by file name and extension, with default file/directory icons.

## 7. Sorting

Comparison order for two entries:

1. Directory grouping (`group_dirs`). Symlinks to directories count as directories.
   Never affected by `reverse`.
2. `dotfiles_first`. Never affected by `reverse`.
3. Primary key, reversed when `reverse` is on.
4. `secondary_by` key, if set, also reversed.

Keys: `name` (string, case-insensitive by default), `name_natural` (digit runs compared as
numbers: `v2 < v10`), `extension` (then name), `size`, `modified`, `created`,
`git_status` (M, A, D, ?, !, clean), `none` (filesystem order).
`-t` and `-S` set `reverse` so that the natural "newest/largest first" order comes out;
`-r` then flips whatever is in effect. In tree view every directory's children are sorted the same
way. Sorting is stable.

## 8. Git integration

- The repository is discovered from the listed path upwards (gix). Outside a repository there is no
  git status and `.gitignore` files are not honoured (`.elxignore` is).
- Statuses: index vs HEAD (`A`, `D`, `M`) and worktree vs index (`M`, `D`), untracked (`?`),
  ignored (`!`, only computed when needed: `dim`/`show` or the footer). Conflicts show as `M`.
- Directories get a summary status: `M` if any descendant is modified/added/deleted, otherwise `?`
  if any descendant is untracked. Ignored descendants do not mark the directory.
- An entry inside an ignored directory is ignored.
- Paths are matched by canonical absolute path of the parent directory (the final symlink is not
  followed), so listing through a symlinked directory or a relative path gives the same statuses.

## 9. Files and tooling

| Path | Purpose |
|---|---|
| `src/cli.rs` | clap definitions. |
| `src/config.rs` | Config structs, defaults, template. |
| `src/display/mod.rs` | `DisplayOptions::build`: layer merging (section 4.1). |
| `src/display/{grid,long,tree,formatter}.rs` | Views and cell formatting. |
| `src/fs/walker.rs` | Directory walk, ignore matching, collapse, hidden counts. |
| `src/fs/filter.rs` | Post-filters. |
| `src/sort.rs` | Sorting. |
| `src/git.rs` | Git status map. |
| `src/theme/` | Icons and name colors. |
| `tests/` | Integration tests (binary run with an isolated `XDG_CONFIG_HOME`). |
| `cliff.toml`, `CHANGELOG.md` | Changelog generated by git-cliff from Conventional Commits (`make changelog`). |
| `.github/workflows/release.yml` | On a `v*` tag push: GitHub release with git-cliff notes. |

Releases: `main` holds tagged releases `vX.Y.Z`, `dev` is the integration branch. The Arch
package is built and published to the personal pacman repository `raf` with
`~/projects/update.sh raf elx`.
