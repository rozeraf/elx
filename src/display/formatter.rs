use crate::fs::entry::Entry;
use crate::theme::Theme;
use crate::display::DisplayOptions;
use crate::git::GitStatus;
use std::os::unix::fs::PermissionsExt;
use chrono::{DateTime, Local};
use users::{get_user_by_uid, get_group_by_gid};
use crossterm::style::{Color, Stylize};
use unicode_width::UnicodeWidthStr;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Column {
    Permissions,
    Links,
    Owner,
    Group,
    Size,
    Date,
    Name,
    Git,
}

pub fn wrap_hyperlink(uri: &str, text: &str) -> String {
    format!("\x1b]8;;{}\x1b\\{}\x1b]8;;\x1b\\", uri, text)
}

pub(crate) fn should_hyperlink(options: &DisplayOptions, entry: &Entry) -> bool {
    if !options.hyperlinks.enabled {
        return false;
    }
    let ft = entry.metadata.file_type();
    if ft.is_dir() {
        return options.hyperlinks.dirs;
    }
    if ft.is_symlink() {
        return options.hyperlinks.symlinks;
    }
    // Regular file — check if executable
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        if entry.metadata.mode() & 0o111 != 0 {
            return options.hyperlinks.executables;
        }
    }
    options.hyperlinks.files
}

pub struct Cell {
    pub content: String,
    pub width: usize,
}

pub struct ColumnFormatter<'a> {
    pub theme: &'a Theme,
    pub options: &'a DisplayOptions,
}

impl<'a> ColumnFormatter<'a> {
    pub fn format_column(&self, col: Column, entry: &Entry) -> Cell {
        let metadata = &entry.metadata;
        
        match col {
            Column::Permissions => {
                let mode = metadata.permissions().mode();
                let content = if !self.options.color {
                    format_permissions(mode, metadata.is_dir())
                } else {
                    format_permissions_colored(mode, metadata.is_dir())
                };
                Cell { content, width: 10 }
            }
            Column::Links => {
                #[cfg(unix)]
                let nlink = {
                    use std::os::unix::fs::MetadataExt;
                    metadata.nlink()
                };
                #[cfg(not(unix))]
                let nlink = 1;
                
                let content = nlink.to_string();
                let width = content.len();
                Cell { content, width }
            }
            Column::Owner => {
                #[cfg(unix)]
                let uid = {
                    use std::os::unix::fs::MetadataExt;
                    metadata.uid()
                };
                #[cfg(not(unix))]
                let uid = 0;

                let owner_name = get_user_by_uid(uid)
                    .map(|u| u.name().to_string_lossy().into_owned())
                    .unwrap_or_else(|| uid.to_string());
                
                let width = owner_name.len();
                let content = if !self.options.color {
                    owner_name
                } else {
                    owner_name.with(Color::Yellow).to_string()
                };
                Cell { content, width }
            }
            Column::Group => {
                #[cfg(unix)]
                let gid = {
                    use std::os::unix::fs::MetadataExt;
                    metadata.gid()
                };
                #[cfg(not(unix))]
                let gid = 0;

                let group_name = get_group_by_gid(gid)
                    .map(|g| g.name().to_string_lossy().into_owned())
                    .unwrap_or_else(|| gid.to_string());
                
                let width = group_name.len();
                let content = if !self.options.color {
                    group_name
                } else {
                    group_name.with(Color::Yellow).to_string()
                };
                Cell { content, width }
            }
            Column::Size => {
                let size_text = format_size(metadata.len());
                let width = size_text.len();
                let content = if !self.options.color {
                    size_text
                } else {
                    size_text.with(Color::Green).to_string()
                };
                Cell { content, width }
            }
            Column::Date => {
                let modified: DateTime<Local> = metadata.modified()
                    .map(|t| t.into())
                    .unwrap_or_else(|_| Local::now());
                let date_str = modified.format("%b %d %H:%M").to_string();
                let width = date_str.len();
                let content = if !self.options.color {
                    date_str
                } else {
                    date_str.with(Color::Cyan).to_string()
                };
                Cell { content, width }
            }
            Column::Git => {
                let (symbol, color) = match entry.git_status {
                    Some(GitStatus::Modified) => ("M", Color::Yellow),
                    Some(GitStatus::Added) => ("A", Color::Green),
                    Some(GitStatus::Deleted) => ("D", Color::Red),
                    Some(GitStatus::Untracked) => ("?", Color::Blue),
                    Some(GitStatus::Ignored) => ("!", Color::Grey),
                    _ => (" ", Color::Reset),
                };
                
                let content = if self.options.color && color != Color::Reset {
                    symbol.with(color).to_string()
                } else {
                    symbol.to_string()
                };
                Cell { content, width: 1 }
            }
            Column::Name => {
                let icon_str = if !self.options.icons {
                    "".to_string()
                } else {
                    self.theme.icons.get_icon(&entry.path, entry.metadata.is_dir())
                };

                let icon_width = if icon_str.is_empty() { 0 } else { icon_str.width() + 1 };
                let icon = if icon_str.is_empty() {
                    "".to_string()
                } else {
                    format!("{} ", icon_str)
                };

                let name = if !self.options.color {
                    entry.name.clone()
                } else {
                    self.theme.colors.colorize(&entry.name, &entry.metadata).to_string()
                };

                let suffix = if self.options.classify && entry.metadata.is_dir() { "/" } else { "" };
                let suffix_width = suffix.len();

                let target = if let Some(t) = &entry.link_target {
                    let target_name = t.display().to_string();
                    if !self.options.color {
                        format!(" -> {}", target_name)
                    } else {
                        format!(" -> {}", target_name.with(Color::Cyan))
                    }
                } else {
                    "".to_string()
                };

                let target_width = if let Some(t) = &entry.link_target {
                    t.display().to_string().width() + 4
                } else {
                    0
                };

                let content = format!("{}{}{}{}", icon, name, suffix, target);

                let content = if should_hyperlink(self.options, entry) {
                    let uri = format!("file://{}", entry.abs_path.display());
                    wrap_hyperlink(&uri, &content)
                } else {
                    content
                };

                Cell {
                    content,
                    width: icon_width + entry.name.width() + suffix_width + target_width,
                }
            }
        }
    }
}

fn format_permissions(mode: u32, is_dir: bool) -> String {
    let mut s = String::with_capacity(10);
    s.push(if is_dir { 'd' } else { '-' });
    let chars = ['r', 'w', 'x'];
    for i in (0..3).rev() {
        let bits = (mode >> (i * 3)) & 0o7;
        s.push(if bits & 4 != 0 { chars[0] } else { '-' });
        s.push(if bits & 2 != 0 { chars[1] } else { '-' });
        s.push(if bits & 1 != 0 { chars[2] } else { '-' });
    }
    s
}

fn format_permissions_colored(mode: u32, is_dir: bool) -> String {
    let mut s = String::new();
    
    if is_dir {
        s.push_str(&"d".with(Color::Blue).to_string());
    } else {
        s.push_str(&"-".with(Color::Grey).to_string());
    }

    let chars = [('r', Color::Yellow), ('w', Color::Red), ('x', Color::Green)];
    for i in (0..3).rev() {
        let bits = (mode >> (i * 3)) & 0o7;
        
        if bits & 4 != 0 {
            s.push_str(&chars[0].0.to_string().with(chars[0].1).to_string());
        } else {
            s.push_str(&"-".with(Color::Grey).to_string());
        }

        if bits & 2 != 0 {
            s.push_str(&chars[1].0.to_string().with(chars[1].1).to_string());
        } else {
            s.push_str(&"-".with(Color::Grey).to_string());
        }

        if bits & 1 != 0 {
            s.push_str(&chars[2].0.to_string().with(chars[2].1).to_string());
        } else {
            s.push_str(&"-".with(Color::Grey).to_string());
        }
    }
    s
}

fn format_size(size: u64) -> String {
    const UNITS: &[&str] = &["B", "K", "M", "G", "T"];
    let mut size = size as f64;
    let mut unit_idx = 0;
    while size >= 1024.0 && unit_idx < UNITS.len() - 1 {
        size /= 1024.0;
        unit_idx += 1;
    }
    if unit_idx == 0 {
        format!("{:.0} {}", size, UNITS[unit_idx])
    } else {
        format!("{:.1} {}", size, UNITS[unit_idx])
    }
}
