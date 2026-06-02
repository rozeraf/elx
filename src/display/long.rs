use crate::fs::entry::Entry;
use crate::theme::Theme;
use crate::cli::Cli;
use std::os::unix::fs::PermissionsExt;
use chrono::{DateTime, Local};
use users::{get_user_by_uid, get_group_by_gid};
use crossterm::style::{Color, Stylize};
use unicode_width::UnicodeWidthStr;
use std::collections::HashMap;

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
}

pub struct LongView<'a> {
    entries: &'a [Entry],
    theme: &'a Theme,
    args: &'a Cli,
    columns: Vec<Column>,
}

struct Cell {
    content: String,
    width: usize,
}

impl<'a> LongView<'a> {
    pub fn new(entries: &'a [Entry], theme: &'a Theme, args: &'a Cli, columns: Vec<Column>) -> Self {
        Self {
            entries,
            theme,
            args,
            columns,
        }
    }

    pub fn render(&self) {
        let mut table: Vec<HashMap<Column, Cell>> = Vec::new();
        let mut col_widths: HashMap<Column, usize> = HashMap::new();

        for entry in self.entries {
            let mut row = HashMap::new();
            
            for col in &self.columns {
                let cell = self.format_column(*col, entry);
                let current_max = col_widths.entry(*col).or_insert(0);
                *current_max = (*current_max).max(cell.width);
                row.insert(*col, cell);
            }
            
            table.push(row);
        }

        for row in table {
            for (i, col) in self.columns.iter().enumerate() {
                if let Some(cell) = row.get(col) {
                    let width = col_widths[col];
                    let is_last = i == self.columns.len() - 1;
                    
                    match col {
                        Column::Permissions | Column::Date | Column::Name => {
                            print!("{}", cell.content);
                            if !is_last {
                                print!("{} ", " ".repeat(width.saturating_sub(cell.width)));
                            }
                        }
                        Column::Links | Column::Size => {
                            print!("{}{}", " ".repeat(width.saturating_sub(cell.width)), cell.content);
                            if !is_last {
                                print!(" ");
                            }
                        }
                        Column::Owner | Column::Group => {
                            print!("{}", cell.content);
                            if !is_last {
                                print!("{} ", " ".repeat(width.saturating_sub(cell.width)));
                            }
                        }
                    }
                }
            }
            println!();
        }
    }

    fn format_column(&self, col: Column, entry: &Entry) -> Cell {
        let metadata = &entry.metadata;
        
        match col {
            Column::Permissions => {
                let mode = metadata.permissions().mode();
                let content = if self.args.no_color {
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
                let content = if self.args.no_color {
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
                let content = if self.args.no_color {
                    group_name
                } else {
                    group_name.with(Color::Yellow).to_string()
                };
                Cell { content, width }
            }
            Column::Size => {
                let size_text = format_size(metadata.len());
                let width = size_text.len();
                let content = if self.args.no_color {
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
                let content = if self.args.no_color {
                    date_str
                } else {
                    date_str.with(Color::Cyan).to_string()
                };
                Cell { content, width }
            }
            Column::Name => {
                let icon_str = if self.args.no_icons {
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

                let name = if self.args.no_color {
                    entry.name.clone()
                } else {
                    self.theme.colors.colorize(&entry.name, &entry.metadata).to_string()
                };

                let target = if let Some(t) = &entry.link_target {
                    let target_name = t.display().to_string();
                    if self.args.no_color {
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

                Cell {
                    content: format!("{}{}{}", icon, name, target),
                    width: icon_width + entry.name.width() + target_width,
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
