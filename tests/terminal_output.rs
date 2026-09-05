#![cfg(unix)]

use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT_ID: AtomicUsize = AtomicUsize::new(0);

struct Fixture(PathBuf);

impl Fixture {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "elx-terminal-test-{}-{}",
            std::process::id(),
            NEXT_ID.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir(&path).unwrap();
        std::fs::create_dir(path.join("config")).unwrap();
        std::fs::create_dir(path.join("files")).unwrap();
        std::fs::write(path.join("files/evil\x1b[31m\nFAKE"), "").unwrap();
        std::os::unix::fs::symlink("target\x1b]2;injected\x07\nFAKE", path.join("files/link"))
            .unwrap();
        Self(path)
    }

    fn output(&self, args: &[&str]) -> String {
        let mut command = Command::new(env!("CARGO_BIN_EXE_elx"));
        for (key, _) in std::env::vars_os() {
            if key.to_string_lossy().starts_with("ELX_") {
                command.env_remove(key);
            }
        }
        let output = command
            .env("XDG_CONFIG_HOME", self.0.join("config"))
            .args(["--no-icons", "--no-color", "--no-hyperlinks"])
            .args(args)
            .arg(self.0.join("files"))
            .output()
            .unwrap();
        assert!(output.status.success(), "{:?}", output.stderr);
        String::from_utf8(output.stdout).unwrap()
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        std::fs::remove_dir_all(&self.0).unwrap();
    }
}

#[test]
fn filenames_and_symlink_targets_cannot_inject_terminal_controls() {
    let fixture = Fixture::new();
    for mode in [vec!["-1"], vec!["-l"], vec!["-T"], vec!["-lT"]] {
        let output = fixture.output(&mode);
        assert!(!output.contains(['\x1b', '\x07', '\r']));
        assert!(output.contains("evil\\u{1b}[31m\\nFAKE"));
        assert_eq!(output.lines().count(), 2, "{output:?}");
        if mode != ["-1"] {
            assert!(output.contains("target\\u{1b}]2;injected\\u{7}\\nFAKE"));
        }
    }
}

#[test]
fn styling_and_hyperlinks_use_escaped_labels_and_encoded_paths() {
    let fixture = Fixture::new();
    for mode in [vec!["-1"], vec!["-l"], vec!["-T"]] {
        let mut args = mode;
        args.extend(["--color", "--hyperlinks"]);
        let output = fixture.output(&args);
        assert!(output.contains("evil\\u{1b}[31m\\nFAKE"));
        assert!(output.contains("evil%1B%5B31m%0AFAKE"));
        assert!(!output.contains("\x1b]2;injected"));
        assert_eq!(output.lines().count(), 2);
    }
}
