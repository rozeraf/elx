#![cfg(unix)]

use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT_ID: AtomicUsize = AtomicUsize::new(0);

struct Fixture(PathBuf);

impl Fixture {
    fn new(files: &[&str], config: &str) -> Self {
        let root = std::env::temp_dir().join(format!(
            "elx-filter-test-{}-{}",
            std::process::id(),
            NEXT_ID.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir_all(root.join("files")).unwrap();
        for file in files {
            let path = root.join("files").join(file);
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(path, "").unwrap();
        }
        std::fs::create_dir_all(root.join("config/elx")).unwrap();
        std::fs::write(root.join("config/elx/config.toml"), config).unwrap();
        Self(root)
    }

    fn write(&self, name: &str, content: &str) {
        std::fs::write(self.0.join("files").join(name), content).unwrap();
    }

    fn list(&self, args: &[&str]) -> Vec<String> {
        self.list_at("", args)
    }

    fn list_at(&self, sub: &str, args: &[&str]) -> Vec<String> {
        let mut command = Command::new(env!("CARGO_BIN_EXE_elx"));
        for (key, _) in std::env::vars_os() {
            if key.to_string_lossy().starts_with("ELX_") {
                command.env_remove(key);
            }
        }
        let output = command
            .env("XDG_CONFIG_HOME", self.0.join("config"))
            .args(["-T", "--no-icons", "--no-color", "--no-hyperlinks"])
            .args(args)
            .arg(self.0.join("files").join(sub))
            .output()
            .unwrap();
        assert!(output.status.success(), "{:?}", output.stderr);
        String::from_utf8(output.stdout)
            .unwrap()
            .lines()
            .map(|line| {
                line.trim_start_matches(|c: char| " │├└─".contains(c))
                    .to_string()
            })
            .collect()
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        std::fs::remove_dir_all(&self.0).unwrap();
    }
}

fn has(list: &[String], name: &str) -> bool {
    list.iter().any(|line| line == name)
}

#[test]
fn cli_negation_reincludes_config_pattern() {
    let fixture = Fixture::new(
        &[".env", ".env.example", "a.log"],
        "ignore = [\".env*\", \"*.log\"]\n",
    );
    let list = fixture.list(&["-a"]);
    assert!(
        !has(&list, ".env") && !has(&list, ".env.example") && !has(&list, "a.log"),
        "{list:?}"
    );
    let list = fixture.list(&["-a", "--ignore", "!.env.example"]);
    assert!(
        has(&list, ".env.example") && !has(&list, ".env") && !has(&list, "a.log"),
        "{list:?}"
    );
}

#[test]
fn patterns_with_a_slash_are_anchored_to_the_listed_root() {
    let fixture = Fixture::new(&["sub/keep.txt", "other/sub/keep.txt"], "");
    let list = fixture.list(&["--ignore", "sub/keep.txt"]);
    assert_eq!(
        list.iter().filter(|l| *l == "keep.txt").count(),
        1,
        "{list:?}"
    );
    let list = fixture.list(&["--ignore", "keep.txt"]);
    assert!(!has(&list, "keep.txt"), "{list:?}");
}

#[test]
fn listed_root_is_never_filtered_by_its_own_pattern() {
    let fixture = Fixture::new(&["target/built"], "ignore = [\"target\"]\n");
    assert!(has(&fixture.list_at("target", &[]), "built"));
    assert!(!has(&fixture.list(&[]), "target"));
}

#[test]
fn elxignore_hides_entries_unless_unrestricted() {
    let fixture = Fixture::new(&["secret", "nested/also-secret", "nested/visible"], "");
    fixture.write(".elxignore", "secret\nalso-secret\n");
    let list = fixture.list(&[]);
    assert!(
        !has(&list, "secret") && !has(&list, "also-secret") && has(&list, "visible"),
        "{list:?}"
    );
    for flag in ["-u", "-aa"] {
        let list = fixture.list(&[flag]);
        assert!(
            has(&list, "secret") && has(&list, "also-secret") && has(&list, ".elxignore"),
            "{flag}: {list:?}"
        );
    }
}

#[test]
fn unrestricted_and_no_config_ignore_drop_config_patterns() {
    let fixture = Fixture::new(&["a.log", "b.tmp", ".hidden"], "ignore = [\"*.log\"]\n");
    let list = fixture.list(&[]);
    assert!(!has(&list, "a.log") && !has(&list, ".hidden"));
    let list = fixture.list(&["--no-config-ignore", "--ignore", "*.tmp"]);
    assert!(
        has(&list, "a.log") && !has(&list, "b.tmp") && !has(&list, ".hidden"),
        "{list:?}"
    );
    let list = fixture.list(&["-u"]);
    assert!(has(&list, "a.log") && has(&list, ".hidden"), "{list:?}");
}

#[test]
fn only_keeps_matching_files_and_the_dirs_leading_to_them() {
    let fixture = Fixture::new(
        &[
            "src/main.rs",
            "src/notes.txt",
            "docs/guide.md",
            "Cargo.toml",
        ],
        "",
    );
    let list = fixture.list(&["--only", "*.rs,*.toml"]);
    assert_eq!(list, ["src", "main.rs", "Cargo.toml"]);
    let list = fixture.list_at("", &["--only", "*.rs"]);
    assert!(!has(&list, "docs") && !has(&list, "notes.txt"), "{list:?}");
}

#[test]
fn type_filters_in_tree_and_flat_views() {
    let fixture = Fixture::new(&["a/b/file", "top.txt"], "");
    for dir in ["a/c", "empty"] {
        std::fs::create_dir_all(fixture.0.join("files").join(dir)).unwrap();
    }
    let list = fixture.list(&["-D"]);
    assert_eq!(list, ["a", "b", "c", "empty"]);
    let list = fixture.list(&["--files-only"]);
    assert_eq!(list, ["a", "b", "file", "top.txt"]);
    let list = fixture.list(&["--files-only", "--depth", "1"]);
    assert_eq!(list, ["top.txt"]);
    let list = fixture.list(&["-D", "--only", "b"]);
    assert_eq!(list, ["a", "b"]);
}
