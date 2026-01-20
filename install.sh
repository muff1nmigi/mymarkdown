#!/bin/bash
set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}=== MyMarkdown Installer ===${NC}"
echo ""

# Check if running as root for system-wide install
INSTALL_DIR="/usr/local/bin"
DESKTOP_DIR="/usr/share/applications"

if [ "$EUID" -ne 0 ]; then
    echo -e "${YELLOW}Note: Running without root. Will install to ~/.local instead.${NC}"
    INSTALL_DIR="$HOME/.local/bin"
    DESKTOP_DIR="$HOME/.local/share/applications"
    mkdir -p "$INSTALL_DIR" "$DESKTOP_DIR"
fi

# Check dependencies
echo -e "${YELLOW}Checking dependencies...${NC}"

check_cmd() {
    if ! command -v "$1" &> /dev/null; then
        echo -e "${RED}Error: $1 is not installed.${NC}"
        echo "Please install $1 and try again."
        exit 1
    fi
}

check_cmd cargo
check_cmd pkg-config

# Check GTK4 and libadwaita
if ! pkg-config --exists gtk4; then
    echo -e "${RED}Error: GTK4 development libraries not found.${NC}"
    echo "Install with: sudo pacman -S gtk4 (Arch) or sudo apt install libgtk-4-dev (Debian/Ubuntu)"
    exit 1
fi

if ! pkg-config --exists libadwaita-1; then
    echo -e "${RED}Error: libadwaita development libraries not found.${NC}"
    echo "Install with: sudo pacman -S libadwaita (Arch) or sudo apt install libadwaita-1-dev (Debian/Ubuntu)"
    exit 1
fi

if ! pkg-config --exists gtksourceview-5; then
    echo -e "${RED}Error: GtkSourceView5 development libraries not found.${NC}"
    echo "Install with: sudo pacman -S gtksourceview5 (Arch) or sudo apt install libgtksourceview-5-dev (Debian/Ubuntu)"
    exit 1
fi

if ! pkg-config --exists webkitgtk-6.0; then
    echo -e "${RED}Error: WebKitGTK6 development libraries not found.${NC}"
    echo "Install with: sudo pacman -S webkit2gtk-4.1 (Arch) or sudo apt install libwebkitgtk-6.0-dev (Debian/Ubuntu)"
    exit 1
fi

echo -e "${GREEN}All dependencies found!${NC}"
echo ""

# Build
echo -e "${YELLOW}Building release binary...${NC}"
cargo build --release

if [ ! -f "target/release/mymd" ]; then
    echo -e "${RED}Build failed!${NC}"
    exit 1
fi

echo -e "${GREEN}Build successful!${NC}"
echo ""

# Install binary
echo -e "${YELLOW}Installing binary to $INSTALL_DIR...${NC}"
cp target/release/mymd "$INSTALL_DIR/"
chmod 755 "$INSTALL_DIR/mymd"

# Install desktop file
echo -e "${YELLOW}Installing desktop file to $DESKTOP_DIR...${NC}"
cp data/org.gnome.MyMarkdown.desktop "$DESKTOP_DIR/"

# Update desktop database
if command -v update-desktop-database &> /dev/null; then
    update-desktop-database "$DESKTOP_DIR" 2>/dev/null || true
fi

echo ""
echo -e "${GREEN}=== Installation Complete ===${NC}"
echo ""
echo "You can now run MyMarkdown with:"
echo "  mymd              # Open with new file"
echo "  mymd filename     # Open/create filename.md"
echo ""
echo "Or find it in your application menu."
