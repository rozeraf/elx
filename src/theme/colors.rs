use crossterm::style::{Color, Stylize, StyledContent};
use std::fs::Metadata;

pub struct ColorManager;

impl ColorManager {
    pub fn new() -> Self {
        Self
    }

    pub fn colorize(&self, name: &str, metadata: &Metadata) -> StyledContent<String> {
        let file_type = metadata.file_type();
        
        if file_type.is_dir() {
            return name.to_string().with(Color::Blue).bold();
        }
        
        if file_type.is_symlink() {
            return name.to_string().with(Color::Cyan);
        }

        // Basic executable check for Unix
        #[cfg(unix)]
        {
            use std::os::unix::fs::MetadataExt;
            if metadata.mode() & 0o111 != 0 {
                return name.to_string().with(Color::Green).bold();
            }
        }

        name.to_string().with(Color::White)
    }
}
