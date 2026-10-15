#!/usr/bin/env sh
# Installs design:os studio (designos-studio, designos, designos-mcp, designos-diff)
# for the current user. No sudo needed by default.
#
#   ./install.sh                    install under ~/.local
#   PREFIX=/usr/local ./install.sh  install system-wide
set -eu

here=$(cd "$(dirname "$0")" && pwd)
prefix=${PREFIX:-$HOME/.local}

install -d "$prefix/bin" "$prefix/share/applications" "$prefix/share/mime/packages"

for bin in designos-studio designos designos-mcp designos-diff; do
  if [ -f "$here/$bin" ]; then
    install -m 755 "$here/$bin" "$prefix/bin/$bin"
  fi
done

# Point Exec and TryExec at the installed binary so desktop launchers work even when ~/.local/bin is not on PATH.
sed \
  -e "s|^Exec=designos-studio|Exec=$prefix/bin/designos-studio|" \
  -e "s|^Exec=designos|Exec=$prefix/bin/designos-studio|" \
  -e "s|^TryExec=designos-studio|TryExec=$prefix/bin/designos-studio|" \
  "$here/designos.desktop" > "$prefix/share/applications/designos.desktop"
chmod 644 "$prefix/share/applications/designos.desktop"

[ -f "$here/designos-mime.xml" ] && install -m 644 "$here/designos-mime.xml" "$prefix/share/mime/packages/designos.xml"

for png in "$here"/icons/*.png; do
  [ -f "$png" ] || continue
  size=$(basename "$png" .png)   # 16, 32, ... 512
  d="$prefix/share/icons/hicolor/${size}x${size}/apps"
  install -d "$d"
  install -m 644 "$png" "$d/designos.png"
done

command -v update-desktop-database >/dev/null 2>&1 && update-desktop-database "$prefix/share/applications" 2>/dev/null || true
command -v update-mime-database >/dev/null 2>&1 && update-mime-database "$prefix/share/mime" 2>/dev/null || true
command -v gtk-update-icon-cache >/dev/null 2>&1 && gtk-update-icon-cache -q -t "$prefix/share/icons/hicolor" 2>/dev/null || true

echo "design:os studio installed to $prefix"
echo "Installed binaries: designos-studio, designos, designos-mcp, designos-diff"
case ":$PATH:" in
  *":$prefix/bin:"*) ;;
  *) echo "note: $prefix/bin is not on your PATH; add it to run 'designos' or 'designos-mcp' from a terminal." ;;
esac
