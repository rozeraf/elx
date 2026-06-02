use crate::fs::entry::Entry;
use crate::theme::Theme;
use crate::display::{DisplayOptions, Column, ColumnFormatter};
use std::collections::HashMap;

pub const VERTICAL: &str = "\u{2502}"; // │
pub const BRANCH: &str = "\u{251c}";   // ├
pub const LAST_BRANCH: &str = "\u{2514}"; // └
pub const HORIZONTAL: &str = "\u{2500}"; // ─

pub struct TreeView<'a> {
    entries: &'a [Entry],
    theme: &'a Theme,
    options: &'a DisplayOptions,
    columns: Vec<Column>,
}

impl<'a> TreeView<'a> {
    pub fn new(entries: &'a [Entry], theme: &'a Theme, options: &'a DisplayOptions, columns: Vec<Column>) -> Self {
        Self {
            entries,
            theme,
            options,
            columns,
        }
    }

    pub fn render(&self) {
        let formatter = ColumnFormatter {
            theme: self.theme,
            options: self.options,
        };

        let mut col_widths: HashMap<Column, usize> = HashMap::new();
        if self.options.long_view {
            self.calculate_widths(&self.entries, &formatter, &mut col_widths);
        }

        for (i, entry) in self.entries.iter().enumerate() {
            let is_last = i == self.entries.len() - 1;
            self.render_entry(entry, "", is_last, &formatter, &col_widths);
        }
    }

    fn calculate_widths(&self, entries: &[Entry], formatter: &ColumnFormatter, widths: &mut HashMap<Column, usize>) {
        for entry in entries {
            for col in &self.columns {
                if *col == Column::Name { continue; }
                let cell = formatter.format_column(*col, entry);
                let current_max = widths.entry(*col).or_insert(0);
                *current_max = (*current_max).max(cell.width);
            }
            if let Some(children) = &entry.children {
                self.calculate_widths(children, formatter, widths);
            }
        }
    }

    fn render_entry(&self, entry: &Entry, prefix: &str, is_last: bool, formatter: &ColumnFormatter, widths: &HashMap<Column, usize>) {
        // 1. Render Metadata (if enabled)
        if self.options.long_view {
             for col in &self.columns {
                if *col == Column::Name { continue; }
                if let Some(&width) = widths.get(col) {
                    let cell = formatter.format_column(*col, entry);
                    match col {
                        Column::Permissions | Column::Date => {
                            print!("{} ", cell.content);
                            print!("{}", " ".repeat(width.saturating_sub(cell.width)));
                        }
                        Column::Links | Column::Size => {
                            print!("{}{}", " ".repeat(width.saturating_sub(cell.width)), cell.content);
                            print!(" ");
                        }
                        Column::Owner | Column::Group => {
                            print!("{} ", cell.content);
                            print!("{}", " ".repeat(width.saturating_sub(cell.width)));
                        }
                        _ => {}
                    }
                }
            }
        }

        // 2. Render Tree structure and Name
        let branch = if is_last { LAST_BRANCH } else { BRANCH };
        
        let icon = if !self.options.icons {
            "".to_string()
        } else {
            format!("{} ", self.theme.icons.get_icon(&entry.path, entry.metadata.is_dir()))
        };

        let name = if !self.options.color {
            entry.name.clone()
        } else {
            self.theme.colors.colorize(&entry.name, &entry.metadata).to_string()
        };

        let suffix = if self.options.classify && entry.metadata.is_dir() { "/" } else { "" };

        println!("{}{} {}{}{}{}", prefix, branch, HORIZONTAL, icon, name, suffix);

        // 3. Render Children
        if let Some(children) = &entry.children {
            let next_prefix = if is_last {
                format!("{}    ", prefix)
            } else {
                format!("{}{}   ", prefix, VERTICAL)
            };
            for (i, child) in children.iter().enumerate() {
                let is_last_child = i == children.len() - 1;
                self.render_entry(child, &next_prefix, is_last_child, formatter, widths);
            }
        }
    }
}
