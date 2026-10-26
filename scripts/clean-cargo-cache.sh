#!/usr/bin/env bash
# Remove only the shared Cargo target cache after confirming no Cargo compiler is running.
set -euo pipefail

root=$(cd "$(dirname "$0")/.." && pwd)
if pgrep -f '(^|/)(cargo|rustc)( |$)' >/dev/null 2>&1; then
  echo "clean-cargo-cache.sh: cargo/rustc is running; refusing to remove the cache" >&2
  exit 1
fi

if [ -z "${CARGO_TARGET_DIR:-}" ]; then
  target_dir="$(python3 "$root/scripts/cargo_target.py" "$root")"
else
  target_dir="$CARGO_TARGET_DIR"
fi
target_dir=$(python3 -c 'from pathlib import Path; import sys; print(Path(sys.argv[1]).expanduser().resolve())' "$target_dir")
if [ -z "$target_dir" ] || [ "$target_dir" = "$root" ] || [[ "$target_dir" == "$root"/* ]]; then
  echo "clean-cargo-cache.sh: target directory must be outside the checkout: $target_dir" >&2
  exit 1
fi

rm -rf "$target_dir"
echo "Removed $target_dir"
