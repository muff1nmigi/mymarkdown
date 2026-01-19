# MyMarkdown

A native GNOME markdown editor with live preview, built with Rust + GTK4 + libadwaita.

![License](https://img.shields.io/badge/license-MIT-blue.svg)
![Rust](https://img.shields.io/badge/rust-1.75+-orange.svg)
![GTK](https://img.shields.io/badge/GTK-4.16-green.svg)
![Platform](https://img.shields.io/badge/platform-Linux-lightgrey.svg)

## Features

- **Live Preview** - See your markdown rendered in real-time as you type
- **Split View** - Editor and preview side by side
- **Native GNOME Look** - Built with libadwaita for seamless desktop integration
- **Syntax Highlighting** - GtkSourceView5 with markdown highlighting
- **Dark Mode Support** - Follows system theme automatically
- **Plain Text Paste** - Strips rich formatting when pasting (fixes Discord/web copy-paste issues)
- **JetBrains Mono Font** - Beautiful monospace font for code

## Installation

### Dependencies

**Arch Linux / CachyOS:**
```bash
sudo pacman -S gtk4 libadwaita gtksourceview5 webkitgtk-6.0
```

**Fedora:**
```bash
sudo dnf install gtk4-devel libadwaita-devel gtksourceview5-devel webkitgtk6.0-devel
```

**Ubuntu 24.04+:**
```bash
sudo apt install libgtk-4-dev libadwaita-1-dev libgtksourceview-5-dev libwebkitgtk-6.0-dev
```

### Build from Source

```bash
# Clone the repository
git clone https://github.com/muff1nmigi/mymarkdown.git
cd mymarkdown

# Build release
cargo build --release

# Install (optional)
sudo cp target/release/mymd /usr/local/bin/
sudo cp data/org.gnome.MyMarkdown.desktop /usr/share/applications/
```

## Usage

```bash
# Open with empty file
mymd

# Create or open a file
mymd notes          # Creates/opens notes.md
mymd README.md      # Opens README.md
```

## Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `Ctrl+N` | New file |
| `Ctrl+O` | Open file |
| `Ctrl+S` | Save |
| `Ctrl+Shift+S` | Save As |
| `Ctrl+P` | Toggle Preview |

## Tech Stack

- **Language:** Rust
- **UI Toolkit:** GTK4 + libadwaita
- **Editor:** GtkSourceView5
- **Preview:** WebKitGTK6
- **Markdown Parser:** pulldown-cmark

## License

MIT License - see [LICENSE](LICENSE) for details.

## Contributing

Contributions are welcome! Feel free to open issues or submit pull requests.

---

Made with ❤️ and [Claude Code](https://claude.ai/code)
