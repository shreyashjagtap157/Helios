#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="${1:-$(pwd)}"
cd "$ROOT_DIR"

if [[ ! -d .git ]]; then
  echo "error: $ROOT_DIR is not a git repository" >&2
  exit 1
fi

echo "[phase0.1] running full git object integrity scan"
git fsck --full --strict

echo "[phase0.1] expiring reflogs and pruning unreachable objects"
git reflog expire --expire=now --all
git gc --prune=now --aggressive
git prune --expire=now
git repack -Ad

echo "[phase0.1] cleaning every Cargo manifest target tree"
mapfile -d '' manifests < <(find . -type f -name Cargo.toml -not -path "*/target/*" -print0)
for manifest in "${manifests[@]}"; do
  echo "  -> cargo clean --manifest-path ${manifest#./}"
  cargo clean --manifest-path "$manifest"
done

echo "[phase0.1] rebuilding git index from HEAD"
rm -f .git/index
git reset --mixed HEAD

echo "[phase0.1] verifying tracked files are byte-readable"
unreadable=0
while IFS= read -r -d '' tracked; do
  if [[ ! -r "$tracked" ]]; then
    echo "unreadable tracked file: $tracked" >&2
    unreadable=$((unreadable + 1))
  fi
done < <(git ls-files -z)

if [[ "$unreadable" -ne 0 ]]; then
  echo "phase0.1 failed: $unreadable unreadable tracked file(s) detected" >&2
  exit 2
fi

echo "phase0.1 completed: 0 unreadable tracked files"
