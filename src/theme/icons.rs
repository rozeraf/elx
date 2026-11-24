use std::collections::HashMap;
use std::path::Path;

pub struct IconManager {
    icons_by_name: HashMap<String, String>,
    icons_by_extension: HashMap<String, String>,
}

impl IconManager {
    pub fn new() -> Self {
        let (icons_by_name, icons_by_extension) = Self::default_mappings();

        Self {
            icons_by_name,
            icons_by_extension,
        }
    }

    fn default_mappings() -> (HashMap<String, String>, HashMap<String, String>) {
        let mut icons_by_name = HashMap::new();
        let mut icons_by_extension = HashMap::new();

        // Common names
        icons_by_name.insert("cargo.toml".to_string(), "\u{e68b}".to_string());
        icons_by_name.insert("cargo.lock".to_string(), "\u{e68b}".to_string());
        icons_by_name.insert(".gitignore".to_string(), "\u{e725}".to_string());
        icons_by_name.insert("license".to_string(), "\u{e60a}".to_string());
        icons_by_name.insert("readme.md".to_string(), "\u{e609}".to_string());

        // Common extensions
        icons_by_extension.insert("rs".to_string(), "\u{e7a8}".to_string());
        icons_by_extension.insert("md".to_string(), "\u{e609}".to_string());
        icons_by_extension.insert("toml".to_string(), "\u{e615}".to_string());
        icons_by_extension.insert("txt".to_string(), "\u{f15c}".to_string());
        icons_by_extension.insert("json".to_string(), "\u{e60b}".to_string());
        icons_by_extension.insert("lock".to_string(), "\u{f023}".to_string());

        (icons_by_name, icons_by_extension)
    }

    pub fn get_icon(&self, path: &Path, is_dir: bool) -> String {
        if is_dir {
            return "\u{f115}".to_string(); // Default directory icon
        }

        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string().to_lowercase())
            .unwrap_or_default();

        if let Some(icon) = self.icons_by_name.get(&name) {
            return icon.clone();
        }

        let extension = path
            .extension()
            .map(|e| e.to_string_lossy().to_string().to_lowercase())
            .unwrap_or_default();

        if let Some(icon) = self.icons_by_extension.get(&extension) {
            return icon.clone();
        }

        "\u{f016}".to_string() // Default file icon
    }
}
