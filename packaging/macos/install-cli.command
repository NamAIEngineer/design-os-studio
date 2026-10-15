#!/usr/bin/env bash
# design:os studio — macOS CLI & Gatekeeper helper
# Clears quarantine on /Applications/designos.app and symlinks the 4 CLI binaries:
#   designos-studio, designos, designos-mcp, designos-diff
set -euo pipefail

APP_PATH="/Applications/designos.app"
if [ ! -d "$APP_PATH" ]; then
  HERE="$(cd "$(dirname "$0")" && pwd)"
  if [ -d "$HERE/designos.app" ]; then
    echo "Copying designos.app to /Applications..."
    cp -R "$HERE/designos.app" "$APP_PATH"
  else
    echo "Error: $APP_PATH not found. Please drag designos.app to /Applications first." >&2
    exit 1
  fi
fi

echo "Clearing macOS Gatekeeper quarantine attribute on $APP_PATH..."
xattr -dr com.apple.quarantine "$APP_PATH" 2>/dev/null || true

BIN_TARGET="${DESIGNOS_BIN_DIR:-$HOME/.local/bin}"
if [ ! -d "$BIN_TARGET" ] && [ -w "/usr/local/bin" ]; then
  BIN_TARGET="/usr/local/bin"
else
  mkdir -p "$BIN_TARGET"
fi

for bin in designos-studio designos designos-mcp designos-diff; do
  if [ -x "$APP_PATH/Contents/MacOS/$bin" ]; then
    ln -sf "$APP_PATH/Contents/MacOS/$bin" "$BIN_TARGET/$bin"
    echo "Linked $BIN_TARGET/$bin -> $APP_PATH/Contents/MacOS/$bin"
  fi
done

echo ""
echo "design:os studio CLI tools installed to $BIN_TARGET"
case ":$PATH:" in
  *":$BIN_TARGET:"*) ;;
  *)
    echo "Note: add $BIN_TARGET to your shell PATH:"
    echo "  export PATH=\"$BIN_TARGET:\$PATH\""
    ;;
esac
