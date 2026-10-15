#!/usr/bin/env bash
# Capture all 16 high-resolution Retina 2x visual proof screenshots for design:os studio (DIR-23).
# Runs headlessly via `--screenshot <path>` with `with_active(false)`.

set -euo pipefail

export PATH="$HOME/.cargo/bin:$PATH"

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT"

BIN="./target/release/designos-studio"
if [[ ! -x "$BIN" ]]; then
  echo "==> Building release binary ($BIN)..."
  cargo build --release -p designos-app
fi

mkdir -p docs/images

PROOF_LIB="/tmp/designos-proof-lib"
rm -rf "$PROOF_LIB"
echo "==> Seeding temporary showcase library at $PROOF_LIB..."
"$BIN" --seed-library "$PROOF_LIB"

echo "==> [1/16] Capturing docs/images/designos-hero.png..."
"$BIN" --fixture showcase --select "Home" --size 1440x900 --scale 2 \
  --screenshot docs/images/designos-hero.png

echo "==> [2/16] Capturing docs/images/designos-agents-chat.png..."
"$BIN" --fixture showcase --select "Home" --panel agents-chat --size 1440x900 --scale 2 \
  --screenshot docs/images/designos-agents-chat.png

echo "==> [3/16] Capturing docs/images/designos-spec-reader.png..."
"$BIN" --fixture showcase --panel specs --size 1600x1000 --scale 2 \
  --screenshot docs/images/designos-spec-reader.png

echo "==> [4/16] Capturing docs/images/designos-assets-panel.png..."
"$BIN" --fixture showcase --select "Home" --panel assets --size 1440x900 --scale 2 \
  --screenshot docs/images/designos-assets-panel.png

echo "==> [5/16] Capturing docs/images/designos-actions-cmdk.png..."
"$BIN" --fixture showcase --select "Home" --panel cmdk --size 1440x900 --scale 2 \
  --screenshot docs/images/designos-actions-cmdk.png

echo "==> [6/16] Capturing docs/images/designos-devmode.png..."
"$BIN" --fixture showcase --select "Home" --dev react --dev-demo --size 1440x900 --scale 2 \
  --screenshot docs/images/designos-devmode.png

echo "==> [7/16] Capturing docs/images/designos-variables.png..."
"$BIN" --fixture proto-vars --panel variables --size 1440x900 --scale 2 \
  --screenshot docs/images/designos-variables.png

echo "==> [8/16] Capturing docs/images/designos-components.png..."
"$BIN" --fixture showcase --select "Button" --size 1440x900 --scale 2 \
  --screenshot docs/images/designos-components.png

echo "==> [9/16] Capturing docs/images/designos-home.png..."
"$BIN" --library "$PROOF_LIB" --home --size 1440x900 --scale 2 \
  --screenshot docs/images/designos-home.png

echo "==> [10/16] Capturing docs/images/designos-prototype.png..."
"$BIN" --fixture proto-vars --tab prototype --panel interaction --size 1440x900 --scale 2 \
  --screenshot docs/images/designos-prototype.png

echo "==> [11/16] Capturing docs/images/designos-comments.png..."
"$BIN" --fixture comments --size 1440x900 --scale 2 \
  --screenshot docs/images/designos-comments.png

echo "==> [12/16] Capturing docs/images/designos-main-menu.png..."
"$BIN" --fixture showcase --select "Home" --menu main --size 1440x900 --scale 2 \
  --screenshot docs/images/designos-main-menu.png

echo "==> [13/16] Capturing docs/images/designos-color-picker.png..."
"$BIN" --fixture showcase --select "Balance card" --panel picker-gradient --size 1440x900 --scale 2 \
  --screenshot docs/images/designos-color-picker.png

echo "==> [14/16] Capturing docs/images/designos-agent-spec.png (NodeSpec -> Auto-Layout 5.0 -> .fig v106 roundtrip)..."
rm -f /tmp/agent-console.designos /tmp/agent-console.fig
"$BIN" /tmp/agent-console.designos --apply-spec examples/specs/easeui-agent-console.spec.json
"$BIN" /tmp/agent-console.designos --export-fig /tmp/agent-console.fig
"$BIN" /tmp/agent-console.fig --select "EaseUI — Agent-First Spec Console" --size 1440x900 --scale 2 \
  --screenshot docs/images/designos-agent-spec.png

echo "==> [15/16] Capturing docs/images/designos-overview.png..."
"$BIN" --fixture showcase --zoom 50 --rulers --size 1440x900 --scale 2 \
  --screenshot docs/images/designos-overview.png

echo "==> [16/16] Copying docs/images/designos-hero.png -> docs/images/cover.png..."
cp docs/images/designos-hero.png docs/images/cover.png

echo "==> Done! All 16 visual proof screenshots captured in docs/images/."
