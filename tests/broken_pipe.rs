#![cfg(unix)]

use std::os::unix::process::ExitStatusExt;
use std::process::{Command, Stdio};

#[test]
fn closed_stdout_ends_quietly_instead_of_panicking() {
    let dir = std::env::temp_dir().join(format!("elx-pipe-test-{}", std::process::id()));
    std::fs::create_dir_all(dir.join("config")).unwrap();
    for i in 0..4000 {
        std::fs::write(dir.join(format!("file-with-a-long-name-{i:05}")), "").unwrap();
    }
    let mut child = Command::new(env!("CARGO_BIN_EXE_elx"))
        .env("XDG_CONFIG_HOME", dir.join("config"))
        .args(["-l", "--no-icons", "--no-color"])
        .arg(&dir)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    drop(child.stdout.take());
    let output = child.wait_with_output().unwrap();
    std::fs::remove_dir_all(&dir).unwrap();
    assert!(
        output.stderr.is_empty(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(output.status.signal(), Some(libc::SIGPIPE));
}
