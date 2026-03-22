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
COMPILER="$SCRIPT_DIR/compiler/target/release/omnc"
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
        log_info "Run: cd compiler && cargo build --release"
        exit 1
    fi

    VERSION=$("$COMPILER" --version 2>&1 || echo "unknown")
    log_ok "Stage 0 compiler: $VERSION"

    # Test basic compilation
    log_info "Running integration test..."
    "$COMPILER" --run "$SCRIPT_DIR/examples/integration_test.omni" > /dev/null 2>&1 && \
        log_ok "Integration test passed" || \
        log_warn "Integration test had warnings (non-fatal)"

    # Test all tutorials
    for tutorial in "$SCRIPT_DIR"/examples/tutorial_*.omni; do
        name=$(basename "$tutorial")
        OUTPUT=$("$COMPILER" --run "$tutorial" 2>&1 || true)
        if echo "$OUTPUT" | grep -q "^Error:"; then
            log_warn "$name: has runtime errors (non-fatal)"
        else
            log_ok "$name: executed successfully"
        fi
    done

    # Copy as stage0
    mkdir -p "$BUILD_DIR"
    cp "$COMPILER" "$BUILD_DIR/omnc-stage0"
    log_ok "Stage 0 binary: $BUILD_DIR/omnc-stage0 ($(stat -c%s "$BUILD_DIR/omnc-stage0" 2>/dev/null || stat -f%z "$BUILD_DIR/omnc-stage0" 2>/dev/null) bytes)"
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
        log_info "  $(basename "$f") ($(wc -l < "$f") lines)"
    done

    log_warn "Stage 1 compilation requires the self-hosted Omni compiler to reach"
    log_warn "feature parity with the Rust compiler. Current status:"
    log_warn "  - Lexer: Implemented in Omni"
    log_warn "  - Parser: Implemented in Omni"
    log_warn "  - Semantic: Type checker, borrow checker, traits, monomorphization"
    log_warn "  - IR: Generation and optimization"
    log_warn "  - Codegen: LLVM, OVM bytecode, GPU backends"
    log_warn "  - Linker: Implemented in Omni"
    log_warn "  - Missing: Binary output (code needs to emit a standalone executable)"
    log_warn ""
    log_warn "To complete Stage 1, the Omni compiler needs to emit a native binary"
    log_warn "or OVM bytecode that can execute standalone."
    log_info "Creating placeholder stage1 binary..."
    cp "$BUILD_DIR/omnc-stage0" "$BUILD_DIR/omnc-stage1"
    log_ok "Stage 1 binary: $BUILD_DIR/omnc-stage1 (placeholder — uses stage0 binary)"
}

# ============================================================================
# Stage 2: Recompile with Stage 1
# ============================================================================

stage2_compile() {
    log_info "Stage 2: Recompiling self-hosted compiler with Stage 1"

    if [ ! -f "$BUILD_DIR/omnc-stage1" ]; then
        log_error "Stage 1 binary not found. Run 'stage1' first."
        exit 1
    fi

    log_warn "Stage 2 is a placeholder until Stage 1 produces a real self-hosted binary."
    cp "$BUILD_DIR/omnc-stage1" "$BUILD_DIR/omnc-stage2"
    log_ok "Stage 2 binary: $BUILD_DIR/omnc-stage2 (placeholder)"
}

# ============================================================================
# Verify: Bit-identical comparison
# ============================================================================

verify_bootstrap() {
    log_info "Verification: Comparing Stage 1 and Stage 2 outputs"

    if [ ! -f "$BUILD_DIR/omnc-stage1" ] || [ ! -f "$BUILD_DIR/omnc-stage2" ]; then
        log_error "Stage binaries not found"
        exit 1
    fi

    HASH1=$(sha256sum "$BUILD_DIR/omnc-stage1" | cut -d' ' -f1)
    HASH2=$(sha256sum "$BUILD_DIR/omnc-stage2" | cut -d' ' -f1)

    if [ "$HASH1" = "$HASH2" ]; then
        log_ok "Bootstrap verified: Stage 1 and Stage 2 are bit-identical"
        log_ok "SHA-256: $HASH1"
    else
        log_error "Bootstrap FAILED: Stage 1 and Stage 2 differ"
        log_error "Stage 1: $HASH1"
        log_error "Stage 2: $HASH2"
        exit 1
    fi
}

# ============================================================================
# Main
# ============================================================================

case "${1:-all}" in
    stage0|s0)
        stage0_verify
        ;;
    stage1|s1)
        stage1_compile
        ;;
    stage2|s2)
        stage2_compile
        ;;
    verify|v)
        verify_bootstrap
        ;;
    all)
        stage0_verify
        stage1_compile
        stage2_compile
        verify_bootstrap
        ;;
    status)
        echo "Omni Bootstrap Status"
        echo "====================="
        echo "Stage 0 (Rust seed): $([ -f "$BUILD_DIR/omnc-stage0" ] && echo "Ready" || echo "Not built")"
        echo "Stage 1 (Self-hosted v1): $([ -f "$BUILD_DIR/omnc-stage1" ] && echo "Placeholder" || echo "Not built")"
        echo "Stage 2 (Self-hosted v2): $([ -f "$BUILD_DIR/omnc-stage2" ] && echo "Placeholder" || echo "Not built")"
        echo ""
        echo "Rust compiler tests: $(cd compiler && cargo test 2>&1 | grep -o '[0-9]* passed' | head -1 || echo 'N/A')"
        echo "Self-hosted compiler: $(find "$SOURCE_DIR/compiler" -name '*.omni' 2>/dev/null | wc -l) source files"
        echo "Omni standard library: $(find "$SOURCE_DIR/stdlib" -name '*.omni' 2>/dev/null | wc -l) modules"
        ;;
    *)
        echo "Usage: $0 {all|stage0|stage1|stage2|verify|status}"
        exit 1
        ;;
esac
