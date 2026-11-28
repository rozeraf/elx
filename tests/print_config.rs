use std::process::Command;

#[test]
fn print_config_shows_merged_settings() {
    let dir = std::env::temp_dir().join(format!("elx-print-config-test-{}", std::process::id()));
    std::fs::create_dir_all(dir.join("elx")).unwrap();
    std::fs::write(
        dir.join("elx/config.toml"),
        "ignore = [\"*.log\"]\n[tree]\ngit_ignore = \"dim\"\ncollapse = [\"target\"]\n",
    )
    .unwrap();
    let mut command = Command::new(env!("CARGO_BIN_EXE_elx"));
    for (key, _) in std::env::vars_os() {
        if key.to_string_lossy().starts_with("ELX_") {
            command.env_remove(key);
        }
    }
    let output = command
        .env("XDG_CONFIG_HOME", &dir)
        .args(["--print-config", "-T", "-t", "--ignore", "*.tmp"])
        .output()
        .unwrap();
    std::fs::remove_dir_all(&dir).unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    for line in [
        "# active sections: [tree], [when_not_tty]",
        "git_ignore = \"dim\"",
        "ignore = [\"*.log\", \"*.tmp\"]",
        "collapse = [\"target\"]",
        "icons = false",
        "by = \"modified\"",
        "reverse = true",
    ] {
        assert!(
            stdout.lines().any(|l| l == line),
            "missing {line:?} in:\n{stdout}"
        );
    }
}
