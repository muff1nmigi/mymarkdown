#!/bin/bash
set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${RED}=== MyMarkdown Uninstaller ===${NC}"
echo ""

# Determine install locations
if [ "$EUID" -eq 0 ]; then
    INSTALL_DIR="/usr/local/bin"
    DESKTOP_DIR="/usr/share/applications"
else
    INSTALL_DIR="$HOME/.local/bin"
    DESKTOP_DIR="$HOME/.local/share/applications"
fi

# Remove binary
if [ -f "$INSTALL_DIR/mymd" ]; then
    echo -e "${YELLOW}Removing binary from $INSTALL_DIR...${NC}"
    rm -f "$INSTALL_DIR/mymd"
    echo -e "${GREEN}Binary removed.${NC}"
else
    echo -e "${YELLOW}Binary not found in $INSTALL_DIR, checking other location...${NC}"
    # Try the other location
    if [ "$EUID" -eq 0 ]; then
        ALT_DIR="$HOME/.local/bin"
    else
        ALT_DIR="/usr/local/bin"
    fi
    if [ -f "$ALT_DIR/mymd" ]; then
        echo -e "${YELLOW}Found in $ALT_DIR. Run with sudo to remove system-wide install.${NC}"
    fi
fi

# Remove desktop file
if [ -f "$DESKTOP_DIR/org.gnome.MyMarkdown.desktop" ]; then
    echo -e "${YELLOW}Removing desktop file from $DESKTOP_DIR...${NC}"
    rm -f "$DESKTOP_DIR/org.gnome.MyMarkdown.desktop"
    echo -e "${GREEN}Desktop file removed.${NC}"
else
    echo -e "${YELLOW}Desktop file not found in $DESKTOP_DIR.${NC}"
fi

# Update desktop database
if command -v update-desktop-database &> /dev/null; then
    update-desktop-database "$DESKTOP_DIR" 2>/dev/null || true
fi

echo ""
echo -e "${GREEN}=== Uninstallation Complete ===${NC}"
echo ""
echo "MyMarkdown has been removed from your system."
echo "Your markdown files have NOT been deleted."
