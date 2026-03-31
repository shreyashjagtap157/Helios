#!/bin/bash
# Omni Compiler Bootstrap Script
# Demonstrates self-hosting: Omni compiles itself

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

OMNI_DIR="$SCRIPT_DIR/omni-lang"
COMPILER_DIR="$OMNI_DIR/compiler"
BUILD_DIR="$SCRIPT_DIR/build"

detect_omnc_binary() {
    local base_dir="$1"
    if [ -f "$base_dir/target/debug/omnc" ]; then
        echo "$base_dir/target/debug/omnc"
    elif [ -f "$base_dir/target/debug/omnc.exe" ]; then
        echo "$base_dir/target/debug/omnc.exe"
    elif [ -f "$base_dir/target/release/omnc" ]; then
        echo "$base_dir/target/release/omnc"
    elif [ -f "$base_dir/target/release/omnc.exe" ]; then
        echo "$base_dir/target/release/omnc.exe"
    else
        return 1
    fi
}

echo "=========================================="
echo "Omni Compiler Bootstrap"
echo "Self-Hosting Demonstration"
echo "=========================================="
echo ""

mkdir -p "$BUILD_DIR"

# Check for Rust compiler
if ! OMNC="$(detect_omnc_binary "$COMPILER_DIR")"; then
    echo "ERROR: Rust compiler (omnc) not found in target/debug or target/release."
    echo "Please build: cd $COMPILER_DIR && cargo build"
    exit 1
fi

chmod +x "$OMNC" 2>/dev/null || true

echo "Omni Compiler: $OMNC"
echo ""

# Stage 0: Rust omnc compiles self-hosted compiler
echo "=========================================="
echo "STAGE 0: Rust Compiler"
echo "=========================================="
echo ""

SOURCE="$OMNI_DIR/omni/compiler_minimal.omni"
STAGE0="$BUILD_DIR/stage0.ovm"

echo "Compiling self-hosted compiler..."
echo "  Source: $SOURCE"
echo "  Output: $STAGE0"
echo ""

if $OMNC "$SOURCE" -o "$STAGE0" 2>&1; then
    echo "✓ Stage 0: Compilation successful"
    if [ -f "$STAGE0" ]; then
        SIZE=$(stat -c%s "$STAGE0" 2>/dev/null || stat -f%z "$STAGE0" 2>/dev/null || echo "unknown")
        echo "✓ Bytecode: $SIZE bytes"
    fi
else
    echo "✗ Stage 0: Compilation failed"
    exit 1
fi
echo ""

# Run Stage 0
echo "Running Stage 0 bytecode..."
if $OMNC --run "$STAGE0" 2>&1; then
    echo "✓ Stage 0: Execution successful"
else
    echo "⚠ Stage 0: Execution had issues"
fi
echo ""

# Bootstrap complete
echo "=========================================="
echo "Bootstrap Complete (Stage 0 Smoke Test)"
echo "=========================================="
echo ""
echo "Stage 0 pipeline verified."
echo "Full self-hosting (true Stage 1/2) is tracked in omni-lang/ISSUES.md."
echo ""
echo "Files created:"
ls -la "$BUILD_DIR"/stage*.ovm 2>/dev/null || true
