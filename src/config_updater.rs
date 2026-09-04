use anyhow::{Context, Result};
use std::io::{self, Write};
use std::path::PathBuf;
use toml_edit::{DocumentMut, Item, Table, Value};

pub const CURRENT_SCHEMA_VERSION: u32 = 3;

pub struct ConfigUpdater {
    path: PathBuf,
    doc: DocumentMut,
    interactive: bool,
    pub dry_run: bool,
    changed: bool,
    report: Vec<String>,
}

struct Migration {
    from_version: u32,
    to_version: u32,
    _description: &'static str,
    _requires_interaction: bool,
}

impl ConfigUpdater {
    pub fn new(path: PathBuf, interactive: bool) -> Result<Self> {
        let content = std::fs::read_to_string(&path)
            .with_context(|| format!("Failed to read config file: {}", path.display()))?;
        let doc = content
            .parse::<DocumentMut>()
            .with_context(|| format!("Failed to parse config file: {}", path.display()))?;

        Ok(Self {
            path,
            doc,
            interactive,
            dry_run: false,
            changed: false,
            report: Vec::new(),
        })
    }

    pub fn run(&mut self) -> Result<()> {
        println!("[elx --update-config]");

        let detected = self.detect_version();
        self.apply_migrations()?;

        if self.changed {
            if self.dry_run {
                self.print_report();
                println!(
                    "\n[dry-run] Config would be updated to schema version {}: {}",
                    CURRENT_SCHEMA_VERSION,
                    self.path.display()
                );
            } else {
                self.save()?;
                self.print_report();
                println!(
                    "\nConfig updated to schema version {}: {}",
                    CURRENT_SCHEMA_VERSION,
                    self.path.display()
                );
            }
        } else {
            println!(
                "Config is up to date (schema version {}). No changes needed.",
                detected
            );
        }

        Ok(())
    }

    fn detect_version(&self) -> u32 {
        self.doc
            .get("schema_version")
            .and_then(|v| v.as_integer())
            .map(|v| v as u32)
            .unwrap_or(0)
    }

    fn apply_migrations(&mut self) -> Result<()> {
        let detected = self.detect_version();

        if detected > CURRENT_SCHEMA_VERSION {
            anyhow::bail!(
                "Config schema version {} is newer than the supported version {}",
                detected,
                CURRENT_SCHEMA_VERSION
            );
        }

        let migrations = vec![
            Migration {
                from_version: 0,
                to_version: 1,
                _description: "Migrate when_not_tty no_icons/no_color to icons/color",
                _requires_interaction: false,
            },
            Migration {
                from_version: 1,
                to_version: 2,
                _description: "Migrate flat hyperlinks boolean to [hyperlinks] table",
                _requires_interaction: true,
            },
            Migration {
                from_version: 2,
                to_version: 3,
                _description: "Clean up hyperlinks table fields",
                _requires_interaction: false,
            },
        ];

        for m in migrations {
            if detected <= m.from_version {
                match m.to_version {
                    1 => self.migrate_0_to_1()?,
                    2 => self.migrate_1_to_2()?,
                    3 => self.migrate_2_to_3()?,
                    _ => {}
                }
            }
        }

        self.add_missing_fields()?;

        if detected != CURRENT_SCHEMA_VERSION {
            self.doc.insert(
                "schema_version",
                toml_edit::value(CURRENT_SCHEMA_VERSION as i64),
            );
            if let Some(mut key) = self.doc.key_mut("schema_version") {
                key.leaf_decor_mut()
                    .set_prefix("\n# Config schema version\n");
            }
            self.changed = true;
        }

        Ok(())
    }

    fn migrate_0_to_1(&mut self) -> Result<()> {
        let mut migrated = false;
        if let Some(when_not_tty) = self
            .doc
            .get_mut("when_not_tty")
            .and_then(|i| i.as_table_mut())
        {
            if let Some(no_icons) = when_not_tty.remove("no_icons")
                && let Some(b) = no_icons.as_bool()
            {
                when_not_tty.insert("icons", toml_edit::value(!b));
                migrated = true;
            }
            if let Some(no_color) = when_not_tty.remove("no_color")
                && let Some(b) = no_color.as_bool()
            {
                when_not_tty.insert("color", toml_edit::value(!b));
                migrated = true;
            }
        }
        if migrated {
            self.report.push(
                "Migrated `no_icons`/`no_color` to `icons`/`color` in `[when_not_tty]` table"
                    .to_string(),
            );
            self.changed = true;
        }
        Ok(())
    }

    fn migrate_1_to_2(&mut self) -> Result<()> {
        let hyperlinks_bool = self
            .doc
            .get("hyperlinks")
            .and_then(|i| i.as_value())
            .and_then(|v| v.as_bool());
        if let Some(h_bool) = hyperlinks_bool {
            self.doc.remove("hyperlinks");

            let mut enabled = h_bool;
            let mut files = true;
            let mut dirs = false;
            let mut symlinks = true;
            let mut executables = true;

            if !h_bool {
                if self.interactive {
                    println!("\n[update-config] Found structural change:");
                    println!("  hyperlinks = false  →  [hyperlinks] table\n");
                    println!("  You had hyperlinks disabled. New format allows per-type control.");

                    let choice = self.ask_user(
                        "  Migrate with all types disabled? [Y]\n  Or customize? (y/n/customize): ",
                        &["y", "n", "customize"],
                        0,
                    );

                    match choice {
                        0 => {
                            enabled = false;
                            files = false;
                            dirs = false;
                            symlinks = false;
                            executables = false;
                        }
                        1 => {
                            enabled = false;
                            files = true;
                            dirs = false;
                            symlinks = true;
                            executables = true;
                        }
                        2 => {
                            enabled = self.ask_user(
                                "  Enable hyperlinks globally? (y/n) [n]: ",
                                &["y", "n"],
                                1,
                            ) == 0;
                            files = self.ask_user(
                                "  Enable hyperlinks for regular files? (y/n) [y]: ",
                                &["y", "n"],
                                0,
                            ) == 0;
                            dirs = self.ask_user(
                                "  Enable hyperlinks for directories? (y/n) [n]: ",
                                &["y", "n"],
                                1,
                            ) == 0;
                            symlinks = self.ask_user(
                                "  Enable hyperlinks for symlinks? (y/n) [y]: ",
                                &["y", "n"],
                                0,
                            ) == 0;
                            executables = self.ask_user(
                                "  Enable hyperlinks for executables? (y/n) [y]: ",
                                &["y", "n"],
                                0,
                            ) == 0;
                        }
                        _ => {}
                    }
                } else {
                    enabled = false;
                    files = true;
                    dirs = false;
                    symlinks = true;
                    executables = true;
                }
            }

            let mut hyperlinks_table = toml_edit::Table::new();
            hyperlinks_table.insert("enabled", toml_edit::value(enabled));
            hyperlinks_table.insert("files", toml_edit::value(files));
            hyperlinks_table.insert("dirs", toml_edit::value(dirs));
            hyperlinks_table.insert("symlinks", toml_edit::value(symlinks));
            hyperlinks_table.insert("executables", toml_edit::value(executables));

            self.doc.insert("hyperlinks", Item::Table(hyperlinks_table));
            self.report.push(
                "Migrated `hyperlinks` from boolean to `[hyperlinks]` table format".to_string(),
            );
            self.changed = true;
        }
        Ok(())
    }

    fn migrate_2_to_3(&mut self) -> Result<()> {
        let mut changed = false;

        if self.doc.remove("exclude_env").is_some() {
            self.report.push(
                "Removed `exclude_env` (this option had no effect and has been removed)"
                    .to_string(),
            );
            changed = true;
        }

        if let Some(hyperlinks) = self
            .doc
            .get_mut("hyperlinks")
            .and_then(|i| i.as_table_mut())
        {
            if hyperlinks.remove("exclude_env").is_some() {
                self.report.push(
                    "Removed `exclude_env` (this option had no effect and has been removed)"
                        .to_string(),
                );
                changed = true;
            }

            let underlines = [
                "underline_files",
                "underline_directories",
                "underline_symlinks",
            ];
            let mut removed_underlines = false;
            for u in underlines {
                if hyperlinks.remove(u).is_some() {
                    removed_underlines = true;
                }
            }
            if removed_underlines {
                self.report.push(
                    "Removed deprecated underlining settings from `[hyperlinks]` table".to_string(),
                );
                changed = true;
            }

            if let Some(dirs_val) = hyperlinks.remove("directories") {
                hyperlinks.insert("dirs", dirs_val);
                self.report
                    .push("Renamed `directories` to `dirs` in `[hyperlinks]` table".to_string());
                changed = true;
            }

            if !hyperlinks.contains_key("executables") {
                hyperlinks.insert("executables", toml_edit::value(true));
                self.report
                    .push("Added `executables = true` to `[hyperlinks]` table".to_string());
                changed = true;
            }
        }

        if let Some(when_not_tty) = self
            .doc
            .get_mut("when_not_tty")
            .and_then(|i| i.as_table_mut())
            && when_not_tty.remove("hyperlinks").is_some()
        {
            self.report.push(
                "Removed deprecated `hyperlinks` setting from `[when_not_tty]` table".to_string(),
            );
            changed = true;
        }

        if changed {
            self.changed = true;
        }
        Ok(())
    }

    fn add_missing_fields(&mut self) -> Result<()> {
        let mut added_count = 0;

        if !self.doc.contains_key("icons") {
            self.doc.insert("icons", toml_edit::value(true));
            if let Some(mut key) = self.doc.key_mut("icons") {
                key.leaf_decor_mut()
                    .set_prefix("\n# Display icons next to file names (requires Nerd Font)\n");
            }
            added_count += 1;
        }

        if !self.doc.contains_key("color") {
            self.doc.insert("color", toml_edit::value(true));
            if let Some(mut key) = self.doc.key_mut("color") {
                key.leaf_decor_mut()
                    .set_prefix("\n# Use colors in output\n");
            }
            added_count += 1;
        }

        if !self.doc.contains_key("git_ignore") {
            self.doc.insert("git_ignore", toml_edit::value(true));
            if let Some(mut key) = self.doc.key_mut("git_ignore") {
                key.leaf_decor_mut()
                    .set_prefix("\n# Respect .gitignore files\n");
            }
            added_count += 1;
        }

        if !self.doc.contains_key("ignore") {
            let arr = toml_edit::Array::new();
            self.doc.insert("ignore", Item::Value(Value::Array(arr)));
            if let Some(mut key) = self.doc.key_mut("ignore") {
                key.leaf_decor_mut().set_prefix("\n# Global ignore patterns (glob)\n# Example: ignore = [\"*.json\", \"node_modules\", \"bin\"]\n");
            }
            added_count += 1;
        }

        if !self.doc.contains_key("classify") {
            self.doc.insert("classify", toml_edit::value(false));
            if let Some(mut key) = self.doc.key_mut("classify") {
                key.leaf_decor_mut()
                    .set_prefix("\n# Append indicator (one of /) to directories\n");
            }
            added_count += 1;
        }

        let hyperlinks_table = self
            .doc
            .entry("hyperlinks")
            .or_insert_with(|| Item::Table(Table::new()))
            .as_table_mut()
            .context("hyperlinks is not a table")?;

        if !hyperlinks_table.contains_key("enabled") {
            hyperlinks_table.insert("enabled", toml_edit::value(true));
            if let Some(mut key) = hyperlinks_table.key_mut("enabled") {
                key.leaf_decor_mut().set_prefix(
                    "\n# Master switch: enable clickable OSC 8 hyperlinks in supported terminals\n",
                );
            }
            added_count += 1;
        }
        if !hyperlinks_table.contains_key("files") {
            hyperlinks_table.insert("files", toml_edit::value(true));
            if let Some(mut key) = hyperlinks_table.key_mut("files") {
                key.leaf_decor_mut()
                    .set_prefix("\n# Control which entry types get hyperlinks\n");
            }
            added_count += 1;
        }
        if !hyperlinks_table.contains_key("dirs") {
            hyperlinks_table.insert("dirs", toml_edit::value(false));
            if let Some(mut key) = hyperlinks_table.key_mut("dirs") {
                key.leaf_decor_mut()
                    .set_prefix("# disabled: clicking a dir opens it in a file manager\n");
            }
            added_count += 1;
        }
        if !hyperlinks_table.contains_key("symlinks") {
            hyperlinks_table.insert("symlinks", toml_edit::value(true));
            added_count += 1;
        }
        if !hyperlinks_table.contains_key("executables") {
            hyperlinks_table.insert("executables", toml_edit::value(true));
            added_count += 1;
        }

        let long_table = self
            .doc
            .entry("long")
            .or_insert_with(|| Item::Table(Table::new()))
            .as_table_mut()
            .context("long is not a table")?;

        if !long_table.contains_key("headers") {
            long_table.insert("headers", toml_edit::value(true));
            if let Some(mut key) = long_table.key_mut("headers") {
                key.leaf_decor_mut()
                    .set_prefix("\n# Display column headers in long listing format\n");
            }
            added_count += 1;
        }
        if !long_table.contains_key("autohide_columns") {
            long_table.insert("autohide_columns", toml_edit::value(true));
            if let Some(mut key) = long_table.key_mut("autohide_columns") {
                key.leaf_decor_mut().set_prefix("\n# Automatically hide columns that are empty for all shown entries (e.g., Git column if no changes)\n");
            }
            added_count += 1;
        }
        if !long_table.contains_key("columns") {
            let mut arr = toml_edit::Array::new();
            arr.push("git");
            arr.push("permissions");
            arr.push("links");
            arr.push("owner");
            arr.push("group");
            arr.push("size");
            arr.push("date");
            arr.push("name");
            long_table.insert("columns", Item::Value(Value::Array(arr)));
            if let Some(mut key) = long_table.key_mut("columns") {
                key.leaf_decor_mut().set_prefix("\n# Columns to display in long listing format.\n# Available columns: git, permissions, links, owner, group, size, date, name\n");
            }
            added_count += 1;
        }

        let when_not_tty_table = self
            .doc
            .entry("when_not_tty")
            .or_insert_with(|| Item::Table(Table::new()))
            .as_table_mut()
            .context("when_not_tty is not a table")?;

        if !when_not_tty_table.contains_key("icons") {
            when_not_tty_table.insert("icons", toml_edit::value(false));
            if let Some(mut key) = when_not_tty_table.key_mut("icons") {
                key.leaf_decor_mut().set_prefix(
                    "\n# Configuration for when output is redirected (e.g., to a file or pipe)\n",
                );
            }
            added_count += 1;
        }
        if !when_not_tty_table.contains_key("color") {
            when_not_tty_table.insert("color", toml_edit::value(false));
            added_count += 1;
        }
        if !when_not_tty_table.contains_key("one_per_line") {
            when_not_tty_table.insert("one_per_line", toml_edit::value(true));
            added_count += 1;
        }

        if added_count > 0 {
            self.report.push(format!(
                "Added {} new fields with default values",
                added_count
            ));
            self.changed = true;
        }

        Ok(())
    }

    fn save(&self) -> Result<()> {
        let mut bak_path = self.path.clone();
        bak_path.set_extension("toml.bak");
        std::fs::copy(&self.path, &bak_path)
            .with_context(|| format!("Failed to create backup at {}", bak_path.display()))?;
        println!("Backup saved to: {}", bak_path.display());

        std::fs::write(&self.path, self.doc.to_string()).with_context(|| {
            format!("Failed to write updated config to {}", self.path.display())
        })?;

        Ok(())
    }

    fn print_report(&self) {
        if !self.report.is_empty() {
            println!("\nChanges applied:");
            for r in &self.report {
                println!("  • {}", r);
            }
        }
    }

    fn ask_user(&self, question: &str, options: &[&str], default: usize) -> usize {
        if !self.interactive {
            return default;
        }

        loop {
            print!("{}", question);
            let _ = io::stdout().flush();

            let mut input = String::new();
            if io::stdin().read_line(&mut input).is_err() {
                return default;
            }

            let trimmed = input.trim().to_lowercase();
            if trimmed.is_empty() {
                return default;
            }

            for (i, opt) in options.iter().enumerate() {
                if trimmed == opt.to_lowercase() {
                    return i;
                }
            }

            for (i, opt) in options.iter().enumerate() {
                if opt.to_lowercase().starts_with(&trimmed) {
                    return i;
                }
            }

            println!(
                "Invalid option. Please choose one of: {}",
                options.join(", ")
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn updater(config: &str) -> ConfigUpdater {
        ConfigUpdater {
            path: PathBuf::from("config.toml"),
            doc: config.parse().unwrap(),
            interactive: false,
            dry_run: true,
            changed: false,
            report: Vec::new(),
        }
    }

    #[test]
    fn rejects_newer_schema_without_modifying_it() {
        let mut updater = updater("schema_version = 99\n");
        let error = updater.apply_migrations().unwrap_err();

        assert!(
            error
                .to_string()
                .contains("newer than the supported version")
        );
        assert_eq!(updater.detect_version(), 99);
        assert!(!updater.changed);
    }

    #[test]
    fn migrates_legacy_schema_to_current_version() {
        let mut updater = updater("icons = false\n");
        updater.apply_migrations().unwrap();

        assert_eq!(updater.detect_version(), CURRENT_SCHEMA_VERSION);
        assert!(updater.doc["hyperlinks"].is_table());
        assert!(updater.changed);
    }
}
