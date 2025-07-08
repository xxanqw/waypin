<div align="center">

# 📌 Waypin

**A sleek clipboard viewer for Wayland/X11 with GTK3**

[![License: GPL v3](https://img.shields.io/badge/License-GPLv3-blue.svg)](https://www.gnu.org/licenses/gpl-3.0)
[![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)](https://www.rust-lang.org)
[![GTK3](https://img.shields.io/badge/GTK-3.0-green.svg)](https://gtk.org)
[![Wayland](https://img.shields.io/badge/Wayland-supported-purple.svg)](https://wayland.freedesktop.org)

*Instantly preview and manage your clipboard content with a beautiful, responsive interface*

[Features](#-features) • [Installation](#-installation) • [Usage](#-usage) • [Building](#-building-from-source) • [Contributing](#-contributing)

</div>

---

## 🌟 Features

### 📝 **Text Clipboard Support**
- **Rich Text Viewing**: Display clipboard text content in a scrollable, word-wrapped interface
- **Live Editing**: Modify clipboard text directly in the viewer
- **One-Click Copy**: Instantly copy modified text back to clipboard

### 🖼️ **Image Clipboard Support**
- **Multi-Format Support**: PNG, JPEG, and GIF image formats
- **Smart Scaling**: Automatic image scaling while maintaining aspect ratio
- **Original Size Display**: View images at their native resolution

### 🎨 **Modern Interface**
- **GTK3 Native**: Clean, system-integrated appearance
- **Modal Dialog**: Always-on-top, focused viewing experience
- **Responsive Design**: Adapts to different content sizes and screen resolutions
- **Custom Icon**: Distinctive application branding

### 🔧 **Cross-Platform Compatibility**
- **Wayland Native**: Full support for modern Wayland compositors
- **X11 Fallback**: Seamless operation on traditional X11 systems
- **Smart Detection**: Automatic protocol selection

---

## 📦 Installation

### Arch Linux (AUR)
```bash
# Using your favorite AUR helper
yay -S waypin
# or
paru -S waypin
```

### Manual Installation
```bash
# Clone the repository
git clone https://github.com/xxanqw/waypin.git
cd waypin

# Build and install
cargo build --release
sudo cp target/release/waypin /usr/local/bin/
```

### Dependencies
- `gtk3` - GUI framework
- `gdk-pixbuf2` - Image loading and processing
- `wl-clipboard` - Wayland clipboard utilities

---

## 🚀 Usage

### Basic Usage
Simply run Waypin to view your current clipboard content:

```bash
waypin
```

### What Happens
- **Text Content**: Opens an editable text viewer with copy functionality
- **Image Content**: Displays images with scaling and navigation controls
- **File Lists**: Safely ignored to prevent accidental operations
- **Empty Clipboard**: Provides helpful error messaging

### Keyboard Shortcuts
- **Ctrl+C**: Copy modified text (in text viewer)
- **Scroll**: Navigate through large images
- **Escape**: Close the viewer window

---

## 🛠️ Building from Source

### Prerequisites
```bash
# Arch Linux
sudo pacman -S rust gtk3 gdk-pixbuf2 wl-clipboard git

# Ubuntu/Debian
sudo apt install cargo libgtk-3-dev libgdk-pixbuf2.0-dev wl-clipboard git

# Fedora
sudo dnf install cargo gtk3-devel gdk-pixbuf2-devel wl-clipboard git
```

### Build Steps
```bash
git clone https://github.com/xxanqw/waypin.git
cd waypin
cargo build --release
```

### Development
```bash
# Run in debug mode
cargo run

# Run tests
cargo test
```

---

## 🎯 Use Cases

- **Developer Workflow**: Quickly preview copied code snippets or error messages
- **Content Creation**: Review copied text before pasting into documents
- **Image Editing**: Preview copied images before processing
- **System Administration**: Examine clipboard content for security purposes
- **General Productivity**: Universal clipboard inspection tool

---

## 🔮 Roadmap

- [ ] **Document Viewer Support**: View PDF, DOCX, ODT, and other document formats
- [x] **GUI Redesign**: Modern interface overhaul with improved user experience
- [ ] **Hotkey Support**: Global shortcuts for instant access

---

## 🤝 Contributing

We welcome contributions! Here's how you can help:

1. **Fork** the repository
2. **Create** a feature branch (`git checkout -b feature/amazing-feature`)
3. **Commit** your changes (`git commit -m 'Add amazing feature'`)
4. **Push** to the branch (`git push origin feature/amazing-feature`)
5. **Open** a Pull Request

### Development Guidelines
- Follow Rust naming conventions
- Add tests for new features
- Update documentation for API changes
- Ensure GTK3 compatibility

---

## 📄 License

This project is licensed under the **GNU General Public License v3.0** - see the [LICENSE](LICENSE) file for details.

---

## 👨‍💻 Author

**Ivan Potiienko** - *Initial work* - [@xxanqw](https://github.com/xxanqw)

---

## 🙏 Acknowledgments

- **GTK Team** - For the excellent GUI framework
- **Wayland Developers** - For the modern display protocol
- **Rust Community** - For the amazing ecosystem and tools

---

<div align="center">

**[⬆️ Back to Top](#-waypin)**

Made with ❤️ and 🦀 Rust

</div>
