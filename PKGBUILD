# Maintainer: Ivan Potiienko <contact@xxanqw.pp.ua>
pkgname=waypin
pkgver=0.1.8
pkgrel=1
pkgdesc="A clipboard viewer for Wayland/X11 with GTK3, written in Rust"
arch=('x86_64')
url="https://github.com/xxanqw/waypin"
license=('GPL3')
depends=('gtk3' 'gdk-pixbuf2' 'wl-clipboard')
makedepends=('cargo' 'git')
# Optional: only needed for the background-daemon global-shortcut mode.
# The systemd unit is installed unconditionally but self-skips on non-Hyprland
# sessions via its Condition* directives — no install-hook probing required.
optdepends=(
  'hyprland: for the global-shortcut background daemon'
  'systemd: for the background daemon service'
  'gtk-layer-shell: for pin-on-top preview windows (wlroots/KWin/COSMIC/etc.)'
)
source=("git+https://github.com/xxanqw/waypin.git#branch=restoring")
sha256sums=('SKIP')

build() {
  cd "$pkgname"
  cargo build --release --locked --target-dir "$srcdir/target" --features global-shortcuts
}

package() {
  cd "$pkgname"
  install -Dm755 "$srcdir/target/release/waypin" "$pkgdir/usr/bin/waypin"
  install -Dm644 systemd/waypin.service "$pkgdir/usr/lib/systemd/user/waypin.service"
}