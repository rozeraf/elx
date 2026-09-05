use crate::display::DisplayOptions;
use crate::fs::entry::Entry;
use crate::git::GitStatus;
use crate::theme::Theme;
use chrono::{DateTime, Local};
use crossterm::style::{Color, Stylize};
use serde::{Deserialize, Serialize};
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use unicode_width::UnicodeWidthStr;
use users::{get_group_by_gid, get_user_by_uid};

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

/// Escape terminal controls and bidirectional formatting without changing filesystem paths.
pub fn escape_terminal_text(text: &str) -> String {
    let mut escaped = String::new();
    for ch in text.chars() {
        if ch.is_control()
            || matches!(ch, '\u{061c}' | '\u{200e}' | '\u{200f}' | '\u{202a}'..='\u{202e}' | '\u{2066}'..='\u{2069}')
        {
            escaped.extend(ch.escape_default());
        } else if ch == '\\' {
            escaped.push_str("\\\\");
        } else {
            escaped.push(ch);
        }
    }
    escaped
}

pub fn wrap_hyperlink(uri: &str, text: &str) -> String {
    format!("\x1b]8;;{}\x1b\\{}\x1b]8;;\x1b\\", uri, text)
}

pub fn file_uri(path: &Path) -> String {
    #[cfg(unix)]
    let bytes = {
        use std::os::unix::ffi::OsStrExt;
        path.as_os_str().as_bytes()
    };
    #[cfg(not(unix))]
    let bytes = path.to_string_lossy().as_bytes();

    let mut uri = String::from("file://");
    for byte in bytes {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' | b'/' => {
                uri.push(*byte as char);
            }
            _ => uri.push_str(&format!("%{byte:02X}")),
        }
    }
    uri
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
                let modified: DateTime<Local> = metadata
                    .modified()
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
                    self.theme
                        .icons
                        .get_icon(&entry.path, entry.metadata.is_dir())
                };

                let icon_width = if icon_str.is_empty() {
                    0
                } else {
                    icon_str.width() + 1
                };
                let icon = if icon_str.is_empty() {
                    "".to_string()
                } else {
                    format!("{} ", icon_str)
                };

                let safe_name = escape_terminal_text(&entry.name);
                let name = if !self.options.color {
                    safe_name.clone()
                } else {
                    self.theme
                        .colors
                        .colorize(&safe_name, &entry.metadata)
                        .to_string()
                };

                let suffix = if self.options.classify && entry.metadata.is_dir() {
                    "/"
                } else {
                    ""
                };
                let suffix_width = suffix.len();

                let target = if let Some(t) = &entry.link_target {
                    let target_name = escape_terminal_text(&t.to_string_lossy());
                    if !self.options.color {
                        format!(" -> {}", target_name)
                    } else {
                        format!(" -> {}", target_name.with(Color::Cyan))
                    }
                } else {
                    "".to_string()
                };

                let target_width = if let Some(t) = &entry.link_target {
                    escape_terminal_text(&t.to_string_lossy()).width() + 4
                } else {
                    0
                };

                let content = format!("{}{}{}{}", icon, name, suffix, target);

                let content = if should_hyperlink(self.options, entry) {
                    let uri = file_uri(&entry.abs_path);
                    wrap_hyperlink(&uri, &content)
                } else {
                    content
                };

                Cell {
                    content,
                    width: icon_width + safe_name.width() + suffix_width + target_width,
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

#[cfg(test)]
mod tests {
    use super::{escape_terminal_text, file_uri};
    use std::path::Path;

    #[test]
    fn file_uri_percent_encodes_unsafe_bytes() {
        assert_eq!(file_uri(Path::new("/tmp/a b#c")), "file:///tmp/a%20b%23c");
    }

    #[test]
    fn escapes_terminal_controls_and_preserves_readable_unicode() {
        assert_eq!(
            escape_terminal_text("файл界\n\r\t\x1b\x07\u{009b}\u{202e}"),
            "файл界\\n\\r\\t\\u{1b}\\u{7}\\u{9b}\\u{202e}"
        );
        assert_ne!(escape_terminal_text("a\n"), escape_terminal_text("a\\n"));
    }
}
