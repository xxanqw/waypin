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
                    if let Err(error) = copy_text_to_clipboard(&text_to_copy) {
                        eprintln!("Failed to copy text to clipboard: {error}");
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

        window.set_type_hint(gtk::gdk::WindowTypeHint::Dialog);
        window.set_keep_above(true);
        window.set_modal(true);

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
            let vbox = Box::new(Orientation::Vertical, 0);

            // Image widget
            let image = Image::new();
            image.set_hexpand(true);
            image.set_vexpand(true);

            // Scrolled window for panning
            let scrolled = ScrolledWindow::new(None::<&Adjustment>, None::<&Adjustment>);
            scrolled.set_policy(gtk::PolicyType::Automatic, gtk::PolicyType::Automatic);
            scrolled.set_margin_top(0);
            scrolled.set_margin_bottom(0);
            scrolled.set_margin_start(0);
            scrolled.set_margin_end(0);
            scrolled.add(&image);
            vbox.pack_start(&scrolled, true, true, 0);

            // Copy button
            let copy_btn = Button::with_label("Copy to Clipboard");
            copy_btn.set_margin_top(10);
            copy_btn.set_margin_bottom(10);
            copy_btn.set_margin_start(10);
            copy_btn.set_margin_end(10);
            copy_btn.set_halign(gtk::Align::End);

            let img_data_clone = img_data_owned.clone();
            let mime_type_clone = mime_type_owned.clone();
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
            window.set_size_request(100, 100); // allow smaller resizing

            // Clone orig_pixbuf for use in the closure and scale based on scrolled viewport.
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

            // Set initial image at original size (pixbuf already has native size)
            image.set_from_pixbuf(Some(&orig_pixbuf));

            window.show_all();
            window.present();
        } else {
            eprintln!("Failed to decode image data.");
        }
    });
    app.run();
}

fn main() {
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
    let clipboard_content_type = detect_clipboard_content_type(&types);

    if clipboard_content_type == ClipboardContentType::File {
        eprintln!("Clipboard contains a file list, ignoring.");
        return;
    }
    if clipboard_content_type == ClipboardContentType::Unsupported {
        eprintln!("Clipboard does not contain supported image or text types.");
        std::process::exit(1);
    }

    if let Err(error) = gtk::init() {
        eprintln!("Failed to initialize GTK: {error}");
        std::process::exit(1);
    }

    const ICON: &[u8] = include_bytes!("icon.ico");
    let loader = gtk::gdk_pixbuf::PixbufLoader::new();
    if loader.write(ICON).is_ok() && loader.close().is_ok() {
        if let Some(icon_pixbuf) = loader.pixbuf() {
            gtk::Window::set_default_icon(&icon_pixbuf);
        }
    }

    match clipboard_content_type {
        ClipboardContentType::Image => {
            let mime_type = get_image_format_from_types(&types)
                .expect("image clipboard content must have a supported MIME type");
            println!("Detected image in clipboard.");
            let image_data = run_command(&["wl-paste", "--type", mime_type]).unwrap_or_default();
            if image_data.is_empty() {
                eprintln!("No supported image found in clipboard or wl-paste failed.");
                return;
            }
            show_clipboard_image(&image_data, mime_type);
        }
        ClipboardContentType::Text => {
            println!("Detected text in clipboard.");
            let text = run_command(&["wl-paste", "--no-newline"]).unwrap_or_default();
            if text.is_empty() {
                eprintln!("No text found in clipboard or wl-paste failed.");
                return;
            }
            show_clipboard_text(&String::from_utf8_lossy(&text));
        }
        ClipboardContentType::File | ClipboardContentType::Unsupported => unreachable!(),
    }
}
