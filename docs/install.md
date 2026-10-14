# Installing DESIGN:OS Studio

Download the latest build for your platform from [GitHub Releases](https://github.com/jangtrinh/design-os-studio/releases) or build directly from source.

## Building & Packaging on macOS (Apple Silicon / Intel)

```bash
git clone https://github.com/jangtrinh/design-os-studio.git
cd design-os-studio
export PATH="$HOME/.cargo/bin:$PATH"

# Build optimized release binaries
cargo build --release -p designos-app -p designos-mcp
```

The macOS app bundle is located at `target/release/bundle/DESIGN-OS Studio.app` (with `designos-studio` and `designos` CLI binaries inside `Contents/MacOS/`).

If macOS Gatekeeper prompts on first launch of an unsigned build:

```bash
xattr -dr com.apple.quarantine "/Applications/DESIGN-OS Studio.app"
```

## GPU Rendering

DESIGN:OS Studio renders with `wgpu` (Metal on macOS, Vulkan on Linux/Windows, DirectX 12 on Windows) and automatically falls back to the deterministic `tiny-skia` CPU renderer when needed. Pass `--gpu` on the CLI to force GPU rendering in headless `--screenshot` mode.
