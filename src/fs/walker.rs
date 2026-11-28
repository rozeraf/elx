use crate::config::GitIgnoreMode;
use crate::fs::entry::Entry;
use crate::git::{GitStatus, find_repo, get_statuses, status_for_path};
use anyhow::Result;
use ignore::WalkBuilder;
use ignore::gitignore::{Gitignore, GitignoreBuilder};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub struct WalkOptions {
    pub show_hidden: bool,
    pub git_ignore: GitIgnoreMode,
    /// Honor `.ignore` and `.elxignore` files.
    pub ignore_files: bool,
    pub max_depth: usize,
    pub ignore_globs: Vec<String>,
    pub collapse_globs: Vec<String>,
}

pub struct Walker {
    path: PathBuf,
    opts: WalkOptions,
}

struct Matchers {
    ignore: Gitignore,
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

    pub fn collect(&self) -> Result<Vec<Entry>> {
        let abs_root = self.path.canonicalize().unwrap_or(self.path.clone());
        let git_statuses = if let Some(repo_root) = find_repo(&abs_root) {
            get_statuses(&repo_root, self.opts.git_ignore != GitIgnoreMode::Hide)
        } else {
            HashMap::new()
        };

        let matchers = Matchers {
            ignore: build_matcher(&self.path, &self.opts.ignore_globs)?,
            collapse: build_matcher(&self.path, &self.opts.collapse_globs)?,
        };

        self.collect_at(&self.path, 1, &git_statuses, &matchers)
    }

    fn collect_at(
        &self,
        path: &Path,
        current_depth: usize,
        git_statuses: &HashMap<PathBuf, GitStatus>,
        matchers: &Matchers,
    ) -> Result<Vec<Entry>> {
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

        for result in builder.build() {
            let ignore_entry = result?;
            let entry_path = ignore_entry.path().to_path_buf();

            if entry_path == path {
                continue;
            }

            let mut entry = Entry::from_path(entry_path.clone())?;
            entry.git_status = status_for_path(git_statuses, &entry.abs_path);

            if entry.metadata.is_dir() && current_depth < self.opts.max_depth {
                if matchers.collapse.matched(&entry_path, true).is_ignore() {
                    entry.collapsed = true;
                } else {
                    entry.children = Some(self.collect_at(
                        &entry_path,
                        current_depth + 1,
                        git_statuses,
                        matchers,
                    )?);
                }
            }
            entries.push(entry);
        }

        Ok(entries)
    }
}
