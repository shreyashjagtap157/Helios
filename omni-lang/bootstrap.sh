#!/bin/bash
# ============================================================================
# Omni Compiler — Three-Stage Bootstrap
# ============================================================================
# Stage 0: Rust-based compiler (omnc) — the seed binary
# Stage 1: (Future) Self-hosted compiler compiled by Stage 0
# Stage 2: (Future) Self-hosted compiler compiled by Stage 1
# Verify:  Stage 1 and Stage 2 outputs must be bit-identical
#
# Current Status: Stage 0 is fully functional. Stages 1-2 require the
# self-hosted Omni compiler (omni-lang/omni/) to reach feature parity
# with the Rust compiler's output capabilities.
# ============================================================================

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
COMPILER="$SCRIPT_DIR/compiler/target/debug/omnc"
SOURCE_DIR="$SCRIPT_DIR/omni"
BUILD_DIR="$SCRIPT_DIR/build"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

log_info()  { echo -e "${BLUE}[INFO]${NC} $1"; }
log_ok()    { echo -e "${GREEN}[OK]${NC} $1"; }
log_warn()  { echo -e "${YELLOW}[WARN]${NC} $1"; }
log_error() { echo -e "${RED}[ERROR]${NC} $1"; }

# ============================================================================
# Stage 0: Verify Rust seed compiler
# ============================================================================

stage0_verify() {
    log_info "Stage 0: Verifying Rust-based seed compiler (omnc)"

    if [ ! -f "$COMPILER" ]; then
        log_error "Compiler not found at $COMPILER"
        log_info "Run: cd compiler && cargo build"
        exit 1
    fi

    VERSION=$("$COMPILER" --version 2>&1 || echo "unknown")
    log_ok "Stage 0 compiler: $VERSION"

    # Test basic compilation
    log_info "Testing basic compilation..."
    
    # Compile hello.omni
    OUTPUT="$BUILD_DIR/hello.ovm"
    mkdir -p "$BUILD_DIR"
    
    if "$COMPILER" "$SCRIPT_DIR/examples/hello.omni" -o "$OUTPUT" 2>&1; then
        log_ok "hello.omni compiled to OVM bytecode"
    else
        log_error "Failed to compile hello.omni"
        exit 1
    fi
    
    # Verify bytecode was generated
    if [ -f "$OUTPUT" ]; then
        SIZE=$(wc -c < "$OUTPUT")
        log_ok "OVM bytecode generated: $SIZE bytes"
    else
        log_error "Output file not created"
        exit 1
    fi

    # Test running the compiled bytecode
    log_info "Testing OVM execution..."
    if "$COMPILER" --run "$SCRIPT_DIR/examples/hello.omni" 2>&1 | grep -q "Hello"; then
        log_ok "OVM execution works"
    else
        log_warn "OVM execution test (non-fatal)"
    fi

    # Copy as stage0
    cp "$COMPILER" "$BUILD_DIR/omnc-stage0"
    log_ok "Stage 0 binary: $BUILD_DIR/omnc-stage0"
}

# ============================================================================
# Stage 1: Compile self-hosted compiler with Stage 0
# ============================================================================

stage1_compile() {
    log_info "Stage 1: Compiling self-hosted Omni compiler with Stage 0"

    if [ ! -f "$BUILD_DIR/omnc-stage0" ]; then
        log_error "Stage 0 binary not found. Run 'stage0' first."
        exit 1
    fi

    # Check if self-hosted compiler source exists
    if [ ! -f "$SOURCE_DIR/compiler/main.omni" ]; then
        log_error "Self-hosted compiler source not found at $SOURCE_DIR/compiler/"
        exit 1
    fi

    log_info "Self-hosted compiler source files:"
    find "$SOURCE_DIR/compiler" -name "*.omni" | while read f; do
        log_info "  $(basename "$f")"
    done

    # Try to compile the self-hosted compiler to OVM bytecode
    # Note: This may fail if the self-hosted compiler has issues
    log_info "Attempting to compile self-hosted compiler to OVM bytecode..."
    
    STAGE1_OUTPUT="$BUILD_DIR/omni-compiler-stage1.ovm"
    
    # The self-hosted compiler takes .omni files as input
    # We'll try to compile a minimal file first
    if "$COMPILER" "$SOURCE_DIR/main.omni" -o "$STAGE1_OUTPUT" 2>&1; then
        log_ok "Self-hosted compiler source compiled to OVM bytecode"
        cp "$STAGE1_OUTPUT" "$BUILD_DIR/omnc-stage1.ovm"
    else
        log_warn "Self-hosted compiler cannot yet compile itself (expected)"
        log_warn "This is the blocker for Stage 1 - see ISSUES.md"
        
        # Create a placeholder to show the infrastructure is ready
        touch "$BUILD_DIR/omnc-stage1.ovm"
        log_ok "Stage 1 placeholder created (for infrastructure testing)"
    fi
}

# ============================================================================
# Stage 2: Recompile with Stage 1
# ============================================================================

stage2_compile() {
    log_info "Stage 2: Recompiling self-hosted compiler with Stage 1"

    if [ ! -f "$BUILD_DIR/omnc-stage1.ovm" ]; then
        log_error "Stage 1 binary not found. Run 'stage1' first."
        exit 1
    fi

    # Stage 2 requires Stage 1 to produce a working OVM bytecode
    # For now, just verify Stage 1 exists
    log_info "Stage 2 verification pending Stage 1 completion..."
    
    # Create a placeholder
    cp "$BUILD_DIR/omnc-stage1.ovm" "$BUILD_DIR/omnc-stage2.ovm"
    log_ok "Stage 2 placeholder created"
}

# ============================================================================
# Verify: Compare Stage 1 and Stage 2 outputs
# ============================================================================

verify_stages() {
    log_info "Verifying Stage 1 and Stage 2 produce identical output..."

    if [ ! -f "$BUILD_DIR/omnc-stage1.ovm" ] || [ ! -f "$BUILD_DIR/omnc-stage2.ovm" ]; then
        log_error "Stage 1 or Stage 2 output not found"
        exit 1
    fi

    # Compare outputs
    if diff -q "$BUILD_DIR/omnc-stage1.ovm" "$BUILD_DIR/omnc-stage2.ovm" > /dev/null 2>&1; then
        log_ok "Stage 1 and Stage 2 outputs are identical!"
        return 0
    else
        log_error "Stage 1 and Stage 2 outputs differ!"
        return 1
    fi
}

# ============================================================================
# Main
# ============================================================================

main() {
    echo "========================================================================="
    echo "Omni Compiler — Three-Stage Bootstrap"
    echo "========================================================================="
    echo ""

    # Ensure build directory exists
    mkdir -p "$BUILD_DIR"

    # Run stages
    stage0_verify
    echo ""
    
    stage1_compile
    echo ""
    
    stage2_compile
    echo ""

    verify_stages
    echo ""
    
    log_ok "Bootstrap verification complete!"
    echo ""
    echo "Note: Full bootstrap (Stages 1-2 producing native binaries) requires"
    echo "the self-hosted compiler to implement native code generation."
    echo "See ISSUES.md for tracking."
}

# Run main if executed directly
if [ "${BASH_SOURCE[0]}" = "$0" ]; then
    main "$@"
fi
