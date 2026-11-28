use crate::config::{GroupDirs, SortConfig, SortKey};
use crate::fs::entry::Entry;

#[derive(Debug, Clone)]
pub struct SortOptions {
    pub by: SortKey,
    pub secondary_by: Option<SortKey>,
    pub ignore_case: bool,
    pub reverse: bool,
    pub group_dirs: GroupDirs,
    pub dotfiles_first: bool,
}

impl From<&SortConfig> for SortOptions {
    fn from(c: &SortConfig) -> Self {
        Self {
            by: c.by,
            secondary_by: c.secondary_by,
            ignore_case: c.ignore_case,
            reverse: c.reverse,
            group_dirs: c.group_dirs,
            dotfiles_first: c.dotfiles_first,
        }
    }
}

fn natural_cmp(a: &str, b: &str, ignore_case: bool) -> std::cmp::Ordering {
    use std::cmp::Ordering;

    let a_norm: std::borrow::Cow<str> = if ignore_case {
        a.to_lowercase().into()
    } else {
        a.into()
    };
    let b_norm: std::borrow::Cow<str> = if ignore_case {
        b.to_lowercase().into()
    } else {
        b.into()
    };

    let mut a_chars = a_norm.chars().peekable();
    let mut b_chars = b_norm.chars().peekable();

    loop {
        match (a_chars.peek().copied(), b_chars.peek().copied()) {
            (None, None) => return Ordering::Equal,
            (None, _) => return Ordering::Less,
            (_, None) => return Ordering::Greater,
            (Some(ac), Some(bc)) => {
                if ac.is_ascii_digit() && bc.is_ascii_digit() {
                    let an = collect_digits(&mut a_chars);
                    let bn = collect_digits(&mut b_chars);
                    match an.cmp(&bn) {
                        Ordering::Equal => continue,
                        other => return other,
                    }
                } else {
                    a_chars.next();
                    b_chars.next();
                    match ac.cmp(&bc) {
                        Ordering::Equal => continue,
                        other => return other,
                    }
                }
            }
        }
    }
}

fn collect_digits<I: Iterator<Item = char>>(iter: &mut std::iter::Peekable<I>) -> u64 {
    let mut n = 0u64;
    while let Some(&c) = iter.peek() {
        if c.is_ascii_digit() {
            n = n.saturating_mul(10).saturating_add(c as u64 - '0' as u64);
            iter.next();
        } else {
            break;
        }
    }
    n
}

fn git_priority(status: &Option<crate::git::GitStatus>) -> u8 {
    use crate::git::GitStatus;
    match status {
        Some(GitStatus::Conflict) => 0,
        Some(GitStatus::Modified) => 1,
        Some(GitStatus::Added) => 2,
        Some(GitStatus::Deleted) => 3,
        Some(GitStatus::Untracked) => 4,
        Some(GitStatus::Ignored) => 5,
        None => 6,
    }
}

/// Single key only; `compare_entries` applies grouping and reverse.
fn cmp_by_key(a: &Entry, b: &Entry, key: SortKey, ignore_case: bool) -> std::cmp::Ordering {
    use std::cmp::Ordering;

    match key {
        SortKey::None => Ordering::Equal,

        SortKey::Name => {
            if ignore_case {
                a.name.to_lowercase().cmp(&b.name.to_lowercase())
            } else {
                a.name.cmp(&b.name)
            }
        }

        SortKey::NameNatural => natural_cmp(&a.name, &b.name, ignore_case),

        SortKey::Extension => {
            let ext = |e: &Entry| {
                let raw = e
                    .path
                    .extension()
                    .map(|x| x.to_string_lossy().into_owned())
                    .unwrap_or_default();
                if ignore_case { raw.to_lowercase() } else { raw }
            };
            ext(a).cmp(&ext(b)).then_with(|| {
                if ignore_case {
                    a.name.to_lowercase().cmp(&b.name.to_lowercase())
                } else {
                    a.name.cmp(&b.name)
                }
            })
        }

        SortKey::Size => a.metadata.len().cmp(&b.metadata.len()),

        SortKey::Modified => a.metadata.modified().ok().cmp(&b.metadata.modified().ok()),

        SortKey::Created => a.metadata.created().ok().cmp(&b.metadata.created().ok()),

        SortKey::GitStatus => git_priority(&a.git_status).cmp(&git_priority(&b.git_status)),
    }
}

fn is_dotfile(entry: &Entry) -> bool {
    entry.name.starts_with('.')
}

fn compare_entries(a: &Entry, b: &Entry, opts: &SortOptions) -> std::cmp::Ordering {
    use std::cmp::Ordering;

    // Grouping (dirs, dotfiles) is never affected by reverse.
    if opts.group_dirs != GroupDirs::Mixed {
        let a_dir = a.metadata.is_dir() || a.links_to_dir;
        let b_dir = b.metadata.is_dir() || b.links_to_dir;
        if a_dir != b_dir {
            return match opts.group_dirs {
                GroupDirs::First => {
                    if a_dir {
                        Ordering::Less
                    } else {
                        Ordering::Greater
                    }
                }
                GroupDirs::Last => {
                    if a_dir {
                        Ordering::Greater
                    } else {
                        Ordering::Less
                    }
                }
                GroupDirs::Mixed => unreachable!(),
            };
        }
    }

    if opts.dotfiles_first {
        let a_dot = is_dotfile(a);
        let b_dot = is_dotfile(b);
        if a_dot != b_dot {
            return if a_dot {
                Ordering::Less
            } else {
                Ordering::Greater
            };
        }
    }

    let primary = cmp_by_key(a, b, opts.by, opts.ignore_case);
    let primary = if opts.reverse {
        primary.reverse()
    } else {
        primary
    };

    if primary != Ordering::Equal {
        return primary;
    }

    if let Some(secondary_key) = opts.secondary_by {
        let secondary = cmp_by_key(a, b, secondary_key, opts.ignore_case);
        return if opts.reverse {
            secondary.reverse()
        } else {
            secondary
        };
    }

    Ordering::Equal
}

pub fn sort_entries(entries: &mut [Entry], opts: &SortOptions) {
    if matches!(opts.by, SortKey::None)
        && opts.group_dirs == GroupDirs::Mixed
        && !opts.dotfiles_first
    {
        return;
    }

    entries.sort_by(|a, b| compare_entries(a, b, opts));

    for entry in entries.iter_mut() {
        if let Some(children) = &mut entry.children {
            sort_entries(children, opts);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{SortOptions, sort_entries};
    use crate::config::SortConfig;
    use crate::fs::entry::Entry;

    #[test]
    fn symlinks_to_directories_group_with_directories() {
        let root = std::env::temp_dir().join(format!("elx-sort-test-{}", std::process::id()));
        std::fs::create_dir_all(root.join("zdir")).unwrap();
        std::fs::write(root.join("afile"), "").unwrap();
        std::os::unix::fs::symlink("zdir", root.join("blink")).unwrap();
        std::os::unix::fs::symlink("afile", root.join("cfilelink")).unwrap();
        let mut entries: Vec<Entry> = ["afile", "blink", "cfilelink", "zdir"]
            .iter()
            .map(|name| Entry::from_path(root.join(name)).unwrap())
            .collect();
        sort_entries(&mut entries, &SortOptions::from(&SortConfig::default()));
        std::fs::remove_dir_all(&root).unwrap();
        let names: Vec<&str> = entries.iter().map(|e| e.name.as_str()).collect();
        assert_eq!(names, ["blink", "zdir", "afile", "cfilelink"]);
    }
}
