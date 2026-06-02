use ignore::WalkBuilder;
use std::path::Path;
use crate::fs::entry::Entry;
use anyhow::Result;

pub struct Walker {
    path: std::path::PathBuf,
    show_hidden: bool,
}

impl Walker {
    pub fn new(path: &Path, show_hidden: bool) -> Self {
        Self {
            path: path.to_path_buf(),
            show_hidden,
        }
    }

    pub fn collect(&self) -> Result<Vec<Entry>> {
        let mut entries = Vec::new();
        
        let walker = WalkBuilder::new(&self.path)
            .hidden(!self.show_hidden)
            .git_ignore(true)
            .max_depth(Some(1))
            .build();

        for result in walker {
            match result {
                Ok(ignore_entry) => {
                    let path = ignore_entry.path().to_path_buf();
                    
                    if path == self.path {
                        continue;
                    }

                    if let Ok(entry) = Entry::from_path(path) {
                        entries.push(entry);
                    }
                }
                Err(err) => eprintln!("Error: {}", err),
            }
        }

        Ok(entries)
    }
}
