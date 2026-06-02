use ignore::WalkBuilder;
use std::path::Path;
use crate::fs::entry::Entry;
use anyhow::Result;

pub struct Walker {
    path: std::path::PathBuf,
    show_hidden: bool,
    use_git_ignore: bool,
    max_depth: usize,
}

impl Walker {
    pub fn new(path: &Path, show_hidden: bool, use_git_ignore: bool, max_depth: usize) -> Self {
        Self {
            path: path.to_path_buf(),
            show_hidden,
            use_git_ignore,
            max_depth,
        }
    }

    pub fn collect(&self) -> Result<Vec<Entry>> {
        self.collect_at(&self.path, 1)
    }

    fn collect_at(&self, path: &Path, current_depth: usize) -> Result<Vec<Entry>> {
        let mut entries = Vec::new();
        
        let walker = WalkBuilder::new(path)
            .hidden(!self.show_hidden)
            .git_ignore(self.use_git_ignore)
            .max_depth(Some(1))
            .build();

        for result in walker {
            match result {
                Ok(ignore_entry) => {
                    let entry_path = ignore_entry.path().to_path_buf();
                    
                    if entry_path == path {
                        continue;
                    }

                    if let Ok(mut entry) = Entry::from_path(entry_path.clone()) {
                        if entry.metadata.is_dir() && current_depth < self.max_depth {
                            if let Ok(children) = self.collect_at(&entry_path, current_depth + 1) {
                                entry.children = Some(children);
                            }
                        }
                        entries.push(entry);
                    }
                }
                Err(err) => eprintln!("Error: {}", err),
            }
        }

        // Sorting by name is done in main for now, but we can do it here for subdirectories too
        entries.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

        Ok(entries)
    }
}
