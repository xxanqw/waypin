use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::process::Command;

use tempfile::TempDir;

fn mock_wl_paste(script: &str) -> TempDir {
    let temp_dir = tempfile::tempdir().expect("failed to create temporary directory");
    let wl_paste = temp_dir.path().join("wl-paste");
    fs::write(&wl_paste, script).expect("failed to create mock wl-paste");

    let mut permissions = fs::metadata(&wl_paste)
        .expect("failed to read mock wl-paste metadata")
        .permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&wl_paste, permissions).expect("failed to make mock wl-paste executable");

    temp_dir
}

fn waypin_command(mock_bin_dir: &TempDir) -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_waypin"));
    command.env("PATH", mock_bin_dir.path());
    command
}

#[test]
fn arguments_print_usage_without_initializing_gtk() {
    let output = Command::new(env!("CARGO_BIN_EXE_waypin"))
        .arg("--help")
        .output()
        .expect("failed to execute waypin");

    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("Usage:"));
}

#[test]
fn empty_clipboard_is_reported_without_initializing_gtk() {
    let mock_bin_dir = mock_wl_paste("#!/bin/sh\nexit 1\n");
    let output = waypin_command(&mock_bin_dir)
        .output()
        .expect("failed to execute waypin");

    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr)
        .contains("Could not retrieve clipboard types or clipboard is empty."));
}

#[test]
fn unsupported_clipboard_is_reported_without_initializing_gtk() {
    let mock_bin_dir = mock_wl_paste("#!/bin/sh\nprintf 'application/pdf\\n'\n");
    let output = waypin_command(&mock_bin_dir)
        .output()
        .expect("failed to execute waypin");

    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr)
        .contains("Clipboard does not contain supported image or text types."));
}
