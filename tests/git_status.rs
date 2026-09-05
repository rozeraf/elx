#![cfg(unix)]

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT_ID: AtomicUsize = AtomicUsize::new(0);

struct Repo(PathBuf);

impl Repo {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!(
            "elx-git-test-{}-{}",
            std::process::id(),
            NEXT_ID.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir(&root).unwrap();
        std::fs::create_dir(root.join("config")).unwrap();
        std::fs::create_dir(root.join("repo")).unwrap();
        let repo = Self(root);
        repo.git(&["init", "-q"]);
        repo.write("nested/tracked", "before\n");
        repo.write("clean/tracked", "unchanged\n");
        repo.write(".gitignore", "ignored\ncache/\n");
        std::os::unix::fs::symlink("missing-before", repo.path().join("link")).unwrap();
        repo.git(&["add", "."]);
        repo.git(&["commit", "-qm", "initial"]);
        repo
    }

    fn path(&self) -> PathBuf {
        self.0.join("repo")
    }

    fn command(&self, program: &str) -> Command {
        let mut command = Command::new(program);
        for (key, _) in std::env::vars_os() {
            if key.to_string_lossy().starts_with("ELX_")
                || key.to_string_lossy().starts_with("GIT_")
            {
                command.env_remove(key);
            }
        }
        command
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .env("GIT_CONFIG_GLOBAL", "/dev/null")
            .env("XDG_CONFIG_HOME", self.0.join("config"));
        command
    }

    fn git(&self, args: &[&str]) {
        let output = self
            .command("git")
            .current_dir(self.path())
            .args([
                "-c",
                "user.name=Test",
                "-c",
                "user.email=test@example.invalid",
                "-c",
                "commit.gpgSign=false",
                "-c",
                "core.hooksPath=/dev/null",
            ])
            .args(args)
            .output()
            .unwrap();
        assert!(output.status.success(), "{:?}", output.stderr);
    }

    fn write(&self, name: &str, content: &str) {
        let path = self.path().join(name);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, content).unwrap();
    }

    fn list(&self, path: &Path, extra: &[&str]) -> String {
        let output = self
            .command(env!("CARGO_BIN_EXE_elx"))
            .args(["-l", "--no-icons", "--no-color", "--no-hyperlinks"])
            .args(extra)
            .arg(path)
            .output()
            .unwrap();
        assert!(output.status.success(), "{:?}", output.stderr);
        String::from_utf8(output.stdout).unwrap()
    }
}

impl Drop for Repo {
    fn drop(&mut self) {
        std::fs::remove_dir_all(&self.0).unwrap();
    }
}

fn status(output: &str, name: &str) -> Option<char> {
    let line = output
        .lines()
        .find(|line| {
            line.split_whitespace().last() == Some(name) || line.contains(&format!(" {name} -> "))
        })
        .unwrap_or_else(|| panic!("missing {name}: {output}"));
    let first = line.chars().next().unwrap();
    (first != ' ').then_some(first)
}

#[test]
fn individual_files_and_directory_aliases_keep_git_status() {
    let repo = Repo::new();
    repo.write("nested/tracked", "after\n");
    let alias = repo.0.join("alias");
    std::os::unix::fs::symlink(repo.path(), &alias).unwrap();
    for path in [
        repo.path().join("nested"),
        repo.path().join("nested/tracked"),
        alias.join("nested"),
        alias.join("nested/tracked"),
    ] {
        assert_eq!(status(&repo.list(&path, &[]), "tracked"), Some('M'));
    }
    std::fs::remove_file(repo.path().join("link")).unwrap();
    std::os::unix::fs::symlink("missing-after", repo.path().join("link")).unwrap();
    assert_eq!(
        status(&repo.list(&alias.join("link"), &[]), "link"),
        Some('M')
    );
}

#[test]
fn directory_summaries_include_deleted_staged_and_untracked_files() {
    let repo = Repo::new();
    std::fs::remove_file(repo.path().join("nested/tracked")).unwrap();
    repo.write("new/deep/file", "untracked\n");
    let output = repo.list(&repo.path(), &[]);
    assert_eq!(status(&output, "nested"), Some('M'));
    assert_eq!(status(&output, "new"), Some('?'));
    assert_eq!(status(&output, "clean"), None);
    repo.git(&["add", "nested/tracked"]);
    assert_eq!(status(&repo.list(&repo.path(), &[]), "nested"), Some('M'));
    assert_eq!(status(&repo.list(&repo.path(), &["-T"]), "file"), Some('?'));
}

#[test]
fn ignored_files_and_collapsed_directories_show_ignored_status() {
    let repo = Repo::new();
    repo.write("ignored", "ignored\n");
    repo.write("cache/deep/file", "ignored\n");
    repo.write("clean/ignored", "ignored\n");
    let output = repo.list(&repo.path(), &["-G"]);
    assert_eq!(status(&output, "ignored"), Some('!'));
    assert_eq!(status(&output, "cache"), Some('!'));
    assert_eq!(status(&output, "clean"), None);
    assert_eq!(
        status(&repo.list(&repo.path().join("cache/deep"), &["-G"]), "file"),
        Some('!')
    );
    assert_eq!(
        status(&repo.list(&repo.path().join("ignored"), &[]), "ignored"),
        Some('!')
    );
    let output = repo.list(&repo.path(), &[]);
    assert!(!output.contains("ignored"));
    assert!(!output.contains("cache"));
}
