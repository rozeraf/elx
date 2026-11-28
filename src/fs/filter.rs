use crate::fs::entry::Entry;
use crate::git::GitStatus;
use ignore::gitignore::Gitignore;

/// Filters applied after the walk. A directory with loaded children is kept when it matches itself
/// or still has children left after filtering, so the tree keeps the path to every match.
pub struct EntryFilter {
    pub changed: bool,
    /// Matched against files only, or against directories with `dirs_only`.
    pub only: Option<Gitignore>,
    pub dirs_only: bool,
    pub files_only: bool,
}

impl EntryFilter {
    fn is_active(&self) -> bool {
        self.changed || self.only.is_some() || self.dirs_only || self.files_only
    }

    fn matches(&self, entry: &Entry) -> bool {
        let is_dir = entry.metadata.is_dir();
        let type_ok = if self.dirs_only {
            is_dir
        } else if self.files_only {
            !is_dir
        } else {
            true
        };
        let only_ok = self.only.as_ref().is_none_or(|only| {
            (!is_dir || self.dirs_only) && only.matched(&entry.path, is_dir).is_ignore()
        });
        let changed_ok = !self.changed
            || matches!(
                entry.git_status,
                Some(
                    GitStatus::Conflict
                        | GitStatus::Modified
                        | GitStatus::Added
                        | GitStatus::Deleted
                        | GitStatus::Untracked
                )
            );
        type_ok && only_ok && changed_ok
    }

    pub fn apply(&self, entries: Vec<Entry>) -> Vec<Entry> {
        if !self.is_active() {
            return entries;
        }
        entries
            .into_iter()
            .filter_map(|mut entry| match entry.children.take() {
                Some(children) => {
                    let kept = self.apply(children);
                    let keep = self.matches(&entry) || !kept.is_empty();
                    entry.children = Some(kept);
                    keep.then_some(entry)
                }
                None => self.matches(&entry).then_some(entry),
            })
            .collect()
    }
}
