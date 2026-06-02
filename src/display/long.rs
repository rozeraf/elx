use crate::fs::entry::Entry;
use crate::theme::Theme;
use crate::cli::Cli;
use std::os::unix::fs::PermissionsExt;
use chrono::{DateTime, Local};
use users::{get_user_by_uid, get_group_by_gid};
use crossterm::style::{Color, Stylize};

pub struct LongView<'a> {
    entries: &'a [Entry],
    theme: &'a Theme,
    args: &'a Cli,
}

impl<'a> LongView<'a> {
    pub fn new(entries: &'a [Entry], theme: &'a Theme, args: &'a Cli) -> Self {
        Self {
            entries,
            theme,
            args,
        }
    }

    pub fn render(&self) {
        let mut rows = Vec::new();

        for entry in self.entries {
            let metadata = &entry.metadata;
            
            // Permissions
            let mode = metadata.permissions().mode();
            let perms = if self.args.no_color {
                format_permissions(mode, metadata.is_dir())
            } else {
                format_permissions_colored(mode, metadata.is_dir())
            };

            // Links
            #[cfg(unix)]
            let links_str = {
                use std::os::unix::fs::MetadataExt;
                metadata.nlink().to_string()
            };
            #[cfg(not(unix))]
            let links_str = "1".to_string();

            // Owner & Group
            #[cfg(unix)]
            let (owner_name, group_name) = {
                use std::os::unix::fs::MetadataExt;
                let uid = metadata.uid();
                let gid = metadata.gid();
                let u = get_user_by_uid(uid)
                    .map(|u| u.name().to_string_lossy().into_owned())
                    .unwrap_or_else(|| uid.to_string());
                let g = get_group_by_gid(gid)
                    .map(|g| g.name().to_string_lossy().into_owned())
                    .unwrap_or_else(|| gid.to_string());
                (u, g)
            };
            #[cfg(not(unix))]
            let (owner_name, group_name) = ("user".to_string(), "group".to_string());

            let owner = if self.args.no_color {
                owner_name.clone()
            } else {
                owner_name.clone().with(Color::Yellow).to_string()
            };

            let group = if self.args.no_color {
                group_name.clone()
            } else {
                group_name.clone().with(Color::Yellow).to_string()
            };

            // Size
            let size_text = format_size(metadata.len());
            let size = if self.args.no_color {
                size_text.clone()
            } else {
                size_text.clone().with(Color::Green).to_string()
            };

            // Date
            let modified: DateTime<Local> = metadata.modified()
                .map(|t| t.into())
                .unwrap_or_else(|_| Local::now());
            let date_str = modified.format("%b %d %H:%M").to_string();
            let date = if self.args.no_color {
                date_str
            } else {
                date_str.with(Color::Cyan).to_string()
            };

            // Name & Icon
            let icon = if self.args.no_icons {
                "".to_string()
            } else {
                format!("{} ", self.theme.icons.get_icon(&entry.path, entry.metadata.is_dir()))
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

            rows.push(LongRow {
                perms,
                links: links_str.clone(),
                owner,
                group,
                size,
                date,
                icon,
                name,
                target,
                links_len: links_str.len(),
                owner_len: owner_name.len(),
                group_len: group_name.len(),
                size_len: size_text.len(),
            });
        }

        // Calculate column widths
        let mut w_links = 0;
        let mut w_owner = 0;
        let mut w_group = 0;
        let mut w_size = 0;

        for row in &rows {
            w_links = w_links.max(row.links_len);
            w_owner = w_owner.max(row.owner_len);
            w_group = w_group.max(row.group_len);
            w_size = w_size.max(row.size_len);
        }

        for row in rows {
            print!("{} ", row.perms);
            
            // Links: Right-aligned
            print!("{:>width$} ", row.links, width = w_links);
            
            if self.args.no_color {
                print!("{:<width$} ", row.owner, width = w_owner);
                print!("{:<width$} ", row.group, width = w_group);
                print!("{:>width$} ", row.size, width = w_size);
            } else {
                print!("{} ", pad_right_ansi(&row.owner, w_owner));
                print!("{} ", pad_right_ansi(&row.group, w_group));
                print!("{} ", pad_left_ansi(&row.size, w_size));
            }

            println!(
                "{} {}{}{}",
                row.date,
                row.icon,
                row.name,
                row.target,
            );
        }
    }
}

fn pad_left_ansi(s: &str, width: usize) -> String {
    let actual_len = strip_ansi(s).len();
    if actual_len >= width {
        s.to_string()
    } else {
        format!("{}{}", " ".repeat(width - actual_len), s)
    }
}

struct LongRow {
    perms: String,
    links: String,
    owner: String,
    group: String,
    size: String,
    date: String,
    icon: String,
    name: String,
    target: String,
    links_len: usize,
    owner_len: usize,
    group_len: usize,
    size_len: usize,
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

fn strip_ansi(s: &str) -> String {
    let re = regex::Regex::new(r"\x1B\[[0-9;]*[mK]").unwrap();
    re.replace_all(s, "").to_string()
}

fn pad_right_ansi(s: &str, width: usize) -> String {
    let actual_len = strip_ansi(s).len();
    if actual_len >= width {
        s.to_string()
    } else {
        format!("{}{}", s, " ".repeat(width - actual_len))
    }
}
