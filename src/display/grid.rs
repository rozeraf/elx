pub struct GridOptions {
    pub terminal_width: usize,
    pub entries: Vec<GridEntry>,
}

pub struct GridEntry {
    pub display_name: String,
    pub display_width: usize,
}

pub fn render(options: GridOptions) -> String {
    let count = options.entries.len();
    if count == 0 {
        return String::new();
    }

    let max_width = options.entries.iter().map(|e| e.display_width).max().unwrap_or(0);
    let col_width = max_width + 2;
    let mut cols = options.terminal_width / col_width;
    if cols == 0 {
        cols = 1;
    }

    if cols == 1 {
        return options.entries.iter()
            .map(|e| e.display_name.clone())
            .collect::<Vec<_>>()
            .join("\n") + "\n";
    }

    let rows = (count + cols - 1) / cols;
    let mut output = String::new();

    for row in 0..rows {
        for col in 0..cols {
            let idx = col * rows + row;
            if idx < count {
                let entry = &options.entries[idx];
                output.push_str(&entry.display_name);
                
                // Add padding if not the last column and not the last entry in the row
                if col < cols - 1 && (col + 1) * rows + row < count {
                    let padding = col_width - entry.display_width;
                    output.push_str(&" ".repeat(padding));
                }
            }
        }
        output.push('\n');
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_list() {
        let options = GridOptions {
            terminal_width: 80,
            entries: vec![],
        };
        assert_eq!(render(options), "");
    }

    #[test]
    fn test_single_file() {
        let options = GridOptions {
            terminal_width: 80,
            entries: vec![GridEntry {
                display_name: "file1".to_string(),
                display_width: 5,
            }],
        };
        assert_eq!(render(options), "file1\n");
    }

    #[test]
    fn test_exact_columns() {
        let entries = vec![
            GridEntry { display_name: "f1".to_string(), display_width: 2 },
            GridEntry { display_name: "f2".to_string(), display_width: 2 },
            GridEntry { display_name: "f3".to_string(), display_width: 2 },
            GridEntry { display_name: "f4".to_string(), display_width: 2 },
        ];
        // max_width=2, col_width=4, terminal_width=8 => cols=2, rows=2
        let options = GridOptions {
            terminal_width: 8,
            entries,
        };
        let expected = "f1  f3\nf2  f4\n";
        assert_eq!(render(options), expected);
    }

    #[test]
    fn test_odd_number_of_files() {
        let entries = vec![
            GridEntry { display_name: "f1".to_string(), display_width: 2 },
            GridEntry { display_name: "f2".to_string(), display_width: 2 },
            GridEntry { display_name: "f3".to_string(), display_width: 2 },
            GridEntry { display_name: "f4".to_string(), display_width: 2 },
            GridEntry { display_name: "f5".to_string(), display_width: 2 },
        ];
        // max_width=2, col_width=4, terminal_width=8 => cols=2, rows=3
        let options = GridOptions {
            terminal_width: 8,
            entries,
        };
        let expected = "f1  f4\nf2  f5\nf3\n";
        assert_eq!(render(options), expected);
    }
}
