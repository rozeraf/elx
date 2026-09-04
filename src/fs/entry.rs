use std::fs::Metadata;
use std::path::PathBuf;

use crate::git::GitStatus;

#[derive(Debug)]
pub struct Entry {
    pub path: PathBuf,
    pub abs_path: PathBuf,
    pub metadata: Metadata,
    pub name: String,
    pub link_target: Option<PathBuf>,
    pub children: Option<Vec<Entry>>,
    pub git_status: Option<GitStatus>,
}

impl Entry {
    pub fn from_path(path: PathBuf) -> std::io::Result<Self> {
        let metadata = std::fs::symlink_metadata(&path)?;
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| path.display().to_string());

        // Keep symlinks as hyperlink targets instead of resolving them to their destination.
        let abs_path = std::path::absolute(&path).unwrap_or_else(|_| path.clone());

        let link_target = if metadata.file_type().is_symlink() {
            std::fs::read_link(&path).ok()
        } else {
            None
        };

        Ok(Self {
            path,
            abs_path,
            metadata,
            name,
            link_target,
            children: None,
            git_status: None,
        })
    }
}
