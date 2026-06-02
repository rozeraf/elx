pub struct GridOptions {
    pub terminal_width: usize,
    pub entries: Vec<GridEntry>,
    pub one_per_line: bool,
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

    if options.one_per_line {
        return options.entries.iter()
            .map(|e| e.display_name.clone())
            .collect::<Vec<_>>()
            .join("\n") + "\n";
    }

    // 1. Try single line
    let total_width_single_line: usize = options.entries.iter().map(|e| e.display_width).sum::<usize>() 
        + (count.saturating_sub(1) * 2);
    
    if total_width_single_line <= options.terminal_width {
        return options.entries.iter()
            .map(|e| e.display_name.clone())
            .collect::<Vec<_>>()
            .join("  ") + "\n";
    }

    // 2. Variable column width algorithm
    let mut max_cols = options.terminal_width / 2; // Rough upper bound
    if max_cols > count {
        max_cols = count;
    }
    if max_cols == 0 {
        max_cols = 1;
    }

    for cols in (2..=max_cols).rev() {
        let rows = (count + cols - 1) / cols;
        let mut col_widths = vec![0; cols];

        for col in 0..cols {
            let mut current_max = 0;
            for row in 0..rows {
                let idx = col * rows + row;
                if idx < count {
                    current_max = current_max.max(options.entries[idx].display_width);
                }
            }
            col_widths[col] = current_max;
        }

        let total_grid_width: usize = col_widths.iter().sum::<usize>() + 2 * (cols - 1);
        if total_grid_width <= options.terminal_width {
            return build_grid(&options.entries, rows, cols, &col_widths);
        }
    }

    // Fallback: single column
    options.entries.iter()
        .map(|e| e.display_name.clone())
        .collect::<Vec<_>>()
        .join("\n") + "\n"
}

fn build_grid(entries: &[GridEntry], rows: usize, cols: usize, col_widths: &[usize]) -> String {
    let count = entries.len();
    let mut output = String::new();

    for row in 0..rows {
        for col in 0..cols {
            let idx = col * rows + row;
            if idx < count {
                let entry = &entries[idx];
                output.push_str(&entry.display_name);

                if col < cols - 1 && (col + 1) * rows + row < count {
                    let padding = (col_widths[col] + 2) - entry.display_width;
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
    fn test_single_file() {
        let options = GridOptions {
            terminal_width: 80,
            entries: vec![GridEntry {
                display_name: "file1".to_string(),
                display_width: 5,
            }],
            one_per_line: false,
        };
        assert_eq!(render(options), "file1\n");
    }

    #[test]
    fn test_all_in_one_line() {
        let entries = vec![
            GridEntry { display_name: "f1".to_string(), display_width: 2 },
            GridEntry { display_name: "f2".to_string(), display_width: 2 },
            GridEntry { display_name: "f3".to_string(), display_width: 2 },
        ];
        // 2+2+2 + 2*2 = 10. Fits in 10.
        let options = GridOptions {
            terminal_width: 10,
            entries,
            one_per_line: false,
        };
        assert_eq!(render(options), "f1  f2  f3\n");
    }

    #[test]
    fn test_variable_widths() {
        let entries = vec![
            GridEntry { display_name: "long_name".to_string(), display_width: 9 },
            GridEntry { display_name: "s".to_string(), display_width: 1 },
            GridEntry { display_name: "medium".to_string(), display_width: 6 },
            GridEntry { display_name: "f4".to_string(), display_width: 2 },
        ];
        // If terminal is small, should have 2 columns.
        // Col 0: long_name (9), s (1) -> width 11
        // Col 1: medium (6), f4 (2) -> width 8
        // Total: 11 + 6 (last col no padding) = 17
        let options = GridOptions {
            terminal_width: 17,
            entries,
            one_per_line: false,
        };
        let expected = "long_name  medium\ns          f4\n";
        assert_eq!(render(options), expected);
    }

    #[test]
    fn test_odd_number_variable() {
        let entries = vec![
            GridEntry { display_name: "a".to_string(), display_width: 1 },
            GridEntry { display_name: "b".to_string(), display_width: 1 },
            GridEntry { display_name: "c".to_string(), display_width: 1 },
            GridEntry { display_name: "d".to_string(), display_width: 1 },
            GridEntry { display_name: "e".to_string(), display_width: 1 },
        ];
        // cols=2, rows=3.
        // Col 0: a, b, c -> width 3
        // Col 1: d, e -> width 1
        let options = GridOptions {
            terminal_width: 5,
            entries,
            one_per_line: false,
        };
        let expected = "a  d\nb  e\nc\n";
        assert_eq!(render(options), expected);
    }

    #[test]
    fn test_one_per_line() {
        let entries = vec![
            GridEntry { display_name: "a".to_string(), display_width: 1 },
            GridEntry { display_name: "b".to_string(), display_width: 1 },
        ];
        let options = GridOptions {
            terminal_width: 80,
            entries,
            one_per_line: true,
        };
        assert_eq!(render(options), "a\nb\n");
    }
}
