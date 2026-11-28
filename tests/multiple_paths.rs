#![cfg(unix)]

use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT_ID: AtomicUsize = AtomicUsize::new(0);

struct Fixture(PathBuf);

impl Fixture {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!(
            "elx-multi-test-{}-{}",
            std::process::id(),
            NEXT_ID.fetch_add(1, Ordering::Relaxed)
        ));
        for file in ["b.toml", "a.toml", "src/main.rs", "tests/it.rs"] {
            let path = root.join(file);
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(path, "").unwrap();
        }
        std::fs::create_dir_all(root.join("config")).unwrap();
        Self(root)
    }

    fn run(&self, args: &[&str]) -> (String, String, i32) {
        let mut command = Command::new(env!("CARGO_BIN_EXE_elx"));
        for (key, _) in std::env::vars_os() {
            if key.to_string_lossy().starts_with("ELX_") {
                command.env_remove(key);
            }
        }
        let output = command
            .current_dir(&self.0)
            .env("XDG_CONFIG_HOME", self.0.join("config"))
            .args(["--no-icons", "--no-color", "--no-hyperlinks", "-1"])
            .args(args)
            .output()
            .unwrap();
        (
            String::from_utf8(output.stdout).unwrap(),
            String::from_utf8(output.stderr).unwrap(),
            output.status.code().unwrap_or(-1),
        )
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        std::fs::remove_dir_all(&self.0).unwrap();
    }
}

#[test]
fn files_come_first_then_each_directory_under_a_header() {
    let fixture = Fixture::new();
    let (out, _, code) = fixture.run(&["tests", "b.toml", "src", "a.toml"]);
    assert_eq!(code, 0);
    assert_eq!(out, "a.toml\nb.toml\n\nsrc:\nmain.rs\n\ntests:\nit.rs\n");
}

#[test]
fn file_operands_keep_their_path_and_a_single_directory_has_no_header() {
    let fixture = Fixture::new();
    assert_eq!(fixture.run(&["src/main.rs"]).0, "src/main.rs\n");
    assert_eq!(fixture.run(&["src"]).0, "main.rs\n");
}

#[test]
fn missing_operands_are_reported_and_the_rest_is_listed() {
    let fixture = Fixture::new();
    let (out, err, code) = fixture.run(&["a.toml", "missing", "src"]);
    assert_eq!(code, 1);
    assert!(err.contains("cannot access 'missing'"), "{err}");
    assert_eq!(out, "a.toml\n\nsrc:\nmain.rs\n");
}
