#![cfg(unix)]

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT_ID: AtomicUsize = AtomicUsize::new(0);

struct Fixture(PathBuf);

impl Fixture {
    fn new(config: &str) -> Self {
        let root = std::env::temp_dir().join(format!(
            "elx-collapse-test-{}-{}",
            std::process::id(),
            NEXT_ID.fetch_add(1, Ordering::Relaxed)
        ));
        for file in [
            "files/target/deep/built",
            "files/src/main.rs",
            "files/nested/target/obj",
        ] {
            let path = root.join(file);
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(path, "").unwrap();
        }
        std::fs::create_dir_all(root.join("config/elx")).unwrap();
        std::fs::write(root.join("config/elx/config.toml"), config).unwrap();
        Self(root)
    }

    fn files(&self) -> PathBuf {
        self.0.join("files")
    }

    fn output(&self, cwd: &Path, args: &[&str]) -> String {
        let mut command = Command::new(env!("CARGO_BIN_EXE_elx"));
        for (key, _) in std::env::vars_os() {
            if key.to_string_lossy().starts_with("ELX_") {
                command.env_remove(key);
            }
        }
        let output = command
            .current_dir(cwd)
            .env("XDG_CONFIG_HOME", self.0.join("config"))
            .args(["--no-icons", "--no-color", "--no-hyperlinks"])
            .args(args)
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
fn tree_collapses_matching_directories_at_any_depth() {
    let fixture = Fixture::new("collapse = [\"target\"]\n");
    let files = fixture.files();
    let abs = files.to_str().unwrap();
    for (cwd, path) in [(fixture.0.as_path(), abs), (files.as_path(), ".")] {
        let output = fixture.output(cwd, &["-T", path]);
        assert!(output.contains("target …"), "{output}");
        assert!(!output.contains("deep"), "{output}");
        assert!(!output.contains("obj"), "{output}");
        assert!(output.contains("main.rs"), "{output}");
    }
}

#[test]
fn collapse_flags_override_config() {
    let fixture = Fixture::new("[tree]\ncollapse = [\"target\"]\n");
    let files = fixture.files();
    let path = files.to_str().unwrap();
    let output = fixture.output(&fixture.0, &["-T", "--no-collapse", path]);
    assert!(output.contains("deep") && !output.contains('…'), "{output}");
    let output = fixture.output(&fixture.0, &["-T", "--collapse", "src", path]);
    assert!(
        output.contains("src …") && !output.contains("main.rs"),
        "{output}"
    );
    assert!(!output.contains("deep"), "{output}");
}

#[test]
fn collapse_marker_only_appears_in_tree_view() {
    let fixture = Fixture::new("collapse = [\"target\"]\n");
    let files = fixture.files();
    let output = fixture.output(&fixture.0, &["-1", files.to_str().unwrap()]);
    assert!(
        output.contains("target") && !output.contains('…'),
        "{output}"
    );
}
