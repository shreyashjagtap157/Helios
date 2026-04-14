#!/usr/bin/env bash
set -euo pipefail

if [[ $# -lt 1 ]]; then
  echo "usage: $0 <external-helios-repo-path> [source-repo-root]" >&2
  exit 1
fi

EXTERNAL_REPO="$1"
SOURCE_ROOT="${2:-$(pwd)}"

cd "$SOURCE_ROOT"

if [[ ! -d .git ]]; then
  echo "error: $SOURCE_ROOT is not a git repository" >&2
  exit 1
fi

if [[ ! -d helios-framework ]]; then
  echo "error: helios-framework/ directory not found in $SOURCE_ROOT" >&2
  exit 1
fi

SPLIT_BRANCH="helios-extract-$(date +%Y%m%d%H%M%S)"

echo "[phase0.2] splitting helios-framework history into branch $SPLIT_BRANCH"
git subtree split --prefix=helios-framework -b "$SPLIT_BRANCH"

mkdir -p "$EXTERNAL_REPO"
if [[ ! -d "$EXTERNAL_REPO/.git" ]]; then
  git init "$EXTERNAL_REPO"
fi

echo "[phase0.2] importing split history into external HELIOS repository"
git -C "$EXTERNAL_REPO" fetch "$SOURCE_ROOT" "$SPLIT_BRANCH"
git -C "$EXTERNAL_REPO" checkout -B main FETCH_HEAD

echo "[phase0.2] removing helios-framework from Omni repository"
git rm -r --ignore-unmatch helios-framework || true
rm -rf helios-framework

echo "[phase0.2] scrubbing HELIOS references from root Cargo.toml"
if [[ -f omni-lang/Cargo.toml ]]; then
  if grep -Eiq 'helios|helios-framework' omni-lang/Cargo.toml; then
    cp omni-lang/Cargo.toml omni-lang/Cargo.toml.bak.phase0
    perl -i -ne 'print unless /helios-framework|helios/i' omni-lang/Cargo.toml
  fi
fi

echo "[phase0.2] validating HELIOS isolation"
if grep -R -Eiq 'helios-framework|helios' omni-lang/Cargo.toml; then
  echo "error: HELIOS references still present in omni-lang/Cargo.toml" >&2
  exit 2
fi

git branch -D "$SPLIT_BRANCH" >/dev/null 2>&1 || true

echo "phase0.2 completed"
echo "next: run 'cargo build --workspace --manifest-path omni-lang/Cargo.toml' to verify isolation"
