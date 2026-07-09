//! Hyprland global-shortcut listener.
//!
//! Registers the `waypin:toggle` global shortcut via the
//! `hyprland_global_shortcuts_manager_v1` Wayland protocol and calls the
//! provided callback on every `pressed` event. Designed to run as a
//! background thread: the caller passes a closure that signals the GLib main
//! thread to open a new clipboard preview window.
//!
//! The listener thread owns its own `wayland_client::Connection`, completely
//! independent from GDK's Wayland connection, and is parked in
//! `EventQueue::blocking_dispatch()` — zero CPU/wakeups between shortcut
//! events. See AGENTS.md for the rationale.

use std::sync::Arc;
use wayland_client::{
    globals::{registry_queue_init, GlobalListContents},
    protocol::wl_registry,
    Connection, Dispatch, QueueHandle,
};

pub mod protocol {
    #[allow(unused_imports)]
    use wayland_client;
    #[allow(unused_imports)]
    use wayland_client::protocol::*;

    pub mod __interfaces {
        #[allow(unused_imports)]
        use wayland_backend;
        #[allow(unused_imports)]
        use wayland_client::protocol::__interfaces::*;
        wayland_scanner::generate_interfaces!("./protocols/hyprland-global-shortcuts-v1.xml");
    }
    #[allow(unused_imports)]
    use self::__interfaces::*;
    #[allow(unused_imports)]
    use wayland_backend;
    wayland_scanner::generate_client_code!("./protocols/hyprland-global-shortcuts-v1.xml");
}

use protocol::hyprland_global_shortcut_v1::HyprlandGlobalShortcutV1;
use protocol::hyprland_global_shortcuts_manager_v1::HyprlandGlobalShortcutsManagerV1;

pub const APP_ID: &str = "waypin";
pub const SHORTCUT_ID: &str = "toggle";

pub const SHORTCUT_DESCRIPTION: &str = "Show clipboard preview";
pub const SHORTCUT_TRIGGER_DESCRIPTION: &str =
    "Configure in hyprland.lua: hl.bind(\"SUPER + V\", hl.dsp.global(\"waypin:toggle\"))";

struct ListenerState {
    on_pressed: Arc<dyn Fn() + Send + Sync>,
}

impl Dispatch<wl_registry::WlRegistry, GlobalListContents> for ListenerState {
    fn event(
        _: &mut Self,
        _: &wl_registry::WlRegistry,
        _: wl_registry::Event,
        _: &GlobalListContents,
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<HyprlandGlobalShortcutsManagerV1, ()> for ListenerState {
    fn event(
        _: &mut Self,
        _: &HyprlandGlobalShortcutsManagerV1,
        _: protocol::hyprland_global_shortcuts_manager_v1::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<HyprlandGlobalShortcutV1, ()> for ListenerState {
    fn event(
        state: &mut Self,
        _: &HyprlandGlobalShortcutV1,
        event: protocol::hyprland_global_shortcut_v1::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        if matches!(
            event,
            protocol::hyprland_global_shortcut_v1::Event::Pressed { .. }
        ) {
            (state.on_pressed)();
        }
    }
}

/// Block the calling thread, listening for the global shortcut.
///
/// Returns when the Wayland connection is closed or dispatching fails.
/// Pass a `Send + Sync` callback invoked on every `pressed` event.
pub fn run_listener(on_pressed: impl Fn() + Send + Sync + 'static) -> Result<(), String> {
    let on_pressed = Arc::new(on_pressed);

    let conn = Connection::connect_to_env().map_err(|e| {
        format!(
            "could not connect to the Wayland display: {e}. \
             --background requires a running Wayland session (set WAYLAND_DISPLAY)."
        )
    })?;

    let (globals, mut queue) = registry_queue_init::<ListenerState>(&conn)
        .map_err(|e| format!("Failed to retrieve Wayland global list: {e}"))?;

    let qh = queue.handle();

    let manager: HyprlandGlobalShortcutsManagerV1 = globals.bind(&qh, 1..=1, ()).map_err(|e| {
        format!(
            "Hyprland global-shortcuts manager is not advertised ({e}). \
                 --background only works on Hyprland."
        )
    })?;

    let _shortcut: HyprlandGlobalShortcutV1 = manager.register_shortcut(
        SHORTCUT_ID.to_string(),
        APP_ID.to_string(),
        SHORTCUT_DESCRIPTION.to_string(),
        SHORTCUT_TRIGGER_DESCRIPTION.to_string(),
        &qh,
        (),
    );

    let mut state = ListenerState { on_pressed };

    loop {
        match queue.blocking_dispatch(&mut state) {
            Ok(_) => {}
            Err(e) => {
                return Err(format!("Wayland event dispatch failed: {e}"));
            }
        }
    }
}
