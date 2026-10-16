#!/usr/bin/env sh
# design:os model adapter (antigravity) — spec 013. Packet on stdin → arg → host model on stdout.
prompt="$(cat)"
exec agy --dangerously-skip-permissions -p "$prompt"
