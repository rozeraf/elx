use crate::fs::entry::Entry;
use crate::theme::Theme;
use crate::cli::Cli;
use std::os::unix::fs::PermissionsExt;
use chrono::{DateTime, Local};
use users::{get_user_by_uid, get_group_by_gid};
use unicode_width::UnicodeWidthStr;

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
            let perms = format_permissions(mode, metadata.is_dir());

            // Links
            #[cfg(unix)]
            let links = {
                use std::os::unix::fs::MetadataExt;
                metadata.nlink().to_string()
            };
            #[cfg(not(unix))]
            let links = "1".to_string();

            // Owner & Group
            #[cfg(unix)]
            let (owner, group) = {
                use std::os::unix::fs::MetadataExt;
                let uid = metadata.uid();
                let gid = metadata.gid();
                let user_name = get_user_by_uid(uid)
                    .map(|u| u.name().to_string_lossy().into_owned())
                    .unwrap_or_else(|| uid.to_string());
                let group_name = get_group_by_gid(gid)
                    .map(|g| g.name().to_string_lossy().into_owned())
                    .unwrap_or_else(|| gid.to_string());
                (user_name, group_name)
            };
            #[cfg(not(unix))]
            let (owner, group) = ("user".to_string(), "group".to_string());

            // Size
            let size = format_size(metadata.len());

            // Date
            let modified: DateTime<Local> = metadata.modified()
                .map(|t| t.into())
                .unwrap_or_else(|_| Local::now());
            let date = modified.format("%b %d %H:%M").to_string();

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

            rows.push(LongRow {
                perms,
                links,
                owner,
                group,
                size,
                date,
                icon,
                name,
                icon_width: if self.args.no_icons { 0 } else { 2 }, // icon + space
                name_width: entry.name.width(),
            });
        }

        // Calculate column widths
        let mut w_links = 0;
        let mut w_owner = 0;
        let mut w_group = 0;
        let mut w_size = 0;

        for row in &rows {
            w_links = w_links.max(row.links.len());
            w_owner = w_owner.max(row.owner.len());
            w_group = w_group.max(row.group.len());
            w_size = w_size.max(row.size.len());
        }

        for row in rows {
            println!(
                "{} {:>links_w$} {:<owner_w$} {:<group_w$} {:>size_w$} {} {}{}",
                row.perms,
                row.links,
                row.owner,
                row.group,
                row.size,
                row.date,
                row.icon,
                row.name,
                links_w = w_links,
                owner_w = w_owner,
                group_w = w_group,
                size_w = w_size,
            );
        }
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
    #[allow(dead_code)]
    icon_width: usize,
    #[allow(dead_code)]
    name_width: usize,
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

fn format_size(size: u64) -> String {
    const UNITS: &[&str] = &["B", "K", "M", "G", "T"];
    let mut size = size as f64;
    let mut unit_idx = 0;

    while size >= 1024.0 && unit_idx < UNITS.len() - 1 {
        size /= 1024.0;
        unit_idx += 1;
    }

    if unit_idx == 0 {
        format!("{:.0}{}", size, UNITS[unit_idx])
    } else {
        format!("{:.1}{}", size, UNITS[unit_idx])
    }
}
