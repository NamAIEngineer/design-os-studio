#!/usr/bin/env bash
# Remove only the shared Cargo target cache after confirming no Cargo compiler is running.
set -euo pipefail

if pgrep -f '(^|/)(cargo|rustc)( |$)' >/dev/null 2>&1; then
  echo "clean-cargo-cache.sh: cargo/rustc is running; refusing to remove the cache" >&2
  exit 1
fi

rm -rf /Users/nam/Library/Caches/design-os-studio/target
echo "Removed /Users/nam/Library/Caches/design-os-studio/target"
