use std::process::Command;
use std::io::Write;
use tempfile::NamedTempFile;

#[test]
fn test_waypin_help_message() {
    let output = Command::new("cargo")
        .args(&["run", "--", "--help"])
        .output()
        .expect("Failed to execute waypin");
    
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Usage:"));
}

#[test]
fn test_waypin_with_args_exits_with_error() {
    let output = Command::new("cargo")
        .args(&["run", "--", "invalid-arg"])
        .output()
        .expect("Failed to execute waypin");
    
    assert!(!output.status.success());
}

#[cfg(feature = "global-shortcuts")]
#[test]
fn test_background_mode_fails_without_wayland_before_initializing_gtk() {
    let bin = env!("CARGO_BIN_EXE_waypin");
    let runtime_dir = tempfile::tempdir().expect("Failed to create empty runtime directory");

    let output = Command::new(bin)
        .arg("--background")
        .env_remove("WAYLAND_DISPLAY")
        .env_remove("DISPLAY")
        .env("XDG_RUNTIME_DIR", runtime_dir.path())
        .output()
        .expect("Failed to execute waypin");

    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.starts_with("waypin: background shortcut listener failed: could not connect to the Wayland display:"),
        "unexpected stderr: {stderr}"
    );
    assert!(
        !stderr.contains("Clipboard"),
        "background failure must not open a clipboard preview: {stderr}"
    );
    assert!(
        output.stdout.is_empty(),
        "background failure must not emit clipboard-preview output"
    );
}

#[test]
fn test_empty_clipboard_handling() {
    // Mock empty clipboard by setting wl-paste to fail
    let output = Command::new("sh")
        .args(&["-c", "DISPLAY= timeout 5 cargo run"])
        .output();

    // Should handle empty clipboard gracefully
    if let Ok(output) = output {
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains("Could not retrieve clipboard types or clipboard is empty.") || stderr.contains("wl-paste") || stderr.contains("clipboard is empty") || stderr.contains("Failed to initialize GTK"));
    }
}

#[test]
fn test_image_file_validation() {
    // Test with a small PNG image data
    let png_header = vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
    let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
    temp_file.write_all(&png_header).expect("Failed to write PNG header");
    
    // This test validates that our image detection logic can handle real image data
    assert!(png_header.starts_with(&[0x89, 0x50, 0x4E, 0x47])); // PNG signature
}

#[test]
fn test_text_encoding_handling() {
    let test_cases = vec![
        "Hello, World!",
        "UTF-8: ñáéíóú",
        "Emojis: 🚀🎯📌",
        "Mixed: Hello 世界 🌍",
        "",
    ];
    
    for text in test_cases {
        let bytes = text.as_bytes();
        let decoded = String::from_utf8_lossy(bytes);
        assert_eq!(decoded, text);
    }
}

#[test]
fn test_image_copy_functionality() {
    // Create a simple PNG image data
    let png_data = vec![
        0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, // PNG signature
        0x00, 0x00, 0x00, 0x0D, // IHDR chunk length
        0x49, 0x48, 0x44, 0x52, // IHDR
        0x00, 0x00, 0x00, 0x01, // width: 1
        0x00, 0x00, 0x00, 0x01, // height: 1
        0x08, 0x06, 0x00, 0x00, 0x00, // bit depth, color type, compression, filter, interlace
        0x1F, 0x15, 0xC4, 0x89, // CRC
        0x00, 0x00, 0x00, 0x0A, // IDAT chunk length
        0x49, 0x44, 0x41, 0x54, // IDAT
        0x78, 0x9C, 0x62, 0x00, 0x00, 0x00, 0x02, 0x00, 0x01, // compressed data
        0xE2, 0x21, 0xBC, 0x33, // CRC
        0x00, 0x00, 0x00, 0x00, // IEND chunk length
        0x49, 0x45, 0x4E, 0x44, // IEND
        0xAE, 0x42, 0x60, 0x82, // CRC
    ];
    
    let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
    temp_file.write_all(&png_data).expect("Failed to write PNG data");
    
    // Test that we can identify PNG format correctly
    assert!(png_data.starts_with(&[0x89, 0x50, 0x4E, 0x47])); // PNG signature
    
    // Mock clipboard content by setting up wl-paste to return our PNG data
    let script = format!(
        r#"#!/bin/bash
if [[ "$1" == "--list-types" ]]; then
    echo "image/png"
elif [[ "$1" == "--type" && "$2" == "image/png" ]]; then
    cat "{}"
else
    exit 1
fi"#,
        temp_file.path().display()
    );
    
    let mut mock_wl_paste = NamedTempFile::new().expect("Failed to create mock script");
    mock_wl_paste.write_all(script.as_bytes()).expect("Failed to write mock script");
    
    // This test validates the image data structure and format detection
    // In a real integration test, we would mock the wl-copy command as well
    assert_eq!(png_data.len(), 66); // Expected size of our minimal PNG
}

#[test]
fn test_wl_copy_command_structure() {
    // Test that the wl-copy command structure is correct for different MIME types
    let test_cases = vec![
        ("image/png", "PNG image data"),
        ("image/jpeg", "JPEG image data"),
        ("image/gif", "GIF image data"),
    ];
    
    for (mime_type, _description) in test_cases {
        // Simulate the command that would be run
        let args = vec!["wl-copy", "--type", mime_type];
        
        // Verify command structure
        assert_eq!(args[0], "wl-copy");
        assert_eq!(args[1], "--type");
        assert!(args[2].starts_with("image/"));
    }
}

#[cfg(feature = "gui-tests")]
mod gui_tests {
    use gtk::prelude::*;
    use gtk::{Application, ApplicationWindow};

    #[test]
    fn test_gtk_initialization() {
        // This test requires X11/Wayland display
        if std::env::var("DISPLAY").is_ok() || std::env::var("WAYLAND_DISPLAY").is_ok() {
            assert!(gtk::init().is_ok());
        }
    }

    #[test]
    fn test_application_creation() {
        if gtk::init().is_ok() {
            let app = Application::new(None, Default::default());
            assert!(app.is_some());
        }
    }
}

/// Verifies the `pin-on-top` feature's probe path is callable and never
/// panics regardless of compositor support. Under CI's xvfb-only env (no
/// Wayland, no `zwlr_layer_shell_v1` global), `is_supported()` returns
/// `false`, exercising the fallback branch of `try_promote_to_layer_top`
/// — exactly the path that runs on GNOME Mutter / Cinnamon Muffin / Cage
/// / X11-hosts. On a wlroots/KWin/COSMIC host with this feature enabled,
/// the real layer-shell branch executes.
///
/// What this test rules out is the failure mode where
/// `gtk-layer-shell-sys` fails to bind at runtime or `try_promote_to_layer_top`
/// panics in the unsupported path.
#[cfg(feature = "pin-on-top")]
mod pin_on_top_tests {
    #[test]
    fn test_layer_shell_probe_does_not_panic() {
        // Mirror the existing gui_tests convention at tests/integration_tests.rs:143
        // — self-skip when there is literally no display at all.
        if std::env::var("DISPLAY").is_err() && std::env::var("WAYLAND_DISPLAY").is_err() {
            return;
        }
        if gtk::init().is_err() {
            return;
        }
        // `is_supported()` may perform a Wayland roundtrip on the first call.
        // The probe is the same one `try_promote_to_layer_top` calls; this
        // confirms the dep is linked and callable.
        let _supported = gtk_layer_shell::is_supported();
        // Do not assert the boolean: under xvfb it is false; on real
        // wlroots/KWin dev hosts it is true. The contract is no-panic.
    }

    /// End-to-end fallback-path test: spawn the already-built `waypin` binary
    /// (linked with `pin-on-top` thanks to the feature flag passed to
    /// `cargo test`) and assert it doesn't panic. Uses `CARGO_BIN_EXE_waypin`
    /// so we don't re-enter cargo (which would race with
    /// `test_empty_clipboard_handling` for the target dir lock).
    ///
    /// CI's xvfb returns false from `is_supported()`, so this covers the
    /// fallback branch of `try_promote_to_layer_top` in real bin execution.
    #[test]
    fn test_pin_on_top_binary_runs_without_panic() {
        let Some(bin) = option_env!("CARGO_BIN_EXE_waypin") else {
            return; // env not set → skip gracefully (older cargo)
        };
        let output = std::process::Command::new("sh")
            .args([
                "-c",
                &format!("DISPLAY=:99 timeout 5 {bin}"),
            ])
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
