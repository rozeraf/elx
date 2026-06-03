use crate::fs::entry::Entry;
use crate::theme::Theme;
use crate::display::{DisplayOptions, Column, ColumnFormatter, Cell};
use std::collections::HashMap;

pub struct LongView<'a> {
    entries: &'a [Entry],
    theme: &'a Theme,
    options: &'a DisplayOptions,
    columns: Vec<Column>,
    headers: bool,
    autohide_columns: bool,
}

impl<'a> LongView<'a> {
    pub fn new(entries: &'a [Entry], theme: &'a Theme, options: &'a DisplayOptions, columns: Vec<Column>, headers: bool, autohide_columns: bool) -> Self {
        Self {
            entries,
            theme,
            options,
            columns,
            headers,
            autohide_columns,
        }
    }

    pub fn render(&self) {
        let formatter = ColumnFormatter {
            theme: self.theme,
            options: self.options,
        };

        let mut table: Vec<HashMap<Column, Cell>> = Vec::new();
        let mut col_widths: HashMap<Column, usize> = HashMap::new();
        let mut active_columns = self.columns.clone();

        if self.autohide_columns {
            let mut empty_columns = std::collections::HashSet::new();
            for col in &self.columns {
                let mut all_empty = true;
                for entry in self.entries {
                    match col {
                        Column::Git => {
                            if entry.git_status.is_some() {
                                all_empty = false;
                                break;
                            }
                        }
                        // Other columns are generally never "empty" in a way that warrants hiding
                        _ => {
                            all_empty = false;
                            break;
                        }
                    }
                }
                if all_empty {
                    empty_columns.insert(*col);
                }
            }
            active_columns.retain(|c| !empty_columns.contains(c));
        }

        for col in &active_columns {
            let header_len = match col {
                Column::Git => 3,
                Column::Permissions => 4,
                Column::Links => 4,
                Column::Owner => 4,
                Column::Group => 5,
                Column::Size => 4,
                Column::Date => 4,
                Column::Name => 4,
            };
            col_widths.insert(*col, header_len);
        }

        for entry in self.entries {
            let mut row = HashMap::new();
            
            for col in &active_columns {
                let cell = formatter.format_column(*col, entry);
                let current_max = col_widths.entry(*col).or_insert(0);
                *current_max = (*current_max).max(cell.width);
                row.insert(*col, cell);
            }
            
            table.push(row);
        }

        if self.headers {
            for (i, col) in active_columns.iter().enumerate() {
                let header = match col {
                    Column::Git => "Git",
                    Column::Permissions => "Mode",
                    Column::Links => "Link",
                    Column::Owner => "User",
                    Column::Group => "Group",
                    Column::Size => "Size",
                    Column::Date => "Date",
                    Column::Name => "Name",
                };
                
                let width = col_widths[col];
                let is_last = i == active_columns.len() - 1;
                
                let content = if self.options.color {
                    use crossterm::style::Stylize;
                    header.underlined().to_string()
                } else {
                    header.to_string()
                };

                match col {
                    Column::Links | Column::Size => {
                        print!("{}{}", " ".repeat(width.saturating_sub(header.len())), content);
                    }
                    _ => {
                        print!("{}", content);
                        if !is_last {
                            print!("{}", " ".repeat(width.saturating_sub(header.len())));
                        }
                    }
                }
                if !is_last {
                    print!(" ");
                }
            }
            println!();
        }

        for row in table {
            for (i, col) in active_columns.iter().enumerate() {
                if let Some(cell) = row.get(col) {
                    let width = col_widths[col];
                    let is_last = i == active_columns.len() - 1;
                    
                    match col {
                        Column::Permissions | Column::Date | Column::Name | Column::Git => {
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
}
