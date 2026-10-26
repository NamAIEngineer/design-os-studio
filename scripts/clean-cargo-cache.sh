#!/usr/bin/env bash
# Remove only the shared Cargo target cache after confirming no Cargo compiler is running.
set -euo pipefail

root=$(cd "$(dirname "$0")/.." && pwd)
target_dir=$root/../.design-os-studio-target
if [ "$(uname -s)" = "Darwin" ]; then
  target_dir="$HOME/Library/Caches/design-os-studio/target"
fi
target_dir=${CARGO_TARGET_DIR:-$target_dir}

if pgrep -f '(^|/)(cargo|rustc)( |$)' >/dev/null 2>&1; then
  echo "clean-cargo-cache.sh: cargo/rustc is running; refusing to remove the cache" >&2
  exit 1
fi

rm -rf "$target_dir"
echo "Removed $target_dir"
