#!/usr/bin/env sh
# Uninstalls design:os studio from PREFIX (default ~/.local).
set -eu

prefix=${PREFIX:-$HOME/.local}

for bin in designos-studio designos designos-mcp designos-diff; do
  rm -f "$prefix/bin/$bin"
done

rm -f "$prefix/share/applications/designos.desktop"
rm -f "$prefix/share/mime/packages/designos.xml"

for size in 16 32 48 64 128 256 512; do
  rm -f "$prefix/share/icons/hicolor/${size}x${size}/apps/designos.png"
done

command -v update-desktop-database >/dev/null 2>&1 && update-desktop-database "$prefix/share/applications" 2>/dev/null || true
command -v update-mime-database >/dev/null 2>&1 && update-mime-database "$prefix/share/mime" 2>/dev/null || true
command -v gtk-update-icon-cache >/dev/null 2>&1 && gtk-update-icon-cache -q -t "$prefix/share/icons/hicolor" 2>/dev/null || true

echo "design:os studio removed from $prefix"
