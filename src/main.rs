use gtk::gdk_pixbuf::PixbufLoader;
use gtk::prelude::*;
use gtk::Adjustment;
use gtk::{
    Application, ApplicationWindow, Box, Button, Image, Orientation, ScrolledWindow, TextView,
};
use std::io::Write;
use std::process::Command;

fn run_command(args: &[&str]) -> Option<Vec<u8>> {
    let output = Command::new(args[0]).args(&args[1..]).output().ok()?;
    if output.status.success() {
        Some(output.stdout)
    } else {
        None
    }
}

fn has_mime_type(types: &str, mime: &str) -> bool {
    types.lines().any(|line| line == mime)
}

fn show_clipboard_text(text: &str) {
    let app = Application::new(None, Default::default());
    let text_owned = text.to_string();
    app.connect_activate(move |app| {
        let window = ApplicationWindow::new(app);
        window.set_title("Clipboard Text");
        window.set_default_size(400, 300);

        window.set_type_hint(gtk::gdk::WindowTypeHint::Dialog);
        window.set_keep_above(true);
        window.set_modal(true);

        // Add ESC key binding to close window
        window.add_events(gtk::gdk::EventMask::KEY_PRESS_MASK);
        window.connect_key_press_event(move |window, event| {
            if event.keyval() == gtk::gdk::keys::constants::Escape {
                window.close();
            }
            false.into()
        });

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
            buffer.set_text(&text_owned);
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
                    let mut child = Command::new("wl-copy")
                        .stdin(std::process::Stdio::piped())
                        .spawn()
                        .ok();
                    if let Some(ref mut c) = child {
                        if let Some(stdin) = c.stdin.as_mut() {
                            let _ = stdin.write_all(text_to_copy.as_bytes());
                        }
                        let _ = c.wait();
                    }
                }
            }
        });
        vbox.pack_start(&copy_btn, false, false, 0);

        window.add(&vbox);
        window.show_all();
        window.present();
    });
    app.run();
}

fn show_clipboard_image(img_data: &[u8], mime_type: &str) {
    let app = Application::new(None, Default::default());
    let img_data_owned = img_data.to_vec();
    let mime_type_owned = mime_type.to_string();

    app.connect_activate(move |app| {
        let window = ApplicationWindow::new(app);
        window.set_title("Clipboard Image");
        window.set_resizable(true);
        window.set_decorated(false); // Remove titlebar

        window.set_type_hint(gtk::gdk::WindowTypeHint::Dialog);
        window.set_keep_above(true);
        window.set_modal(true);

        // Add drag functionality and motion tracking
        window.add_events(
            gtk::gdk::EventMask::BUTTON_PRESS_MASK 
            | gtk::gdk::EventMask::KEY_PRESS_MASK 
            | gtk::gdk::EventMask::POINTER_MOTION_MASK
            | gtk::gdk::EventMask::ENTER_NOTIFY_MASK
            | gtk::gdk::EventMask::LEAVE_NOTIFY_MASK
        );
        
        window.connect_button_press_event(move |window, event| {
            if event.button() == 1 { // Left mouse button
                window.begin_move_drag(1, event.root().0 as i32, event.root().1 as i32, event.time());
            }
            false.into()
        });

        // Add ESC key binding to close window
        window.connect_key_press_event(move |window, event| {
            if event.keyval() == gtk::gdk::keys::constants::Escape {
                window.close();
            }
            false.into()
        });

        let loader = PixbufLoader::new();
        if loader.write(&img_data_owned).is_err() {
            eprintln!("Failed to load image from clipboard data.");
            return;
        }
        if loader.close().is_err() {
            eprintln!("Failed to finalize image loading.");
            return;
        }
        if let Some(orig_pixbuf) = loader.pixbuf() {
            // Use Overlay to place button on top of image
            let overlay = gtk::Overlay::new();

            // Image widget
            let image = Image::new();
            image.set_hexpand(true);
            image.set_vexpand(true);

            // Scrolled window for panning
            let scrolled = ScrolledWindow::new(None::<&Adjustment>, None::<&Adjustment>);
            scrolled.set_policy(gtk::PolicyType::Automatic, gtk::PolicyType::Automatic);
            scrolled.add(&image);
            
            // Add scrolled window as the main child of overlay
            overlay.add(&scrolled);

            // Copy button as overlay
            let copy_btn = Button::with_label("Copy to Clipboard");
            copy_btn.set_margin_top(10);
            copy_btn.set_margin_end(10);
            copy_btn.set_halign(gtk::Align::End);
            copy_btn.set_valign(gtk::Align::Start);

            let img_data_clone = img_data_owned.clone();
            let mime_type_clone = mime_type_owned.clone();
            copy_btn.connect_clicked(move |_| {
                let mut child = Command::new("wl-copy")
                    .arg("--type")
                    .arg(&mime_type_clone)
                    .stdin(std::process::Stdio::piped())
                    .spawn()
                    .ok();
                if let Some(ref mut c) = child {
                    if let Some(stdin) = c.stdin.as_mut() {
                        let _ = stdin.write_all(&img_data_clone);
                    }
                    let _ = c.wait();
                }
            });
            
            // Add button as overlay
            overlay.add_overlay(&copy_btn);

            // Button fade functionality
            use std::rc::Rc;
            use std::cell::RefCell;
            
            let fade_timeout = Rc::new(RefCell::new(None::<gtk::glib::SourceId>));
            let timer_active = Rc::new(RefCell::new(false));
            let copy_btn_clone_for_fade = copy_btn.clone();
            let fade_timeout_clone = fade_timeout.clone();
            let timer_active_clone = timer_active.clone();
            
            // Function to start fade timer
            let start_fade_timer = move || {
                // Cancel existing timer if active
                if *timer_active_clone.borrow() {
                    if let Some(timeout_id) = fade_timeout_clone.borrow_mut().take() {
                        timeout_id.remove();
                    }
                    *timer_active_clone.borrow_mut() = false;
                }
                
                let copy_btn_for_timeout = copy_btn_clone_for_fade.clone();
                let timer_active_for_timeout = timer_active_clone.clone();
                let timeout_id = gtk::glib::timeout_add_seconds_local(3, move || {
                    copy_btn_for_timeout.set_opacity(0.0);
                    *timer_active_for_timeout.borrow_mut() = false;
                    gtk::glib::ControlFlow::Break
                });
                *fade_timeout_clone.borrow_mut() = Some(timeout_id);
                *timer_active_clone.borrow_mut() = true;
            };
            
            // Function to show button
            let show_button = {
                let copy_btn_show = copy_btn.clone();
                let start_timer = start_fade_timer.clone();
                move || {
                    copy_btn_show.set_opacity(1.0);
                    start_timer();
                }
            };

            // Motion event handler
            let show_button_motion = show_button.clone();
            window.connect_motion_notify_event(move |_window, _event| {
                show_button_motion();
                false.into()
            });

            // Add motion tracking to overlay as well
            let show_button_overlay_motion = show_button.clone();
            overlay.connect_enter_notify_event(move |_overlay, _event| {
                show_button_overlay_motion();
                false.into()
            });

            // Enter/Leave notify handlers
            let show_button_enter = show_button.clone();
            window.connect_enter_notify_event(move |_window, _event| {
                show_button_enter();
                false.into()
            });

            let show_button_focus = show_button.clone();
            window.connect_focus_in_event(move |_window, _event| {
                show_button_focus();
                false.into()
            });

            // Start initial fade timer
            start_fade_timer();

            window.add(&overlay);

            // Set window size to original image size (no extra space for button now)
            let orig_width = orig_pixbuf.width();
            let orig_height = orig_pixbuf.height();
            window.set_default_size(orig_width, orig_height);
            window.set_size_request(100, 100); // allow smaller resizing

            // Clone orig_pixbuf for use in the closure
            let orig_pixbuf_for_closure = orig_pixbuf.clone();
            let image_clone = image.clone();
            window.connect_size_allocate(move |_, alloc| {
                let w = alloc.width();
                let h = alloc.height();
                if w > 0 && h > 0 {
                    let scale = f64::min(
                        w as f64 / orig_pixbuf_for_closure.width() as f64,
                        h as f64 / orig_pixbuf_for_closure.height() as f64,
                    );
                    let new_w = (orig_pixbuf_for_closure.width() as f64 * scale).round() as i32;
                    let new_h = (orig_pixbuf_for_closure.height() as f64 * scale).round() as i32;
                    if let Some(scaled) = orig_pixbuf_for_closure.scale_simple(
                        new_w,
                        new_h,
                        gtk::gdk_pixbuf::InterpType::Bilinear,
                    ) {
                        image_clone.set_from_pixbuf(Some(&scaled));
                    }
                }
            });

            // Set initial image at original size
            if let Some(scaled) = orig_pixbuf.scale_simple(
                orig_width,
                orig_height,
                gtk::gdk_pixbuf::InterpType::Bilinear,
            ) {
                image.set_from_pixbuf(Some(&scaled));
            }

            window.show_all();
            window.present();
        } else {
            eprintln!("Failed to decode image data.");
            return;
        }
    });
    app.run();
}

fn main() {
    // Force GTK to use X11 backend even on Wayland
    unsafe {
        std::env::set_var("GDK_BACKEND", "x11");
    }

    gtk::init().expect("Failed to initialize GTK");

    const ICON: &[u8] = include_bytes!("icon.ico");
    // Load icon from ICON bytes and set as default application icon
    let loader = gtk::gdk_pixbuf::PixbufLoader::new();
    if loader.write(ICON).is_ok() && loader.close().is_ok() {
        if let Some(icon_pixbuf) = loader.pixbuf() {
            gtk::Window::set_default_icon(&icon_pixbuf);
        }
    }

    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 {
        eprintln!(
            "Usage: {}\nJust run with no arguments to show image or text from clipboard.",
            args[0]
        );
        std::process::exit(1);
    }
    let types_raw = run_command(&["wl-paste", "--list-types"]).unwrap_or_default();
    if types_raw.is_empty() {
        eprintln!("Could not retrieve clipboard types or clipboard is empty.");
        std::process::exit(1);
    }
    let types = String::from_utf8_lossy(&types_raw);
    let is_image = has_mime_type(&types, "image/png")
        || has_mime_type(&types, "image/jpeg")
        || has_mime_type(&types, "image/gif");
    let is_text = has_mime_type(&types, "text/plain")
        || has_mime_type(&types, "text/plain;charset=utf-8")
        || has_mime_type(&types, "UTF8_STRING")
        || has_mime_type(&types, "TEXT")
        || has_mime_type(&types, "STRING");
    let is_file = has_mime_type(&types, "text/uri-list");
    if is_file {
        eprintln!("Clipboard contains a file list, ignoring.");
        return;
    } else if is_image {
        println!("Detected image in clipboard.");
        let mut img_data: Vec<u8>;
        let mut mime_type: &str; // Made mime_type mutable

        if has_mime_type(&types, "image/png") {
            img_data = run_command(&["wl-paste", "--type", "image/png"]).unwrap_or_default();
            mime_type = "image/png";
        } else if has_mime_type(&types, "image/jpeg") {
            img_data = run_command(&["wl-paste", "--type", "image/jpeg"]).unwrap_or_default();
            mime_type = "image/jpeg";
        } else if has_mime_type(&types, "image/gif") {
            img_data = run_command(&["wl-paste", "--type", "image/gif"]).unwrap_or_default();
            mime_type = "image/gif";
        } else {
            // Fallback if specific type check failed but is_image was true (should not happen with current logic)
            img_data = run_command(&["wl-paste", "--type", "image/png"]).unwrap_or_default();
            mime_type = "image/png";
            if img_data.is_empty() {
                img_data = run_command(&["wl-paste", "--type", "image/jpeg"]).unwrap_or_default();
                mime_type = "image/jpeg";
                if img_data.is_empty() {
                    img_data =
                        run_command(&["wl-paste", "--type", "image/gif"]).unwrap_or_default();
                    mime_type = "image/gif";
                }
            }
        }

        if img_data.is_empty() {
            eprintln!("No supported image found in clipboard or wl-paste failed.");
            return;
        }
        show_clipboard_image(&img_data, mime_type);
    } else if is_text {
        println!("Detected text in clipboard.");
        let text = run_command(&["wl-paste", "--no-newline"]).unwrap_or_default();
        if text.is_empty() {
            eprintln!("No text found in clipboard or wl-paste failed.");
            return;
        }
        show_clipboard_text(&String::from_utf8_lossy(&text));
    } else {
        eprintln!("Clipboard does not contain supported image or text types.");
        std::process::exit(1);
    }
}
