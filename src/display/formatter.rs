use crate::fs::entry::Entry;
use crate::theme::Theme;
use crate::display::DisplayOptions;
use crate::git::GitStatus;
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
                use crate::config::PermissionsFormat;

                let (content, width) = match self.options.permissions_format {
                    PermissionsFormat::Symbolic => {
                        let c = if self.options.color {
                            format_permissions_colored(&entry.metadata)
                        } else {
                            format_permissions(&entry.metadata)
                        };
                        (c, 10usize)
                    }
                    PermissionsFormat::Octal => {
                        let oct = format_permissions_octal(&entry.metadata);
                        let c = if self.options.color {
                            oct.with(Color::Yellow).to_string()
                        } else {
                            oct
                        };
                        (c, 4usize)
                    }
                    PermissionsFormat::Both => {
                        let sym = if self.options.color {
                            format_permissions_colored(&entry.metadata)
                        } else {
                            format_permissions(&entry.metadata)
                        };
                        let oct = format_permissions_octal(&entry.metadata);
                        let c = if self.options.color {
                            format!("{} {}", sym, oct.with(Color::Yellow))
                        } else {
                            format!("{} {}", sym, oct)
                        };
                        (c, 15usize)
                    }
                };
                Cell { content, width }
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

#[cfg(unix)]
fn file_type_char(metadata: &std::fs::Metadata) -> char {
    let ft = metadata.file_type();
    if ft.is_dir() { return 'd'; }
    if ft.is_symlink() { return 'l'; }
    {
        use std::os::unix::fs::FileTypeExt;
        if ft.is_block_device() { return 'b'; }
        if ft.is_char_device()  { return 'c'; }
        if ft.is_fifo()         { return 'p'; }
        if ft.is_socket()       { return 's'; }
    }
    '-'
}

#[cfg(unix)]
fn triplet_chars(bits: u32, special: bool, sc_exec: char, sc_noexec: char) -> [char; 3] {
    let r = if bits & 4 != 0 { 'r' } else { '-' };
    let w = if bits & 2 != 0 { 'w' } else { '-' };
    let x = if special {
        if bits & 1 != 0 { sc_exec } else { sc_noexec }
    } else {
        if bits & 1 != 0 { 'x' } else { '-' }
    };
    [r, w, x]
}

#[cfg(unix)]
fn format_permissions(metadata: &std::fs::Metadata) -> String {
    use std::os::unix::fs::PermissionsExt;
    let mode = metadata.permissions().mode();
    let perm = mode & 0o7777;

    let special = (perm >> 9) & 0o7;  // старшие 3 бита: setuid/setgid/sticky
    let owner   = (perm >> 6) & 0o7;
    let group   = (perm >> 3) & 0o7;
    let other   =  perm       & 0o7;

    let owner_t = triplet_chars(owner, special & 4 != 0, 's', 'S');
    let group_t = triplet_chars(group, special & 2 != 0, 's', 'S');
    let other_t = triplet_chars(other, special & 1 != 0, 't', 'T');

    let mut s = String::with_capacity(10);
    s.push(file_type_char(metadata));
    s.extend(owner_t.iter());
    s.extend(group_t.iter());
    s.extend(other_t.iter());
    s
}

#[cfg(unix)]
fn format_permissions_colored(metadata: &std::fs::Metadata) -> String {
    use std::os::unix::fs::PermissionsExt;
    use crossterm::style::{Color, Stylize};

    let mode = metadata.permissions().mode();
    let perm = mode & 0o7777;
    let special = (perm >> 9) & 0o7;
    let owner   = (perm >> 6) & 0o7;
    let group   = (perm >> 3) & 0o7;
    let other   =  perm       & 0o7;

    let owner_t = triplet_chars(owner, special & 4 != 0, 's', 'S');
    let group_t = triplet_chars(group, special & 2 != 0, 's', 'S');
    let other_t = triplet_chars(other, special & 1 != 0, 't', 'T');

    let mut s = String::new();

    // Тип файла
    let type_char = file_type_char(metadata);
    s.push_str(&match type_char {
        'd' => type_char.to_string().with(Color::Blue).to_string(),
        'l' | 'c' | 'b' | 'p' | 's' => type_char.to_string().with(Color::Cyan).to_string(),
        _ => type_char.to_string().with(Color::Grey).to_string(),
    });

    // Функция окраски одного символа в триплете
    let color_char = |c: char, pos: usize| -> String {
        match (c, pos) {
            ('r', _) => c.to_string().with(Color::Yellow).to_string(),
            ('w', _) => c.to_string().with(Color::Red).to_string(),
            ('x', _) => c.to_string().with(Color::Green).to_string(),
            ('-', _) => c.to_string().with(Color::Grey).to_string(),
            ('s' | 'S', p) if p < 3 => {  // setuid (owner)
                if c == 's' { c.to_string().with(Color::Magenta).bold().to_string() }
                else { c.to_string().with(Color::Magenta).to_string() }
            }
            ('s' | 'S', p) if p < 6 => {  // setgid (group)
                if c == 's' { c.to_string().with(Color::Magenta).to_string() }
                else { c.to_string().with(Color::DarkMagenta).to_string() }
            }
            ('t' | 'T', _) => {           // sticky (other)
                if c == 't' { c.to_string().with(Color::Cyan).to_string() }
                else { c.to_string().with(Color::DarkCyan).to_string() }
            }
            _ => c.to_string(),
        }
    };

    // Строим 9 символов: owner (0-2), group (3-5), other (6-8)
    for (i, c) in owner_t.iter().chain(group_t.iter()).chain(other_t.iter()).enumerate() {
        s.push_str(&color_char(*c, i));
    }
    s
}

#[cfg(unix)]
fn format_permissions_octal(metadata: &std::fs::Metadata) -> String {
    use std::os::unix::fs::PermissionsExt;
    let mode = metadata.permissions().mode() & 0o7777;
    let special = (mode >> 9) & 0o7;
    let owner   = (mode >> 6) & 0o7;
    let group   = (mode >> 3) & 0o7;
    let other   =  mode       & 0o7;
    format!("{}{}{}{}", special, owner, group, other)
}

#[cfg(not(unix))]
fn format_permissions(_metadata: &std::fs::Metadata) -> String {
    "----------".to_string()
}

#[cfg(not(unix))]
fn format_permissions_colored(_metadata: &std::fs::Metadata) -> String {
    "----------".to_string()
}

#[cfg(not(unix))]
fn format_permissions_octal(_metadata: &std::fs::Metadata) -> String {
    "0000".to_string()
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
