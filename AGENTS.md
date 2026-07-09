# AGENTS.md

Compact guidance for OpenCode sessions working in this repo.

## Project

`waypin` — a GTK3 clipboard viewer for Wayland/X11 written in Rust. Reads the clipboard via `wl-paste` and writes via `wl-copy` (from `wl-clipboard`), then shows text or image in a GTK dialog. Package has both a binary (`src/main.rs` → `waypin`) and a library (`src/lib.rs` → `waypin_lib`) that holds testable logic.

## System dependencies (required at build and runtime)

GTK3, gdk-pixbuf-2.0, `wl-clipboard` (provides `wl-paste`/`wl-copy`).

- Debian/Ubuntu: `sudo apt-get install -y libgtk-3-dev libgdk-pixbuf2.0-dev wl-clipboard xvfb`
- Arch: `sudo pacman -S gtk3 gdk-pixbuf2 wl-clipboard pkg-config`

Build will fail at the `system-deps` / `pkg-config` step without the GTK3 dev packages.

## Commands

- Build: `cargo build` (debug) / `cargo build --release`
- Build with the global-shortcut daemon: `cargo build --features global-shortcuts` (pulls in `wayland-client`/`wayland-backend`/`wayland-protocols`/`wayland-scanner`; runtime Hyprland not required for the build, only for shortcut delivery).
- Run: `cargo run` (one-shot, no args — read clipboard once, show one window, exit on close).
- Run the background daemon: `cargo run --features global-shortcuts -- --background` (only useful under Hyprland).
- Unit tests (no display needed): `cargo test --lib`
- Integration tests (need a display, GTK initializes): `xvfb-run -a cargo test --test integration_tests`
- Only GTK-display-gated tests require the `gui-tests` feature: `cargo test --features gui-tests` — and only run when `DISPLAY` or `WAYLAND_DISPLAY` is set (tests self-skip otherwise).
- Full local verification mirroring CI: `cargo test --lib && xvfb-run -a cargo test --test integration_tests && cargo build --release && cargo build --features global-shortcuts`

There is no lint/typecheck command configured (CI installs `rustfmt`/`clippy` components but does not run them). If asked to lint, use `cargo fmt --check` and `cargo clippy -- -D warnings`.

## Non-obvious facts

- All files under `tests/` are separate integration-test targets. `tests/clipboard_mock.rs` is **not** a helper imported by `integration_tests.rs` — running `cargo test` will compile and run its internal `mod tests` as a standalone suite.
- `[profile.test]` sets `opt-level = 1` — debug tests are partially optimized; don't be surprised by slower compile times.
- The binary **no longer** force-sets `GDK_BACKEND=x11` (was `src/main.rs:203`, removed when the global-shortcut daemon landed). GTK autodetects Wayland/X11. CI's xvfb-only env has no `WAYLAND_DISPLAY` and no `wayland-*` socket in `XDG_RUNTIME_DIR`, so it falls back to X11 cleanly. On a developer Workland machine, `test_empty_clipboard_handling` may fail because `wl-paste` finds the live `wayland-0` socket via `XDG_RUNTIME_DIR` even with `WAYLAND_DISPLAY` unset — run with `XDG_RUNTIME_DIR` pointed at an empty dir to reproduce CI locally.
- `PKGBUILD` builds from the `restoring` git branch with `--locked` and the `global-shortcuts` feature; keep `Cargo.lock` in sync with `Cargo.toml` version bumps and use that branch for Arch packaging. It installs `systemd/waypin.service` to `/usr/lib/systemd/user/`; the unit self-skips on non-Hyprland sessions via `ConditionPathExists=/usr/bin/Hyprland` + `ConditionEnvironment=XDG_CURRENT_DESKTOP=Hyprland`, so the unit file is installed unconditionally — no install-hook env probing.
- CI listens on `main`, `develop`, and `restoring` (push) and `main`/`restoring` (PR). Active development happens on `restoring`.

## Architecture notes

- `src/lib.rs` contains pure logic (`has_mime_type`, `detect_clipboard_content_type`, `get_image_format_from_types`, `copy_image_to_clipboard`, `compute_scaled_dimensions`) plus its unit tests. Prefer adding new testable logic here rather than to `main.rs`.
- `src/main.rs` is GUI + process orchestration. Two runtime modes:
  - **One-shot** (no args): `run_one_shot()` calls `gtk::init()` immediately, builds a `gtk::Application`, fires `activate` once which calls `show_clipboard(&app)`, and quits when the last window closes (`connect_shutdown` + `windows().is_empty()`).
  - **Background** (`--background`, requires `global-shortcuts`): `run_background()` spawns a worker thread running `shortcut::run_listener`, which owns an *independent* `wayland_client::Connection` (decoupled from GDK's), registers `waypin:toggle` via `hyprland_global_shortcuts_manager_v1`, and parks in `EventQueue::blocking_dispatch()` — zero CPU between shortcut events. The worker signals the GLib main thread via a `MainContext::channel`; GTK is **lazily initialized on the first pressed event** so idle RSS stays at ~1–2 MB. Closing preview windows does *not* terminate the daemon — only the compositor dying or systemd stopping the unit does.
  - `show_clipboard(app)` reads via `wl-paste` and dispatches to `build_text_window` / `build_image_window`. Each call builds a fresh `ApplicationWindow` under one shared `Application`, so the daemon opens multiple simultaneous previews.
- `src/shortcut.rs` (feature-gated) vendors `protocols/hyprland-global-shortcuts-v1.xml` and uses `wayland_scanner` proc-macros. The shortcut `app_id:id` is `waypin:toggle`; users must add `bind = SUPER, V, global, waypin:toggle` to `hyprland.conf` — the protocol does not let the client pick the keybinding.
- Image format priority is PNG > JPEG > GIF (`src/lib.rs:49`); text detection accepts several MIME aliases. `text/uri-list` is intentionally ignored as "file list".