use crate::config::GitIgnoreMode;
use crate::fs::entry::Entry;
use crate::git::{GitStatus, find_repo, get_statuses, status_for_path};
use anyhow::Result;
use ignore::WalkBuilder;
use ignore::gitignore::{Gitignore, GitignoreBuilder};
use std::collections::{HashMap, HashSet};
use std::ffi::OsString;
use std::path::{Path, PathBuf};

pub struct WalkOptions {
    pub show_hidden: bool,
    pub git_ignore: GitIgnoreMode,
    /// Honor `.ignore` and `.elxignore` files.
    pub ignore_files: bool,
    pub max_depth: usize,
    pub ignore_globs: Vec<String>,
    /// The part of `ignore_globs` given on the command line, only used to explain hidden entries.
    pub cli_ignore_globs: Vec<String>,
    pub collapse_globs: Vec<String>,
    /// Count entries the walk skipped (costs one extra `read_dir` per directory).
    pub count_hidden: bool,
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct HiddenCounts {
    pub dotfiles: usize,
    /// Reported as ignored by `git status`.
    pub gitignored: usize,
    /// Config or `[all]`/`[tree]` patterns.
    pub pattern: usize,
    /// `--ignore` on the command line.
    pub cli_pattern: usize,
    /// `.elxignore`/`.ignore` files, or tracked files matched by `.gitignore`.
    pub ignore_files: usize,
}

impl HiddenCounts {
    pub fn total(&self) -> usize {
        self.dotfiles + self.gitignored + self.pattern + self.cli_pattern + self.ignore_files
    }
}

#[derive(Debug, Default)]
pub struct WalkStats {
    pub hidden: HiddenCounts,
    /// Entries or directories that could not be read; already reported on stderr.
    pub errors: usize,
}

pub struct Walker {
    path: PathBuf,
    opts: WalkOptions,
}

struct Matchers {
    ignore: Gitignore,
    cli_ignore: Gitignore,
    collapse: Gitignore,
}

/// Gitignore semantics: patterns are relative to `root`, later lines win, `!pattern` re-includes.
pub fn build_matcher(root: &Path, globs: &[String]) -> Result<Gitignore> {
    let mut builder = GitignoreBuilder::new(root);
    for glob in globs {
        builder.add_line(None, glob)?;
    }
    Ok(builder.build()?)
}

impl Walker {
    pub fn new(path: &Path, opts: WalkOptions) -> Self {
        Self {
            path: path.to_path_buf(),
            opts,
        }
    }

    pub fn collect(&self) -> Result<(Vec<Entry>, WalkStats)> {
        let abs_root = self.path.canonicalize().unwrap_or(self.path.clone());
        let git_statuses = if let Some(repo_root) = find_repo(&abs_root) {
            // Ignored statuses also tell gitignored entries apart when counting hidden ones.
            let include_ignored =
                self.opts.git_ignore != GitIgnoreMode::Hide || self.opts.count_hidden;
            get_statuses(&repo_root, include_ignored)
        } else {
            HashMap::new()
        };

        let matchers = Matchers {
            ignore: build_matcher(&self.path, &self.opts.ignore_globs)?,
            cli_ignore: build_matcher(&self.path, &self.opts.cli_ignore_globs)?,
            collapse: build_matcher(&self.path, &self.opts.collapse_globs)?,
        };

        let mut stats = WalkStats::default();
        let entries = self.collect_at(&self.path, 1, &git_statuses, &matchers, &mut stats);
        Ok((entries, stats))
    }

    fn count_hidden(
        &self,
        path: &Path,
        shown: &[Entry],
        git_statuses: &HashMap<PathBuf, GitStatus>,
        matchers: &Matchers,
        hidden: &mut HiddenCounts,
    ) {
        let Ok(dir) = std::fs::read_dir(path) else {
            return;
        };
        let shown: HashSet<OsString> = shown
            .iter()
            .filter_map(|e| e.path.file_name().map(OsString::from))
            .collect();
        for dirent in dir.flatten() {
            let name = dirent.file_name();
            if shown.contains(&name) {
                continue;
            }
            let entry_path = dirent.path();
            let is_dir = dirent.file_type().is_ok_and(|t| t.is_dir());
            if !self.opts.show_hidden && name.as_encoded_bytes().starts_with(b".") {
                hidden.dotfiles += 1;
            } else if status_for_path(git_statuses, &entry_path) == Some(GitStatus::Ignored) {
                hidden.gitignored += 1;
            } else if matchers.cli_ignore.matched(&entry_path, is_dir).is_ignore() {
                hidden.cli_pattern += 1;
            } else if matchers.ignore.matched(&entry_path, is_dir).is_ignore() {
                hidden.pattern += 1;
            } else {
                hidden.ignore_files += 1;
            }
        }
    }

    fn collect_at(
        &self,
        path: &Path,
        current_depth: usize,
        git_statuses: &HashMap<PathBuf, GitStatus>,
        matchers: &Matchers,
        stats: &mut WalkStats,
    ) -> Vec<Entry> {
        let mut entries = Vec::new();

        let hide_git_ignored = self.opts.git_ignore == GitIgnoreMode::Hide;
        let ignore = matchers.ignore.clone();
        let mut builder = WalkBuilder::new(path);
        builder
            .hidden(!self.opts.show_hidden)
            .git_ignore(hide_git_ignored)
            .git_exclude(hide_git_ignored)
            .git_global(hide_git_ignored)
            .ignore(self.opts.ignore_files)
            .max_depth(Some(1))
            .filter_entry(move |e| {
                let is_dir = e.file_type().is_some_and(|t| t.is_dir());
                !ignore.matched(e.path(), is_dir).is_ignore()
            });
        if self.opts.ignore_files {
            builder.add_custom_ignore_filename(".elxignore");
        }

        // Like ls: report unreadable entries and keep listing the rest.
        for result in builder.build() {
            let ignore_entry = match result {
                Ok(e) => e,
                Err(err) => {
                    eprintln!("elx: {err}");
                    stats.errors += 1;
                    continue;
                }
            };
            let entry_path = ignore_entry.path().to_path_buf();

            if entry_path == path {
                continue;
            }

            let mut entry = match Entry::from_path(entry_path.clone()) {
                Ok(e) => e,
                Err(err) => {
                    eprintln!("elx: {}: {err}", entry_path.display());
                    stats.errors += 1;
                    continue;
                }
            };
            entry.git_status = status_for_path(git_statuses, &entry.abs_path);

            if entry.metadata.is_dir() && current_depth < self.opts.max_depth {
                if matchers.collapse.matched(&entry_path, true).is_ignore() {
                    entry.collapsed =
                        Some(std::fs::read_dir(&entry_path).map_or(0, |dir| dir.count()));
                } else {
                    entry.children = Some(self.collect_at(
                        &entry_path,
                        current_depth + 1,
                        git_statuses,
                        matchers,
                        stats,
                    ));
                }
            }
            entries.push(entry);
        }

        if self.opts.count_hidden {
            self.count_hidden(path, &entries, git_statuses, matchers, &mut stats.hidden);
        }
        entries
    }
}

#[cfg(test)]
mod tests {
    use super::{HiddenCounts, WalkOptions, Walker};
    use crate::config::GitIgnoreMode;
    use std::process::Command;

    fn walk(root: &std::path::Path, git_ignore: GitIgnoreMode, show_hidden: bool) -> HiddenCounts {
        let walker = Walker::new(
            root,
            WalkOptions {
                show_hidden,
                git_ignore,
                ignore_files: true,
                max_depth: usize::MAX,
                ignore_globs: vec!["*.log".into(), "*.tmp".into()],
                cli_ignore_globs: vec!["*.tmp".into()],
                collapse_globs: vec!["collapsed".into()],
                count_hidden: true,
            },
        );
        walker.collect().unwrap().1.hidden
    }

    #[test]
    fn counts_hidden_entries_by_reason_across_the_tree() {
        let root = std::env::temp_dir().join(format!("elx-walker-test-{}", std::process::id()));
        for (file, content) in [
            (".gitignore", "built\n"),
            (".elxignore", "secret\n"),
            ("built", ""),
            ("a.log", ""),
            ("secret", ""),
            ("visible", ""),
            ("x.tmp", ""),
            ("sub/.hidden", ""),
            ("sub/b.log", ""),
            ("collapsed/.x", ""),
            ("collapsed/c.log", ""),
        ] {
            let path = root.join(file);
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(path, content).unwrap();
        }
        let status = Command::new("git")
            .args(["init", "-q"])
            .current_dir(&root)
            .env("GIT_CONFIG_GLOBAL", "/dev/null")
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .status()
            .unwrap();
        assert!(status.success());

        let hide = walk(&root, GitIgnoreMode::Hide, false);
        let dim = walk(&root, GitIgnoreMode::Dim, false);
        let all = walk(&root, GitIgnoreMode::Hide, true);
        std::fs::remove_dir_all(&root).unwrap();

        // Dotfiles: .git, .gitignore, .elxignore, sub/.hidden. Collapsed dirs are not read.
        assert_eq!(
            hide,
            HiddenCounts {
                dotfiles: 4,
                gitignored: 1,
                pattern: 2,
                cli_pattern: 1,
                ignore_files: 1,
            }
        );
        assert_eq!(dim.gitignored, 0);
        assert_eq!(all.dotfiles, 0);
        // .git is not gitignored, so with -a it only stays hidden if a pattern matches; it does not.
        assert_eq!(all.gitignored, 1);
    }
}
