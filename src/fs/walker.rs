use crate::fs::entry::Entry;
use crate::git::{GitStatus, find_repo, get_statuses, status_for_path};
use anyhow::Result;
use ignore::WalkBuilder;
use ignore::overrides::OverrideBuilder;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub struct Walker {
    path: PathBuf,
    show_hidden: bool,
    use_git_ignore: bool,
    max_depth: usize,
    ignore_globs: Vec<String>,
}

impl Walker {
    pub fn new(
        path: &Path,
        show_hidden: bool,
        use_git_ignore: bool,
        max_depth: usize,
        ignore_globs: Vec<String>,
    ) -> Self {
        Self {
            path: path.to_path_buf(),
            show_hidden,
            use_git_ignore,
            max_depth,
            ignore_globs,
        }
    }

    pub fn collect(&self) -> Result<Vec<Entry>> {
        let abs_root = self.path.canonicalize().unwrap_or(self.path.clone());
        let git_statuses = if let Some(repo_root) = find_repo(&abs_root) {
            get_statuses(&repo_root, !self.use_git_ignore)
        } else {
            HashMap::new()
        };

        self.collect_at(&self.path, 1, &git_statuses)
    }

    fn collect_at(
        &self,
        path: &Path,
        current_depth: usize,
        git_statuses: &HashMap<PathBuf, GitStatus>,
    ) -> Result<Vec<Entry>> {
        let mut entries = Vec::new();

        let mut override_builder = OverrideBuilder::new(path);
        for glob in &self.ignore_globs {
            override_builder.add(&format!("!{}", glob))?;
        }
        let overrides = override_builder.build()?;

        let walker = WalkBuilder::new(path)
            .hidden(!self.show_hidden)
            .git_ignore(self.use_git_ignore)
            .overrides(overrides)
            .max_depth(Some(1))
            .build();

        for result in walker {
            let ignore_entry = result?;
            let entry_path = ignore_entry.path().to_path_buf();

            if entry_path == path {
                continue;
            }

            let mut entry = Entry::from_path(entry_path.clone())?;
            entry.git_status = status_for_path(git_statuses, &entry.abs_path);

            if entry.metadata.is_dir() && current_depth < self.max_depth {
                entry.children =
                    Some(self.collect_at(&entry_path, current_depth + 1, git_statuses)?);
            }
            entries.push(entry);
        }

        entries.sort_by_key(|entry| entry.name.to_lowercase());

        Ok(entries)
    }
}
