use std::path::PathBuf;
use std::fs::Metadata;

#[derive(Debug)]
pub struct Entry {
    #[allow(dead_code)]
    pub path: PathBuf,
    #[allow(dead_code)]
    pub metadata: Metadata,
    pub name: String,
}

impl Entry {
    pub fn from_path(path: PathBuf) -> std::io::Result<Self> {
        let metadata = std::fs::symlink_metadata(&path)?;
        let name = path.file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| path.display().to_string());
        
        Ok(Self {
            path,
            metadata,
            name,
        })
    }
}
