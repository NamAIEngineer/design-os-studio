# Contributing to DESIGN:OS Studio

**DESIGN:OS Studio** (`design-os-studio`) is our native GPU-accelerated interface design editor and Figma engine built in Rust for the **DESIGN:OS / EaseUI** ecosystem.

## Core Engineering Principles

1. **Enterprise Scale First:** Every canvas, index, and serialization change must hold up on 100k–700k+ node `.fig` documents (`VSF - PCP` benchmark) with `< 16ms` interactive frame times and `0.0%` idle CPU.
2. **EaseUI BrainWave 2.0 Aesthetic:** Editor chrome follows the EaseUI BrainWave 2.0 dark/light token system (`crates/designos-app/src/theme.rs`).
3. **Deterministic Rendering:** `designos-render` (`tiny-skia`) and `designos-gpu` (`wgpu`) share strict visual parity verified via `designos-diff`.
4. **Undoable Command Model:** Every document mutation goes through `designos_doc::Command` with an exact inverse.

## Build and Run

```bash
export PATH="$HOME/.cargo/bin:$PATH"
cargo run --release -p designos-app --bin designos-studio               # Launch DESIGN:OS Studio
cargo run --release -p designos-app --bin designos-studio -- file.fig   # Open a Figma .fig file
```

## Test and Lint Gate

Run these before committing:

```bash
export PATH="$HOME/.cargo/bin:$PATH"
cargo test --workspace --lib
cargo test -p designos-gpu
cargo test -p designos-mcp
cargo clippy --workspace --all-targets -- -D warnings
```

## Headless Screenshots & Visual Diffing

```bash
designos-studio --open-id <id> --select "Frame" --zoom 60 --gpu --size 1440x900 --screenshot out.png
designos-diff compare reference.png out.png --out diff.png --tolerance 2 --threshold 0.001
```
