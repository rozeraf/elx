use std::path::{Path, PathBuf};
use std::collections::HashMap;
use gix::ThreadSafeRepository;
use gix::bstr::ByteSlice;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GitStatus {
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

pub fn get_statuses(repo_root: &Path) -> HashMap<PathBuf, GitStatus> {
    let mut statuses = HashMap::new();
    
    let repo = match ThreadSafeRepository::open(repo_root) {
        Ok(r) => r.to_thread_local(),
        Err(_) => return statuses,
    };

    let work_dir = match repo.work_dir() {
        Some(wd) => wd.to_path_buf(),
        None => return statuses,
    };

    let status_platform = match repo.status(gix::progress::Discard) {
        Ok(s) => s,
        Err(_) => return statuses,
    };

    let results = status_platform
        .into_iter(Vec::new())
        .ok();

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
                                    rela_path,
                                    status,
                                    ..
                                } => {
                                    let path = work_dir.join(gix::path::from_bstr(rela_path.as_bstr()));
                                    let git_status = match status {
                                        gix::status::plumbing::index_as_worktree::EntryStatus::Change(change) => {
                                            match change {
                                                gix::status::plumbing::index_as_worktree::Change::Removed => Some(GitStatus::Deleted),
                                                _ => Some(GitStatus::Modified),
                                            }
                                        }
                                        gix::status::plumbing::index_as_worktree::EntryStatus::Conflict(_) => Some(GitStatus::Modified),
                                        _ => None,
                                    };
                                    if let Some(s) = git_status {
                                        statuses.insert(path, s);
                                    }
                                }
                                WorktreeItem::DirectoryContents {
                                    entry,
                                    ..
                                } => {
                                    let path = work_dir.join(gix::path::from_bstr(entry.rela_path.as_bstr()));
                                    let git_status = match entry.status {
                                        gix::dir::entry::Status::Untracked => Some(GitStatus::Untracked),
                                        gix::dir::entry::Status::Ignored(_) => Some(GitStatus::Ignored),
                                        _ => None,
                                    };
                                    if let Some(s) = git_status {
                                        statuses.insert(path, s);
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
                            statuses.insert(path, git_status);
                        }
                    }
                }
                Err(_) => continue,
            }
        }
    }

    statuses
}
