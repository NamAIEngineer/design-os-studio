#!/usr/bin/env sh
# Installs designos for the current user. No sudo needed.
#
#   ./install.sh            install under ~/.local
#   PREFIX=/some/dir ./install.sh   install elsewhere (bin/ and share/ below it)
#
# To remove: delete ~/.local/bin/designos, ~/.local/bin/designos-diff,
# ~/.local/share/applications/designos.desktop, the designos.png files under
# ~/.local/share/icons/hicolor/*/apps/ and ~/.local/share/mime/packages/designos.xml.
set -eu

here=$(cd "$(dirname "$0")" && pwd)
prefix=${PREFIX:-$HOME/.local}

install -d "$prefix/bin" "$prefix/share/applications" "$prefix/share/mime/packages"
install -m 755 "$here/designos" "$prefix/bin/designos"
[ -f "$here/designos-diff" ] && install -m 755 "$here/designos-diff" "$prefix/bin/designos-diff"
# Point Exec at the installed binary so launchers work even when ~/.local/bin is not on their PATH.
sed "s|^Exec=designos|Exec=$prefix/bin/designos|" "$here/designos.desktop" > "$prefix/share/applications/designos.desktop"
chmod 644 "$prefix/share/applications/designos.desktop"
[ -f "$here/designos-mime.xml" ] && install -m 644 "$here/designos-mime.xml" "$prefix/share/mime/packages/designos.xml"

for png in "$here"/icons/*.png; do
  size=$(basename "$png" .png)   # 16, 32, ... 512
  d="$prefix/share/icons/hicolor/${size}x${size}/apps"
  install -d "$d"
  install -m 644 "$png" "$d/designos.png"
done

command -v update-desktop-database >/dev/null 2>&1 && update-desktop-database "$prefix/share/applications" 2>/dev/null || true
command -v update-mime-database >/dev/null 2>&1 && update-mime-database "$prefix/share/mime" 2>/dev/null || true
command -v gtk-update-icon-cache >/dev/null 2>&1 && gtk-update-icon-cache -q -t "$prefix/share/icons/hicolor" 2>/dev/null || true

echo "designos installed to $prefix"
case ":$PATH:" in
  *":$prefix/bin:"*) ;;
  *) echo "note: $prefix/bin is not on your PATH; add it to run 'designos' from a terminal." ;;
esac
