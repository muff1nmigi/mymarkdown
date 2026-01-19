# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**MyMarkdown** - Native GNOME markdown editor dengan live preview.

- **Stack**: Rust + GTK4 + libadwaita + GtkSourceView5 + WebKitGTK6
- **Target**: GNOME 49+ / CachyOS
- **Binary**: `mymd`

## Build Commands

```bash
cargo build              # Dev build
cargo build --release    # Optimized + stripped
cargo run                # Run directly

# Install to system
sudo cp target/release/mymd /usr/local/bin/
sudo cp data/org.gnome.MyMarkdown.desktop /usr/share/applications/
```

## Usage

```bash
mymd                  # Open with new file
mymd writeup          # Create/open writeup.md
mymd writeup.md       # Create/open writeup.md
```

## Project Structure

```
src/
├── main.rs       # Entry point + CLI args
├── app.rs        # AdwApplication subclass
└── window.rs     # Main window + all UI logic

data/
└── org.gnome.MyMarkdown.desktop
```

## Architecture

### UI Layout
- `AdwApplicationWindow` dengan `AdwHeaderBar`
- `gtk::Paned` horizontal split: Editor (left) | Preview (right)
- Toggle button untuk show/hide preview

### Editor (GtkSourceView5)
- Markdown syntax highlighting
- JetBrains Mono font
- Line numbers, highlight current line
- Auto-indent, smart backspace

### Preview (WebKitGTK6)
- Markdown parsed via `pulldown-cmark`
- HTML rendered dengan dark mode support
- Live update saat text berubah

### Copy-Paste Handler
Custom paste handler (`EventControllerKey`) yang force plain text dari clipboard, mencegah rich text formatting dari Discord/web apps.

## Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| Ctrl+N | New file |
| Ctrl+O | Open file |
| Ctrl+S | Save |
| Ctrl+Shift+S | Save As |
| Ctrl+P | Toggle Preview |

## Dependencies

```toml
gtk = { package = "gtk4", version = "0.10", features = ["v4_16"] }
adw = { package = "libadwaita", version = "0.8", features = ["v1_6"] }
sourceview = { package = "sourceview5", version = "0.10" }
webkit = { package = "webkit6", version = "0.5" }
pulldown-cmark = "0.12"
```

## Development Notes

### Bahasa
Komunikasi pakai Bahasa Indonesia santai (lu/gua).

### File Permissions
Claude Code run as root. Setelah edit:
```bash
chown -R pan:pan /home/pan/Dev/claude/apps/mymarkdown
```
