<div align="center">

# 📌 Waypin

**A sleek clipboard viewer for Wayland/X11 with GTK3**

[![License: GPL v3](https://img.shields.io/badge/License-GPLv3-blue.svg)](https://www.gnu.org/licenses/gpl-3.0)
[![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)](https://www.rust-lang.org)
[![GTK3](https://img.shields.io/badge/GTK-3.0-green.svg)](https://gtk.org)
[![Wayland](https://img.shields.io/badge/Wayland-supported-purple.svg)](https://wayland.freedesktop.org)

*Instantly preview and manage your clipboard content with a beautiful, responsive interface*

</div>

## Running

Just launch `waypin` with no arguments — it reads the clipboard once, shows a preview window, and exits when you close it.

## Background mode (Hyprland global shortcut)

Build with the `global-shortcuts` feature to enable the background daemon:

```bash
cargo build --release --features global-shortcuts
```

Run `waypin --background` to register the `waypin:toggle` global shortcut and stay running. Each shortcut press opens a fresh, independent preview window. The daemon is event-driven and lazily initializes GTK on the first press, so idle RSS sits around 1–2 MB.

Outside Hyprland, or when no Wayland display is available, `waypin --background` prints a `waypin: background shortcut listener failed: ...` error to stderr and exits with status 1. It does not open a preview window. When started by the supplied systemd service, inspect these errors with:

```bash
journalctl --user -u waypin.service -f
```

### Bind the shortcut in `hyprland.conf`

```conf
bind = SUPER, V, global, waypin:toggle
```

### systemd user service (recommended)

A `systemd/waypin.service` unit is provided. The unit self-skips on non-Hyprland sessions via `ConditionPathExists=/usr/bin/Hyprland` and `ConditionEnvironment=XDG_CURRENT_DESKTOP=Hyprland`, so it is safe to install unconditionally.

```bash
# After installing the package (or copying the unit manually to
# /usr/lib/systemd/user/ or ~/.config/systemd/user/):
systemctl --user enable --now waypin.service
```

Resource caps force the daemon into ultra-low-power mode: `MemoryMax=48M`, `Nice=19`, `CPUWeight=20`. The daemon restarts on crash (`Restart=on-failure`).
