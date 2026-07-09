use gtk::gdk_pixbuf::PixbufLoader;
use gtk::prelude::*;
use gtk::Adjustment;
use gtk::{
    Application, ApplicationWindow, Box, Button, Image, Orientation, ScrolledWindow, TextView,
};
use waypin_lib::{
    copy_image_to_clipboard, copy_text_to_clipboard, detect_clipboard_content_type,
    get_image_format_from_types, run_command, ClipboardContentType,
};

#[cfg(feature = "global-shortcuts")]
mod shortcut;

/// Promote a preview window to the wlr-layer-shell `top` layer so it stays
/// always-on-top on wlroots / KDE / COSMIC / Weston / niri / etc.
///
/// Silently no-ops on compositors that don't implement `zwlr_layer_shell_v1`
/// (GNOME Mutter, Cinnamon Muffin, Cage) and under X11. In those cases the
/// caller's existing `set_keep_above(true)` already handles the X11/EWMH path,
/// and on the unsupported Wayland compositors the window just appears as a
/// normal foreground window — the documented limitation.
///
/// Must be called *before* `show_all()` so `init_layer_shell()` runs prior to
/// the window being realized (gtk-layer-shell requirement).
#[cfg(feature = "pin-on-top")]
fn try_promote_to_layer_top(window: &ApplicationWindow, w: i32, h: i32) {
    use gtk_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};

    if !gtk_layer_shell::is_supported() {
        return;
    }

    window.init_layer_shell();
    window.set_layer(Layer::Top);
    // No edges anchored → the compositor places the surface centered on the
    // output (per the wlr-layer-shell spec), giving us a floating preview
    // rather than a stretched panel.
    window.set_anchor(Edge::Top, false);
    window.set_anchor(Edge::Bottom, false);
    window.set_anchor(Edge::Left, false);
    window.set_anchor(Edge::Right, false);
    // OnDemand = click-to-focus text entry; no aggressive keyboard grab.
    window.set_keyboard_mode(KeyboardMode::OnDemand);
    // Hint the compositor with the desired size; the actual allocation arrives
    // via the regular `configure` event. We still call GtkWindow's sizing API
    // so the surface has a sensible default before the first configure.
    let _ = (w, h);
}

#[cfg(not(feature = "pin-on-top"))]
fn try_promote_to_layer_top(_: &ApplicationWindow, _: i32, _: i32) { /* no-op */ }

#[derive(Clone)]
enum ClipboardContent {
    Text(String),
    Image(Vec<u8>, &'static str),
}

/// Reads supported clipboard content without requiring GTK initialization.
fn read_clipboard() -> Option<ClipboardContent> {
    let types_raw = run_command(&["wl-paste", "--list-types"]).unwrap_or_default();
    if types_raw.is_empty() {
        eprintln!("Could not retrieve clipboard types or clipboard is empty.");
        return None;
    }
    let types = String::from_utf8_lossy(&types_raw);
    match detect_clipboard_content_type(&types) {
        ClipboardContentType::File => {
            eprintln!("Clipboard contains a file list, ignoring.");
            None
        }
        ClipboardContentType::Unsupported => {
            eprintln!("Clipboard does not contain supported image or text types.");
            None
        }
        ClipboardContentType::Image => {
            let mime_type = get_image_format_from_types(&types)
                .expect("image clipboard content must have a supported MIME type");
            let image_data = run_command(&["wl-paste", "--type", mime_type]).unwrap_or_default();
            if image_data.is_empty() {
                eprintln!("No supported image found in clipboard or wl-paste failed.");
                None
            } else {
                Some(ClipboardContent::Image(image_data, mime_type))
            }
        }
        ClipboardContentType::Text => {
            let text = run_command(&["wl-paste", "--no-newline"]).unwrap_or_default();
            if text.is_empty() {
                eprintln!("No text found in clipboard or wl-paste failed.");
                None
            } else {
                Some(ClipboardContent::Text(
                    String::from_utf8_lossy(&text).into_owned(),
                ))
            }
        }
    }
}

fn show_clipboard_content(app: &Application, content: &ClipboardContent) {
    match content {
        ClipboardContent::Text(text) => {
            build_text_window(app, text);
        }
        ClipboardContent::Image(data, mime_type) => {
            build_image_window(app, data, mime_type);
        }
    };
}

/// Reads the clipboard and opens a fresh preview window on `app`.
///
/// Called by the background shortcut listener. Each call builds an independent
/// `ApplicationWindow`, so the daemon can show multiple simultaneous previews.
fn show_clipboard(app: &Application) {
    if let Some(content) = read_clipboard() {
        show_clipboard_content(app, &content);
    }
}

fn build_text_window(app: &Application, text: &str) -> ApplicationWindow {
    let window = ApplicationWindow::new(app);
    window.set_title("Clipboard Text");
    window.set_default_size(400, 300);
    window.set_type_hint(gtk::gdk::WindowTypeHint::Dialog);
    window.set_keep_above(true);
    window.set_modal(true);

    let vbox = Box::new(Orientation::Vertical, 10);
    vbox.set_margin_top(16);
    vbox.set_margin_bottom(16);
    vbox.set_margin_start(16);
    vbox.set_margin_end(16);

    let scrolled = ScrolledWindow::new(None::<&gtk::Adjustment>, None::<&gtk::Adjustment>);
    scrolled.set_min_content_height(180);
    scrolled.set_min_content_width(350);
    let text_view = TextView::new();
    text_view.set_wrap_mode(gtk::WrapMode::Word);
    if let Some(buffer) = text_view.buffer() {
        buffer.set_text(text);
    }
    scrolled.add(&text_view);
    vbox.pack_start(&scrolled, true, true, 0);

    let copy_btn = Button::with_label("Copy to Clipboard");
    copy_btn.set_margin_top(10);
    copy_btn.set_halign(gtk::Align::End);
    let text_view_clone = text_view.clone();
    copy_btn.connect_clicked(move |_| {
        if let Some(buffer) = text_view_clone.buffer() {
            let start = buffer.start_iter();
            let end = buffer.end_iter();
            if let Some(text_to_copy) = buffer.text(&start, &end, false) {
                if let Err(error) = copy_text_to_clipboard(&text_to_copy) {
                    eprintln!("Failed to copy text to clipboard: {error}");
                }
            }
        }
    });
    vbox.pack_start(&copy_btn, false, false, 0);

    window.add(&vbox);
    try_promote_to_layer_top(&window, 400, 300);
    window.show_all();
    window.present();
    window
}

fn build_image_window(app: &Application, img_data: &[u8], mime_type: &str) {
    let window = ApplicationWindow::new(app);
    window.set_title("Clipboard Image");
    window.set_resizable(true);
    window.set_type_hint(gtk::gdk::WindowTypeHint::Dialog);
    window.set_keep_above(true);
    window.set_modal(true);

    let loader = PixbufLoader::new();
    if loader.write(img_data).is_err() {
        eprintln!("Failed to load image from clipboard data.");
        return;
    }
    if loader.close().is_err() {
        eprintln!("Failed to finalize image loading.");
        return;
    }
    if let Some(orig_pixbuf) = loader.pixbuf() {
        let vbox = Box::new(Orientation::Vertical, 0);

        let image = Image::new();
        image.set_hexpand(true);
        image.set_vexpand(true);

        let scrolled = ScrolledWindow::new(None::<&Adjustment>, None::<&Adjustment>);
        scrolled.set_policy(gtk::PolicyType::Automatic, gtk::PolicyType::Automatic);
        scrolled.set_margin_top(0);
        scrolled.set_margin_bottom(0);
        scrolled.set_margin_start(0);
        scrolled.set_margin_end(0);
        scrolled.add(&image);
        vbox.pack_start(&scrolled, true, true, 0);

        let copy_btn = Button::with_label("Copy to Clipboard");
        copy_btn.set_margin_top(10);
        copy_btn.set_margin_bottom(10);
        copy_btn.set_margin_start(10);
        copy_btn.set_margin_end(10);
        copy_btn.set_halign(gtk::Align::End);

        let img_data_clone = img_data.to_vec();
        let mime_type_clone = mime_type.to_string();
        copy_btn.connect_clicked(move |_| {
            if let Err(error) = copy_image_to_clipboard(&mime_type_clone, &img_data_clone) {
                eprintln!("Failed to copy image to clipboard: {error}");
            }
        });
        vbox.pack_start(&copy_btn, false, false, 0);

        window.add(&vbox);

        let orig_width = orig_pixbuf.width();
        let orig_height = orig_pixbuf.height();
        let button_height = 55;
        let max_default_dim = 1600;
        let default_w = orig_width.min(max_default_dim);
        let default_h = (orig_height + button_height).min(max_default_dim);
        scrolled.set_min_content_width(1);
        scrolled.set_min_content_height(1);
        window.set_default_size(default_w, default_h);
        window.set_size_request(100, 100);

        let orig_pixbuf_for_closure = orig_pixbuf.clone();
        let image_clone = image.clone();
        scrolled.connect_size_allocate(move |_, alloc| {
            let w = alloc.width();
            let h = alloc.height();
            if w > 0 && h > 0 {
                let (new_w, new_h) = waypin_lib::compute_scaled_dimensions(
                    orig_pixbuf_for_closure.width(),
                    orig_pixbuf_for_closure.height(),
                    w,
                    h,
                    true,
                );
                if new_w > 0 && new_h > 0 {
                    if let Some(scaled) = orig_pixbuf_for_closure.scale_simple(
                        new_w,
                        new_h,
                        gtk::gdk_pixbuf::InterpType::Bilinear,
                    ) {
                        image_clone.set_from_pixbuf(Some(&scaled));
                    }
                }
            }
        });

        image.set_from_pixbuf(Some(&orig_pixbuf));

        try_promote_to_layer_top(&window, default_w, default_h);
        window.show_all();
        window.present();
    } else {
        eprintln!("Failed to decode image data.");
    }
}

/// One-shot mode: stop the application once the last preview window closes.
fn connect_quit_on_last_window_close(app: &Application) {
    let app_clone = app.clone();
    app.connect_shutdown(move |_| {
        if app_clone.windows().is_empty() {
            app_clone.quit();
        }
    });
}

fn run_one_shot() -> i32 {
    let Some(content) = read_clipboard() else {
        return 1;
    };

    gtk::init().expect("Failed to initialize GTK");
    load_icon();
    let app = Application::new(None, Default::default());
    connect_quit_on_last_window_close(&app);
    let app_clone = app.clone();
    app.connect_activate(move |_| {
        show_clipboard_content(&app_clone, &content);
    });
    // Apply no-op run: launches the app and immediately fires `activate`, which
    // builds the initial preview window.
    app.run();
    0
}

fn load_icon() {
    const ICON: &[u8] = include_bytes!("icon.ico");
    let loader = gtk::gdk_pixbuf::PixbufLoader::new();
    if loader.write(ICON).is_ok() && loader.close().is_ok() {
        if let Some(icon_pixbuf) = loader.pixbuf() {
            gtk::Window::set_default_icon(&icon_pixbuf);
        }
    }
}

#[cfg(feature = "global-shortcuts")]
fn run_background() -> i32 {
    use std::cell::Cell;
    use std::cell::RefCell;
    use std::rc::Rc;
    use std::sync::{Arc, Mutex};
    use std::thread;

    enum BackgroundEvent {
        ShortcutPressed,
        ListenerFailed(String),
    }

    // Channel: listener thread -> GLib main thread.
    // Uses the deprecated MainContext::channel API deliberately (still supported
    // in glib 0.18); migration to async-channel + spawn_future_local would
    // balloon the dependency tree for no functional gain here.
    #[allow(deprecated)]
    let (sender, receiver) =
        gtk::glib::MainContext::channel::<BackgroundEvent>(gtk::glib::Priority::default());

    // Listener thread owns its Wayland connection; blocking_dispatch parks it
    // until a shortcut event arrives. send() wakes the GLib main loop once.
    let sender_clone = Arc::new(Mutex::new(sender));
    let sender_for_thread = sender_clone.clone();
    let listener_handle = thread::Builder::new()
        .name("waypin-wl-listener".into())
        .stack_size(64 * 1024)
        .spawn(move || {
            let sender_pressed = sender_for_thread.clone();
            let sender_error = sender_for_thread.clone();
            if let Err(e) = shortcut::run_listener(move || {
                let _ = sender_pressed
                    .lock()
                    .map(|s| s.send(BackgroundEvent::ShortcutPressed));
            }) {
                let _ = sender_error
                    .lock()
                    .map(|s| s.send(BackgroundEvent::ListenerFailed(e)));
            }
        })
        .expect("spawn listener thread");

    // Hold the handle so the thread's lifetime equals the daemon's.
    let _handle = listener_handle;

    // Lazy application: created on first shortcut press to keep heap tiny.
    let app: Rc<RefCell<Option<Application>>> = Rc::new(RefCell::new(None));
    let app_clone = app.clone();
    let exit_code = Rc::new(Cell::new(0));
    let exit_code_for_receiver = exit_code.clone();

    // Run the GLib main loop; the receiver's callback fires on every wake-up.
    let main_ctx = gtk::glib::MainContext::default();
    let main_loop = gtk::glib::MainLoop::new(Some(&main_ctx), false);
    let main_loop_for_receiver = main_loop.clone();
    let _receiver = receiver.attach(Some(&main_ctx), move |event| {
        if let BackgroundEvent::ListenerFailed(error) = event {
            eprintln!("waypin: background shortcut listener failed: {error}");
            exit_code_for_receiver.set(1);
            main_loop_for_receiver.quit();
            return gtk::glib::ControlFlow::Break;
        }

        let mut app_ref = app_clone.borrow_mut();
        if app_ref.is_none() {
            // Lazy GTK init on first shortcut press.
            gtk::init().expect("Failed to initialize GTK");
            load_icon();
            *app_ref = Some(Application::new(None, Default::default()));
            // Closing the last preview window does NOT quit the daemon:
            // the GLib MainLoop keeps spinning and the wayland listener
            // thread keeps the process alive until the compositor dies
            // or systemd stops it.
        }
        if let Some(app) = app_ref.as_ref() {
            show_clipboard(app);
        }
        drop(app_ref);
        gtk::glib::ControlFlow::Continue
    });

    main_loop.run();
    exit_code.get()
}

fn print_usage(prog: &str) {
    eprintln!(
        "Usage: {prog} [--background]\n\
         \n\
         No arguments: read the clipboard once, show one preview, exit on close.\n\
         --background:   register the Hyprland global shortcut `waypin:toggle`\n\
         and stay running, opening a new preview on every shortcut press.\n\
         \n\
         (The global-shortcut background mode requires the `global-shortcuts`\n\
         build feature and a running Hyprland session.)"
    );
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    // --background mode bypasses the one-shot path entirely.
    if args.len() == 2 && args[1] == "--background" {
        #[cfg(feature = "global-shortcuts")]
        {
            std::process::exit(run_background());
        }
        #[cfg(not(feature = "global-shortcuts"))]
        {
            eprintln!(
                "waypin was built without the `global-shortcuts` feature; \
                 --background is unavailable."
            );
            std::process::exit(2);
        }
    }

    if args.len() > 1 {
        print_usage(&args[0]);
        std::process::exit(1);
    }

    // No args: legacy one-shot. No longer force GdkBackend=x11; rely on GTK's
    // autodetect so the daemon and shortcut listener can use native Wayland
    // while xvfb-only CI environments still fall back to X11 transparently.
    std::process::exit(run_one_shot());
}
