# DESIGN:OS Studio — Domain Glossary (`CONTEXT.md`)

**Document (`designos_doc::Document`)**:
The canonical in-memory and serialized design/board/deck tree containing pages, `NodeId -> Node` map, shared styles, design variables, flows, comments, library assets, and embedded images.
_Avoid_: file blob, raw kiwi tree, canvas state.

**Document Revision (`TileView::rev`)**:
A monotonically increasing counter (`u64`) that increments ONLY when the `Document` is mutated (`App::changed()`). Never increment on viewport-only changes (pan, zoom, fit, marquee).
_Avoid_: frame counter, view revision.

**SceneIndex (`designos_render::SceneIndex`)**:
Cached spatial and paint-order index of a single page at a specific `(page_id, rev)`, storing world transforms, world bounding boxes (`bounds`), and ancestor clips for fast viewport culling and hit-testing.
_Avoid_: ad-hoc tree walk, per-frame bounds recomputation.

**Compact Native Format (`.designos` / `.designos`)**:
Single-line JSON representation of `Document` with `skip_serializing_if` on default/empty `Node` fields and `"hex:..."` encoding for embedded image bytes, maintaining 100% backward compatibility when loading legacy pretty-printed `.designos` files.
_Avoid_: multi-gigabyte pretty-printed byte arrays.

**Studio Agent Runtime (`crates/designos-app/src/terminal.rs` + `crates/designos-mcp`)**:
The local-first AI engine powering `design:os studio` where the user logs into their own subscription CLI (`claude`, `codex`, `agy` / `antigravity`, `grok`, or `$SHELL`) inside an embedded `portable-pty` + `vt100` terminal shell with pre-wired environment variables (`DESIGNOS_BIN`, `DESIGNOS_FILE`, `DESIGNOS_MCP_SOCKET`) and live bidirectional Canvas/Document access.
_Avoid_: cloud black-box AI wrapper, hardcoded API-key chat bot.

**Figma AI Action Bridge (`crates/designos-app/src/cmdk.rs` + `panels/rail.rs`)**:
The 1:1 Figma UI3 AI & Agents surface (`Agents` left-rail panel, `Actions ⌘K` AI tab, and contextual canvas AI pills: `First Draft / Make Design`, `Make Prototype`, `Rename Layers`, `Rewrite / Translate / Replace Content`, `Design Gate Audit & Fix`) that routes structured prompts + live selection context (`NodeSpec` JSON, tokens, comments) into the active `Studio Agent Runtime` PTY session (or executes deterministic local transforms immediately) and applies the resulting `NodeSpec`/diff back onto the `Document` with 1-step Undo (`History`).
_Avoid_: disconnected terminal tab, fake static mock AI responses.

**Unified Phosphor Icons System (`crates/designos-app/src/icons.rs` + `assets/icons/phosphor/`)**:
The canonical 100% unified vector icon system across all **`design`:os studio** UI3 chrome and `product-os-spec-driven` Reader surfaces (`DIR-19`, `BR-22`, `D17`), powered by official Phosphor Icons (`https://phosphoricons.com/`, `@phosphor-icons/core` `viewBox="0 0 256 256"`, `fill="currentColor"`) vendored locally and rasterized via `resvg` with DPR-aware pixel snapping while preserving the `Icon::Logo` brand mark.
_Avoid_: mixed icon families (Lucide/Feather/Heroicons), ad-hoc procedural rail/panel icon drawing, emoji icons in UI chrome.

**Agent Chat Wrapper (`crates/designos-app/src/terminal.rs` · `AgentChatTurn` / `AgentChatRole`)**:
The pure native Chat UI wrapper (`DIR-21`, `EP-15`, `REQ-18`, `US-15`, `BR-23`, `D18`) rendered inside the Figma UI3 `Agents` Left Side Panel (`RailItem::Agents`, `S13`) directly on top of the active `portable-pty` + `vt100` terminal shell of **Claude Code (`claude`)**, **Codex (`codex`)**, and **Antigravity CLI (`agy`)** — featuring a 1-click CLI runtime selector pill bar (`Claude Code | Codex | Antigravity`), structured User/Assistant chat turn bubbles (`AgentChatTurn` / `AgentChatRole`) streaming ANSI-stripped stdout (`sync_chat_from_pty` + `strip_ansi_codes`), inline `NodeSpec` detection & 1-click/auto Canvas apply badge inside chat bubbles, Selection + Pin-Comment context attachment (`+`), and instant toggle between the clean Chat thread view and the underlying raw VT100 terminal screen with zero external paid API keys required.
_Avoid_: cloud black-box chat APIs, separate non-PTY mock chat state, raw unstripped ANSI escape noise inside chat bubbles.

**Cross-Platform Release Build & Packaging Workflow (`scripts/build-release.sh` + `.github/workflows/release.yml` · `DIR-22`)**:
The unified user-installable release packaging pipeline (`DIR-22`, `EP-16`, `REQ-19`, `BR-24`, `D19`) bundling all 4 binaries (`designos-studio`, `designos`, `designos-mcp`, `designos-diff`) for **macOS** (`scripts/package-macos.sh` → `.app` `os.design.studio` + drag-to-Applications `.dmg` + `install-cli.command` + `.tar.gz`), **Windows** (`scripts/package-windows.ps1` + `packaging/windows/installer.nsi` → NSIS `-setup.exe` + PowerShell `install.ps1`/`uninstall.ps1` + `.zip`), and **Linux** (`scripts/package-linux.sh` → Debian/Ubuntu `.deb` + portable `.tar.gz` with `./install.sh`, `.desktop`, MIME XML, `AppRun`), and emitting `dist/release-manifest.json` + `SHA256SUMS` for `updater.rs` Layer 1 atomic self-update.
_Avoid_: raw unbundled target binaries without OS desktop/CLI integration or unsigned/unhashed release archives.

