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
}

impl<'a> LongView<'a> {
    pub fn new(entries: &'a [Entry], theme: &'a Theme, options: &'a DisplayOptions, columns: Vec<Column>, headers: bool) -> Self {
        Self {
            entries,
            theme,
            options,
            columns,
            headers,
        }
    }

    pub fn render(&self) {
        let formatter = ColumnFormatter {
            theme: self.theme,
            options: self.options,
        };

        let mut table: Vec<HashMap<Column, Cell>> = Vec::new();
        let mut col_widths: HashMap<Column, usize> = HashMap::new();

        for entry in self.entries {
            let mut row = HashMap::new();
            
            for col in &self.columns {
                let cell = formatter.format_column(*col, entry);
                let current_max = col_widths.entry(*col).or_insert(0);
                *current_max = (*current_max).max(cell.width);
                row.insert(*col, cell);
            }
            
            table.push(row);
        }

        if self.headers {
            for (i, col) in self.columns.iter().enumerate() {
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
                let is_last = i == self.columns.len() - 1;
                
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
            for (i, col) in self.columns.iter().enumerate() {
                if let Some(cell) = row.get(col) {
                    let width = col_widths[col];
                    let is_last = i == self.columns.len() - 1;
                    
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
