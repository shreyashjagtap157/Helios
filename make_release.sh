#!/bin/bash
# Omni Compiler Release Script
# Creates distribution packages for all platforms

set -e

RELEASE_DIR="release"
VERSION="1.0.0"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m'

status() { echo -e "${GREEN}[+]${NC} $1"; }

mkdir -p "$RELEASE_DIR"

# Build release
status "Building release binary..."
cd omni-lang/compiler
cargo build --release
cd ../..

# Create directory structure
PLATFORM="omni-$VERSION-x86_64-pc-windows-msvc"
mkdir -p "$RELEASE_DIR/$PLATFORM"

# Copy files
cp omni-lang/compiler/target/release/omnc.exe "$RELEASE_DIR/$PLATFORM/"
cp -r omni-lang/omni "$RELEASE_DIR/$PLATFORM/"
cp -r omni-lang/std "$RELEASE_DIR/$PLATFORM/"
cp -r omni-lang/examples "$RELEASE_DIR/$PLATFORM/"
cp LICENSE "$RELEASE_DIR/$PLATFORM/"
cp README.md "$RELEASE_DIR/$PLATFORM/"

# Create zip
cd "$RELEASE_DIR"
zip -r "$PLATFORM.zip" "$PLATFORM"
cd ..

status "Release created: $RELEASE_DIR/$PLATFORM.zip"
ls -lh "$RELEASE_DIR/"
