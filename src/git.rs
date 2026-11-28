use gix::ThreadSafeRepository;
use gix::bstr::ByteSlice;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GitStatus {
    Conflict,
    Modified,
    Added,
    Deleted,
    Untracked,
    Ignored,
}

pub fn find_repo(path: &Path) -> Option<PathBuf> {
    gix::discover(path)
        .ok()
        .map(|repo| repo.path().to_path_buf())
}

/// Resolve parent directories, but never follow the final symlink itself.
pub fn status_path(path: &Path) -> PathBuf {
    let absolute = std::path::absolute(path).unwrap_or_else(|_| path.to_path_buf());
    match (absolute.parent(), absolute.file_name()) {
        (Some(parent), Some(name)) => parent
            .canonicalize()
            .map(|parent| parent.join(name))
            .unwrap_or(absolute),
        _ => absolute,
    }
}

pub fn status_for_path(statuses: &HashMap<PathBuf, GitStatus>, path: &Path) -> Option<GitStatus> {
    let path = status_path(path);
    statuses.get(&path).copied().or_else(|| {
        // gix can collapse ignored directories instead of enumerating their contents.
        path.ancestors().skip(1).find_map(|parent| {
            (statuses.get(parent) == Some(&GitStatus::Ignored)).then_some(GitStatus::Ignored)
        })
    })
}

/// `scope`: absolute paths to report on; git status skips everything else, which matters in big repos.
/// gix can report one path several times (index vs HEAD, worktree, dirwalk) in no fixed order;
/// keep the most important status so the result, and whether the entry is hidden, is stable.
fn insert_status(statuses: &mut HashMap<PathBuf, GitStatus>, path: PathBuf, status: GitStatus) {
    fn rank(status: GitStatus) -> u8 {
        match status {
            GitStatus::Conflict => 0,
            GitStatus::Deleted => 1,
            GitStatus::Modified => 2,
            GitStatus::Added => 3,
            GitStatus::Untracked => 4,
            GitStatus::Ignored => 5,
        }
    }
    statuses
        .entry(path)
        .and_modify(|existing| {
            if rank(status) < rank(*existing) {
                *existing = status;
            }
        })
        .or_insert(status);
}

pub fn get_statuses(
    repo_root: &Path,
    include_ignored: bool,
    scope: &[PathBuf],
) -> HashMap<PathBuf, GitStatus> {
    let mut statuses = HashMap::new();

    let repo = match ThreadSafeRepository::open(repo_root) {
        Ok(r) => r.to_thread_local(),
        Err(_) => return statuses,
    };

    let work_dir = match repo.work_dir() {
        Some(wd) => wd.canonicalize().unwrap_or_else(|_| wd.to_path_buf()),
        None => return statuses,
    };

    let status_platform = match repo.status(gix::progress::Discard) {
        Ok(s) => s.dirwalk_options(|options| {
            options
                .emit_untracked(gix::dir::walk::EmissionMode::Matching)
                .emit_ignored(
                    include_ignored.then_some(gix::dir::walk::EmissionMode::CollapseDirectory),
                )
        }),
        Err(_) => return statuses,
    };

    let mut patterns = Vec::new();
    for path in scope {
        use std::os::unix::ffi::OsStrExt;
        match status_path(path).strip_prefix(&work_dir) {
            Ok(rel) if !rel.as_os_str().is_empty() => {
                let mut pattern = gix::bstr::BString::from(":(top,literal)");
                pattern.extend_from_slice(rel.as_os_str().as_bytes());
                patterns.push(pattern);
            }
            // The work tree itself, or a path outside it: no restriction.
            _ => {
                patterns.clear();
                break;
            }
        }
    }
    let results = status_platform.into_iter(patterns).ok();

    if let Some(status_iter) = results {
        for entry in status_iter {
            match entry {
                Ok(item) => {
                    use gix::status::Item;
                    match item {
                        Item::IndexWorktree(it) => {
                            use gix::status::index_worktree::Item as WorktreeItem;
                            match it {
                                WorktreeItem::Modification {
                                    rela_path, status, ..
                                } => {
                                    let path =
                                        work_dir.join(gix::path::from_bstr(rela_path.as_bstr()));
                                    let git_status = match status {
                                        gix::status::plumbing::index_as_worktree::EntryStatus::Change(change) => {
                                            match change {
                                                gix::status::plumbing::index_as_worktree::Change::Removed => Some(GitStatus::Deleted),
                                                _ => Some(GitStatus::Modified),
                                            }
                                        }
                                        gix::status::plumbing::index_as_worktree::EntryStatus::Conflict(_) => Some(GitStatus::Conflict),
                                        _ => None,
                                    };
                                    if let Some(s) = git_status {
                                        insert_status(&mut statuses, path, s);
                                    }
                                }
                                WorktreeItem::DirectoryContents { entry, .. } => {
                                    let path = work_dir
                                        .join(gix::path::from_bstr(entry.rela_path.as_bstr()));
                                    let git_status = match entry.status {
                                        gix::dir::entry::Status::Untracked => {
                                            Some(GitStatus::Untracked)
                                        }
                                        gix::dir::entry::Status::Ignored(_) => {
                                            Some(GitStatus::Ignored)
                                        }
                                        _ => None,
                                    };
                                    if let Some(s) = git_status {
                                        insert_status(&mut statuses, path, s);
                                    }
                                }
                                _ => {}
                            }
                        }
                        Item::TreeIndex(change) => {
                            use gix::diff::index::Change;
                            let (rela_path, _, _, _) = change.fields();
                            let path = work_dir.join(gix::path::from_bstr(rela_path));
                            let git_status = match change {
                                Change::Addition { .. } => GitStatus::Added,
                                Change::Deletion { .. } => GitStatus::Deleted,
                                Change::Modification { .. } => GitStatus::Modified,
                                Change::Rewrite { .. } => GitStatus::Modified,
                            };
                            insert_status(&mut statuses, path, git_status);
                        }
                    }
                }
                Err(_) => continue,
            }
        }
    }

    let mut directories = HashMap::new();
    for (path, status) in &statuses {
        let summary = match status {
            GitStatus::Ignored => continue,
            GitStatus::Untracked => GitStatus::Untracked,
            _ => GitStatus::Modified,
        };
        for parent in path.ancestors().skip(1) {
            if !parent.starts_with(&work_dir) {
                break;
            }
            directories
                .entry(parent.to_path_buf())
                .and_modify(|existing| {
                    if summary == GitStatus::Modified {
                        *existing = summary;
                    }
                })
                .or_insert(summary);
        }
    }
    for (path, status) in directories {
        statuses.entry(path).or_insert(status);
    }
    statuses
}

#[cfg(test)]
mod tests {
    use super::{GitStatus, insert_status};
    use std::collections::HashMap;
    use std::path::PathBuf;

    #[test]
    fn the_most_important_status_wins_in_any_order() {
        let order = [GitStatus::Ignored, GitStatus::Deleted, GitStatus::Untracked];
        for rotation in 0..order.len() {
            let mut statuses = HashMap::new();
            for status in order.iter().cycle().skip(rotation).take(order.len()) {
                insert_status(&mut statuses, PathBuf::from("f"), *status);
            }
            assert_eq!(statuses[&PathBuf::from("f")], GitStatus::Deleted);
        }
    }
}
