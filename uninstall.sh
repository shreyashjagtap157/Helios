#!/bin/bash
# Omni Compiler Uninstallation Script
# Removes the Omni compiler from system directories

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

# Configuration
PREFIX="${PREFIX:-/usr/local}"
BIN_DIR="$PREFIX/bin"
LIB_DIR="$PREFIX/lib/omni"
SHARE_DIR="$PREFIX/share/omni"

status() {
    echo -e "${GREEN}[+]${NC} $1"
}

error() {
    echo -e "${RED}[-]${NC} $1"
}

warn() {
    echo -e "${YELLOW}[!]${NC} $1"
}

# Uninstall
uninstall() {
    status "Uninstalling Omni Compiler from $PREFIX..."
    
    # Remove binary
    if [ -f "$BIN_DIR/omnc" ]; then
        rm -f "$BIN_DIR/omnc"
        status "Removed $BIN_DIR/omnc"
    fi
    
    # Remove library
    if [ -d "$LIB_DIR" ]; then
        rm -rf "$LIB_DIR"
        status "Removed $LIB_DIR"
    fi
    
    # Remove share
    if [ -d "$SHARE_DIR" ]; then
        rm -rf "$SHARE_DIR"
        status "Removed $SHARE_DIR"
    fi
    
    # Clean up empty directories
    rmdir "$BIN_DIR" 2>/dev/null || true
    rmdir "$PREFIX/lib" 2>/dev/null || true
    rmdir "$PREFIX/share" 2>/dev/null || true
    
    status "Uninstallation complete!"
}

# User uninstall
uninstall_user() {
    PREFIX="$HOME/.local"
    uninstall
}

# System uninstall
uninstall_system() {
    if [ "$EUID" -eq 0 ]; then
        uninstall
    else
        warn "Please run with sudo for system-wide uninstall:"
        echo "  sudo ./uninstall.sh"
    fi
}

# Main
main() {
    echo "========================================"
    echo "Omni Compiler Uninstaller"
    echo "========================================"
    echo ""
    
    case "$1" in
        --user)
            uninstall_user
            ;;
        --system)
            uninstall_system
            ;;
        *)
            if [ "$EUID" -eq 0 ]; then
                uninstall_system
            else
                uninstall_user
            fi
            ;;
    esac
}

main "$@"
