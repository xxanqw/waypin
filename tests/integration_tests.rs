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

#[cfg(feature = "global-shortcuts")]
#[test]
fn background_mode_fails_without_wayland_before_initializing_gtk() {
    let bin = env!("CARGO_BIN_EXE_waypin");
    let runtime_dir = tempfile::tempdir().expect("failed to create empty runtime directory");

    let output = Command::new(bin)
        .arg("--background")
        .env_remove("WAYLAND_DISPLAY")
        .env_remove("DISPLAY")
        .env("XDG_RUNTIME_DIR", runtime_dir.path())
        .output()
        .expect("failed to execute waypin");

    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.starts_with(
            "waypin: background shortcut listener failed: could not connect to the Wayland display:"
        ),
        "unexpected stderr: {stderr}"
    );
    assert!(output.stdout.is_empty());
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

#[cfg(feature = "pin-on-top")]
mod pin_on_top_tests {
    #[test]
    fn layer_shell_probe_does_not_panic() {
        if std::env::var("DISPLAY").is_err() && std::env::var("WAYLAND_DISPLAY").is_err() {
            return;
        }
        if gtk::init().is_err() {
            return;
        }
        let _supported = gtk_layer_shell::is_supported();
    }

    #[test]
    fn pin_on_top_binary_runs_without_panic() {
        let bin = env!("CARGO_BIN_EXE_waypin");
        let output = std::process::Command::new("sh")
            .args(["-c", &format!("DISPLAY=:99 timeout 5 {bin}")])
            .output();

        let Ok(output) = output else {
            return;
        };
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            !stderr.contains("panicked"),
            "pin-on-top binary panicked; stderr: {stderr}"
        );
    }
}
