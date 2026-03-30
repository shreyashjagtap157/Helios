#!/bin/bash
# Omni Compiler Installation Script
# Installs the Omni compiler to system directories

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Configuration
PREFIX="${PREFIX:-/usr/local}"
BIN_DIR="$PREFIX/bin"
LIB_DIR="$PREFIX/lib/omni"
SHARE_DIR="$PREFIX/share/omni"
MAN_DIR="$PREFIX/share/man/man1"

# Detect OS
detect_os() {
    case "$(uname -s)" in
        Linux*)     echo "linux";;
        Darwin*)    echo "macos";;
        CYGWIN*)    echo "windows";;
        MINGW*)     echo "windows";;
        *)          echo "unknown";;
    esac
}

# Detect architecture
detect_arch() {
    case "$(uname -m)" in
        x86_64)     echo "x86_64";;
        aarch64|arm64) echo "aarch64";;
        *)          echo "x86_64";;
    esac
}

# Print status
status() {
    echo -e "${GREEN}[+]${NC} $1"
}

# Print error
error() {
    echo -e "${RED}[-]${NC} $1"
}

# Print warning
warn() {
    echo -e "${YELLOW}[!]${NC} $1"
}

# Check if running as root (for system install)
check_root() {
    if [ "$PREFIX" != "$HOME" ] && [ "$EUID" -ne 0 ]; then
        warn "Installing to $PREFIX requires root privileges"
        warn "Run with sudo or set PREFIX=$HOME for user install"
    fi
}

# Build the compiler
build() {
    status "Building Omni Compiler..."
    
    cd "$(dirname "$0")/omni-lang/compiler"
    
    if [ "$(detect_os)" = "windows" ]; then
        cargo build --release
    else
        cargo build --release
    fi
    
    status "Build complete!"
}

# Install the compiler
install_compiler() {
    status "Installing Omni Compiler to $PREFIX..."
    
    # Create directories
    mkdir -p "$BIN_DIR"
    mkdir -p "$LIB_DIR"
    mkdir -p "$SHARE_DIR"
    mkdir -p "$MAN_DIR"
    
    # Copy binary
    cd "$(dirname "$0")/omni-lang/compiler"
    cp target/release/omnc "$BIN_DIR/" 2>/dev/null || cp target/debug/omnc "$BIN_DIR/"
    chmod +x "$BIN_DIR/omnc"
    
    # Copy runtime
    cd "$(dirname "$0")"
    cp -r omni-lang/omni "$LIB_DIR/"
    cp -r omni-lang/std "$SHARE_DIR/"
    cp -r omni-lang/examples "$SHARE_DIR/"
    
    # Copy documentation
    cp LICENSE "$PREFIX/share/omni/" 2>/dev/null || true
    cp README.md "$PREFIX/share/omni/" 2>/dev/null || true
    
    status "Installation complete!"
    echo ""
    echo "Omni Compiler installed to: $BIN_DIR/omnc"
    echo "Library installed to: $LIB_DIR"
    echo "Standard library: $SHARE_DIR/std"
    echo ""
    echo "Add $BIN_DIR to your PATH to use omnc"
}

# User install (no sudo required)
install_user() {
    PREFIX="$HOME/.local"
    install_compiler
}

# System install (requires sudo)
install_system() {
    check_root
    if [ "$EUID" -eq 0 ]; then
        install_compiler
    else
        warn "Please run with sudo for system-wide install:"
        echo "  sudo ./install.sh"
    fi
}

# Main
main() {
    echo "========================================"
    echo "Omni Compiler Installer"
    echo "========================================"
    echo ""
    
    OS=$(detect_os)
    ARCH=$(detect_arch)
    
    echo "Detected: $OS ($ARCH)"
    echo "Install prefix: ${PREFIX:-/usr/local}"
    echo ""
    
    case "$1" in
        --user)
            install_user
            ;;
        --system)
            install_system
            ;;
        --build)
            build
            ;;
        *)
            # Interactive mode
            echo "Choose installation type:"
            echo "  1) User install (~/.local)"
            echo "  2) System install (/usr/local) - requires sudo"
            echo "  3) Build only"
            echo ""
            read -p "Choice [1-3]: " choice
            
            case "$choice" in
                1) install_user ;;
                2) install_system ;;
                3) build ;;
                *) error "Invalid choice"; exit 1 ;;
            esac
            ;;
    esac
}

main "$@"
