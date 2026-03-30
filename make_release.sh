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

# Stage 0 bootstrap smoke gate (does not claim full self-hosting)
status "Running Stage 0 bootstrap smoke test..."
./bootstrap.sh > /dev/null

# Infra bootstrap gate: Stage1 must be a real compile, not placeholder output.
status "Running self-hosted bootstrap infra gate..."
bash omni-lang/bootstrap.sh --allow-infra-placeholders > /dev/null

BOOTSTRAP_STATUS_FILE="omni-lang/build/bootstrap_status.env"
if [ ! -f "$BOOTSTRAP_STATUS_FILE" ]; then
	echo -e "${RED}[-]${NC} Missing bootstrap status file: $BOOTSTRAP_STATUS_FILE"
	exit 1
fi

# shellcheck disable=SC1090
source "$BOOTSTRAP_STATUS_FILE"

if [ "${STAGE1_COMPILATION_OK:-0}" != "1" ]; then
	echo -e "${RED}[-]${NC} Release blocked: Stage1 self-hosted compiler compile failed (placeholder artifact detected)."
	exit 1
fi

if [ "${VERIFY_IDENTICAL:-0}" != "1" ]; then
	echo -e "${RED}[-]${NC} Release blocked: Stage1/Stage2 infra outputs are not identical."
	exit 1
fi

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
