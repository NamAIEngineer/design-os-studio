#!/usr/bin/env node
/**
 * Builds the 3 standalone 1440x1440 Dribbble One-Shape UI Motion HTML engines
 * for Design OS Studio Launch, powered by REAL Design OS Studio Retina 2x assets
 * (Design/marketing/screenshots/01..16-*.png) combined with interactive UI elements
 * (liquid-edge tab indicators, live cursor clicks/drags, key badges, callout HUDs,
 * and spring-driven focal zoom/pan over the real native Rust + wgpu Studio UI).
 *
 * Hard invariants:
 * - Pure function of time inside window.seek(t): no CSS transitions, no animations, no timers.
 * - Closed-form 2nd-order step-response springs M.S(tau, w, z).
 * - Asymmetric leading/trailing edge springs M.liquidEdges for liquid tab/toggle indicators.
 * - Zero will-change on scaled elements so text and 2x Retina screenshots stay razor-sharp.
 * - Non-overlapping blur-swap M.vis(b, bStart, bEnd) for state transitions.
 * - Exact loop closure: Frame(0) === Frame(DURATION).
 */
import fs from 'node:fs';
import path from 'node:path';

const ROOT = '/Users/jangtrinh/Products/design-os-studio';
const OUT_DIR = path.join(ROOT, 'Design/marketing/svg-animation');
fs.mkdirSync(OUT_DIR, { recursive: true });

const GEIST_FONTS_CSS = fs.readFileSync(
  path.join(OUT_DIR, 'geist-fonts.css'),
  'utf8'
);

const SHARED_CSS = `
${GEIST_FONTS_CSS}


:root {
  --canvas-bg: #F4F3EF;
  --ink: #18181B;
  --ink-soft: #27272A;
  --ink-muted: #71717A;
  --surface: #FFFFFF;
  --border: #E4E4E7;
  --orange: #E36323;
  --blue: #0D99FF;
  --pink: #F24822;
  --emerald: #14AE5C;
  --font-sans: 'Geist', -apple-system, BlinkMacSystemFont, sans-serif;
  --font-mono: 'Geist Mono', monospace;
}

* {
  box-sizing: border-box;
  margin: 0;
  padding: 0;
  user-select: none;
  -webkit-font-smoothing: antialiased;
}

html, body {
  width: 1440px;
  height: 1440px;
  overflow: hidden;
  background: var(--canvas-bg);
  font-family: var(--font-sans);
  color: var(--ink);
}

#stage {
  position: relative;
  width: 1440px;
  height: 1440px;
  overflow: hidden;
  background-color: var(--canvas-bg);
  background-image: radial-gradient(rgba(24, 24, 27, 0.11) 1.25px, transparent 1.25px);
  background-size: 24px 24px;
}

#hud-top {
  position: absolute;
  top: 38px;
  left: 48px;
  right: 48px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  z-index: 30;
  pointer-events: none;
}

.brand-pill {
  display: inline-flex;
  align-items: center;
  gap: 10px;
  padding: 10px 18px;
  border-radius: 999px;
  background: rgba(255, 255, 255, 0.92);
  border: 1px solid rgba(24, 24, 27, 0.10);
  box-shadow: 0 8px 24px rgba(24, 24, 27, 0.06);
  font-size: 15px;
  font-weight: 600;
  letter-spacing: -0.01em;
}

.brand-dot {
  width: 10px;
  height: 10px;
  border-radius: 50%;
  background: var(--orange);
}

.beat-pill {
  font-family: var(--font-mono);
  font-size: 13px;
  font-weight: 700;
  padding: 9px 16px;
  border-radius: 999px;
  background: var(--ink);
  color: #FFFFFF;
  letter-spacing: 0.04em;
  box-shadow: 0 8px 20px rgba(24, 24, 27, 0.14);
}

#camera {
  position: absolute;
  inset: 0;
  transform-origin: 720px 720px;
}

/* THE ONE MORPHING SHAPE */
#shape {
  position: absolute;
  left: 720px;
  top: 720px;
  overflow: hidden;
  background: var(--ink);
  border: 1.5px solid rgba(24, 24, 27, 0.14);
  box-shadow:
    0 34px 80px rgba(24, 24, 27, 0.20),
    0 8px 24px rgba(24, 24, 27, 0.10);
}

.state-panel {
  position: absolute;
  inset: 0;
  display: flex;
  flex-direction: column;
  opacity: 0;
  pointer-events: none;
}

/* Real Studio Window Chrome & Asset Viewport */
.studio-window-header {
  height: 44px;
  background: #141417;
  border-bottom: 1px solid rgba(255, 255, 255, 0.09);
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 18px;
  color: #FFFFFF;
  flex-shrink: 0;
  z-index: 5;
}

.traffic-dots {
  display: flex;
  align-items: center;
  gap: 7px;
}
.traffic-dots span {
  width: 11px;
  height: 11px;
  border-radius: 50%;
}
.dot-r { background: #FF5F56; }
.dot-y { background: #FFBD2E; }
.dot-g { background: #27C93F; }

.studio-window-title {
  display: flex;
  align-items: center;
  gap: 10px;
  font-size: 13.5px;
  font-weight: 600;
  color: #E4E4E7;
}

.studio-window-badge {
  font-family: var(--font-mono);
  font-size: 12px;
  font-weight: 700;
  padding: 4px 10px;
  border-radius: 6px;
  background: var(--orange);
  color: #FFFFFF;
}

.studio-viewport {
  position: relative;
  flex: 1;
  overflow: hidden;
  background: #1B1B1F;
}

.studio-screenshot {
  position: absolute;
  left: 0;
  top: 0;
  width: 100%;
  height: 100%;
  object-fit: cover;
  transform-origin: 50% 50%;
}

/* Interactive UI Overlays layered on top of real Studio screenshot */
.ui-overlay-card {
  position: absolute;
  background: rgba(24, 24, 27, 0.94);
  color: #FFFFFF;
  border: 1.5px solid rgba(255, 255, 255, 0.16);
  border-radius: 16px;
  padding: 14px 18px;
  box-shadow: 0 20px 48px rgba(0, 0, 0, 0.38);
  z-index: 6;
}

.ui-overlay-light {
  position: absolute;
  background: rgba(255, 255, 255, 0.96);
  color: #18181B;
  border: 1.5px solid rgba(24, 24, 27, 0.12);
  border-radius: 16px;
  padding: 14px 18px;
  box-shadow: 0 20px 48px rgba(24, 24, 27, 0.22);
  z-index: 6;
}

.liquid-bar {
  position: relative;
  display: flex;
  align-items: center;
  background: #18181B;
  border: 1.5px solid rgba(255, 255, 255, 0.14);
  border-radius: 999px;
  padding: 4px;
  height: 46px;
  box-shadow: 0 14px 34px rgba(0, 0, 0, 0.32);
}

.liquid-indicator {
  position: absolute;
  top: 4px;
  height: 35px;
  border-radius: 999px;
  background: var(--orange);
  z-index: 0;
}

.liquid-tab {
  position: relative;
  z-index: 1;
  flex: 1;
  text-align: center;
  font-size: 13px;
  font-weight: 600;
  color: #FFFFFF;
  padding: 0 12px;
  white-space: nowrap;
}

.focus-ring-overlay {
  position: absolute;
  border: 2.5px solid var(--blue);
  border-radius: 12px;
  box-shadow: 0 0 0 4px rgba(13, 153, 255, 0.20);
  pointer-events: none;
  z-index: 5;
}

.focus-ring-tag {
  position: absolute;
  top: -28px;
  left: -2px;
  background: var(--blue);
  color: #FFFFFF;
  font-family: var(--font-mono);
  font-size: 11.5px;
  font-weight: 700;
  padding: 3px 9px;
  border-radius: 5px;
  white-space: nowrap;
}

/* Selection box around #shape */
#selection-ring {
  position: absolute;
  left: 720px;
  top: 720px;
  border: 2px solid var(--blue);
  pointer-events: none;
  opacity: 0;
  z-index: 12;
}
.corner-handle {
  position: absolute;
  width: 10px;
  height: 10px;
  background: #FFFFFF;
  border: 2px solid var(--blue);
  border-radius: 2px;
}
.corner-handle.tl { left: -6px; top: -6px; }
.corner-handle.tr { right: -6px; top: -6px; }
.corner-handle.bl { left: -6px; bottom: -6px; }
.corner-handle.br { right: -6px; bottom: -6px; }

#dim-badge {
  position: absolute;
  left: 50%;
  bottom: -34px;
  transform: translateX(-50%);
  background: var(--blue);
  color: #FFFFFF;
  font-family: var(--font-mono);
  font-size: 12px;
  font-weight: 700;
  padding: 3px 10px;
  border-radius: 5px;
  white-space: nowrap;
}

/* Cursor & Click Ripple */
#cursor-layer {
  position: absolute;
  inset: 0;
  pointer-events: none;
  z-index: 50;
}

#cursor {
  position: absolute;
  left: 0;
  top: 0;
  width: 34px;
  height: 34px;
  transform-origin: 4px 4px;
}

#cursor-key-pill {
  position: absolute;
  left: 28px;
  top: 22px;
  background: var(--ink);
  color: #FFFFFF;
  border: 1.5px solid rgba(255, 255, 255, 0.24);
  font-family: var(--font-mono);
  font-size: 13px;
  font-weight: 700;
  padding: 5px 12px;
  border-radius: 8px;
  white-space: nowrap;
  box-shadow: 0 10px 24px rgba(0, 0, 0, 0.28);
  opacity: 0;
}

#click-ripple {
  position: absolute;
  width: 58px;
  height: 58px;
  margin-left: -29px;
  margin-top: -29px;
  border-radius: 50%;
  border: 2.5px solid var(--orange);
  opacity: 0;
  pointer-events: none;
}

#hud-bottom {
  position: absolute;
  bottom: 38px;
  left: 48px;
  right: 48px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  pointer-events: none;
  z-index: 30;
}

.state-caption-pill {
  display: inline-flex;
  align-items: center;
  gap: 12px;
  padding: 11px 20px;
  border-radius: 999px;
  background: rgba(255, 255, 255, 0.94);
  border: 1px solid rgba(24, 24, 27, 0.10);
  box-shadow: 0 8px 24px rgba(24, 24, 27, 0.08);
  font-size: 15.5px;
  font-weight: 600;
  color: var(--ink);
}

.state-step-tag {
  font-family: var(--font-mono);
  font-size: 12.5px;
  font-weight: 700;
  padding: 4px 10px;
  border-radius: 999px;
  background: var(--orange);
  color: #FFFFFF;
}
`;

const SHARED_JS_ENGINE = `
const BPM = 100.60;
const TOTAL_BEATS = 28;
const DURATION = 16.70;
const BEAT = DURATION / TOTAL_BEATS;

const M = {
  BEAT,
  DURATION,
  TOTAL_BEATS,
  MORPH: [16, 0.82],
  SNAP: [26, 0.86],
  SOFT: [12, 0.90],
  LIQUID_LEAD: [29, 0.78],
  LIQUID_TRAIL: [14.5, 0.88],

  S(tau, w = 16, z = 0.82) {
    if (tau <= 0) return 0;
    if (tau >= 3.5) return 1;
    if (z < 1) {
      const wd = w * Math.sqrt(1 - z * z);
      return 1 - Math.exp(-z * w * tau) * (Math.cos(wd * tau) + ((z * w) / wd) * Math.sin(wd * tau));
    }
    return 1 - (1 + w * tau) * Math.exp(-w * tau);
  },

  track(b, initVal, steps) {
    let val = initVal;
    let prev = initVal;
    for (let i = 0; i < steps.length; i++) {
      const [b0, target, sp] = steps[i];
      const [w, z] = sp || M.MORPH;
      const tau = (b - b0) * BEAT;
      if (tau > 0) {
        val += (target - prev) * M.S(tau, w, z);
      }
      prev = target;
    }
    return val;
  },

  hexToRgb(hex) {
    const h = hex.replace('#', '');
    return [
      parseInt(h.slice(0, 2), 16),
      parseInt(h.slice(2, 4), 16),
      parseInt(h.slice(4, 6), 16),
    ];
  },

  ctrack(b, initHex, steps) {
    let [r, g, bl] = M.hexToRgb(initHex);
    let [pr, pg, pb] = [r, g, bl];
    for (let i = 0; i < steps.length; i++) {
      const [b0, targetHex, sp] = steps[i];
      const [tr, tg, tb] = M.hexToRgb(targetHex);
      const [w, z] = sp || M.SNAP;
      const tau = (b - b0) * BEAT;
      if (tau > 0) {
        const s = Math.max(0, Math.min(1, M.S(tau, w, z)));
        r += (tr - pr) * s;
        g += (tg - pg) * s;
        bl += (tb - pb) * s;
      }
      [pr, pg, pb] = [tr, tg, tb];
    }
    const clamp = (v) => Math.max(0, Math.min(255, Math.round(v)));
    return \`rgb(\${clamp(r)}, \${clamp(g)}, \${clamp(bl)})\`;
  },

  liquidEdges(b, initLeft, initRight, steps) {
    let left = initLeft;
    let right = initRight;
    let pLeft = initLeft;
    let pRight = initRight;
    for (let i = 0; i < steps.length; i++) {
      const [b0, tLeft, tRight] = steps[i];
      const tau = (b - b0) * BEAT;
      if (tau > 0) {
        const movingRight = tLeft >= pLeft;
        const [wL, zL] = movingRight ? M.LIQUID_TRAIL : M.LIQUID_LEAD;
        const [wR, zR] = movingRight ? M.LIQUID_LEAD : M.LIQUID_TRAIL;
        left += (tLeft - pLeft) * M.S(tau, wL, zL);
        right += (tRight - pRight) * M.S(tau, wR, zR);
      }
      pLeft = tLeft;
      pRight = tRight;
    }
    return { left, width: Math.max(12, right - left) };
  },

  vis(b, bStart, bEnd, isLoopEdge = false) {
    if (isLoopEdge) {
      if (b <= bEnd) {
        const outP = Math.max(0, Math.min(1, (b - (bEnd - 0.16)) / 0.16));
        return { opacity: 1 - outP, blur: outP * 6, scale: 1 - outP * 0.03 };
      }
      if (b >= 27.0) {
        const inP = Math.max(0, Math.min(1, (b - 27.0) / 0.18));
        return { opacity: inP, blur: (1 - inP) * 6, scale: 0.97 + inP * 0.03 };
      }
      return { opacity: 0, blur: 6, scale: 0.96 };
    }
    const enterStart = bStart;
    const enterEnd = bStart + 0.18;
    const exitStart = bEnd - 0.16;
    const exitEnd = bEnd + 0.02;
    if (b < enterStart || b > exitEnd) {
      return { opacity: 0, blur: 6, scale: 0.96 };
    }
    if (b < enterEnd) {
      const p = (b - enterStart) / (enterEnd - enterStart);
      const s = p * p * (3 - 2 * p);
      return { opacity: s, blur: (1 - s) * 5, scale: 0.985 + s * 0.015 };
    }
    if (b > exitStart) {
      const p = (b - exitStart) / (exitEnd - exitStart);
      const s = p * p * (3 - 2 * p);
      return { opacity: 1 - s, blur: s * 5, scale: 1 - s * 0.015 };
    }
    return { opacity: 1, blur: 0, scale: 1 };
  },

  applyVis(el, v) {
    if (!el) return;
    if (v.opacity <= 0.005) {
      el.style.opacity = '0';
      el.style.visibility = 'hidden';
    } else {
      el.style.visibility = 'visible';
      el.style.opacity = v.opacity.toFixed(3);
      const blurPx = v.blur > 0.15 ? \`blur(\${v.blur.toFixed(2)}px)\` : 'none';
      el.style.filter = blurPx;
      el.style.transform = \`scale(\${v.scale.toFixed(4)})\`;
    }
  },

  clickEffect(b, clickBeats) {
    let scale = 1.0;
    let rippleOp = 0;
    let rippleScale = 0.3;
    for (const cb of clickBeats) {
      const db = b - cb;
      if (db >= -0.04 && db <= 0.42) {
        if (db >= 0 && db <= 0.18) {
          scale = 1.0 - 0.14 * Math.sin((db / 0.18) * Math.PI);
        }
        if (db >= 0) {
          const p = db / 0.42;
          rippleOp = Math.max(0, (1 - p) * 0.85);
          rippleScale = 0.35 + p * 1.35;
        }
      }
    }
    return { scale, rippleOp, rippleScale };
  }
};
`;

const CURSOR_SVG = `
<div id="cursor-layer">
  <div id="click-ripple"></div>
  <div id="cursor">
    <svg width="34" height="34" viewBox="0 0 34 34" fill="none" xmlns="http://www.w3.org/2000/svg" style="filter: drop-shadow(0 6px 12px rgba(0,0,0,0.28));">
      <path d="M5.5 4.2L27.5 14.6C28.9 15.25 28.7 17.3 27.2 17.65L17.6 19.9L13.5 28.9C12.85 30.3 10.8 30.1 10.45 28.6L4.1 5.8C3.75 4.5 4.35 3.65 5.5 4.2Z" fill="#18181B" stroke="#FFFFFF" stroke-width="2.4" stroke-linejoin="round"/>
    </svg>
    <div id="cursor-key-pill"></div>
  </div>
</div>
`;

function studioHeader(title, badgeText, badgeColor = 'var(--orange)') {
  return `
    <div class="studio-window-header">
      <div style="display:flex; align-items:center; gap:14px;">
        <div class="traffic-dots">
          <span class="dot-r"></span><span class="dot-y"></span><span class="dot-g"></span>
        </div>
        <div class="studio-window-title">
          <span>${title}</span>
        </div>
      </div>
      <span class="studio-window-badge" style="background:${badgeColor};">${badgeText}</span>
    </div>
  `;
}

// ============================================================================
// VIDEO 01: 01-designos-studio-launch-hero.html
// Uses real assets: 01-hero-canvas, 04-cmdk-ai-synthesizer, 02-agents-chat,
// 03-spec-reader, 06-variables-tokens, 07-components-variants, 05-dev-mode-codegen
// ============================================================================
function buildVideo01Html() {
  return `<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8" />
<title>Design OS Studio — Reel 01: Launch Hero (Real Studio Assets + UI Motion)</title>
<style>
${SHARED_CSS}
</style>
</head>
<body>
<div id="stage">
  <div id="hud-top">
    <div class="brand-pill">
      <span class="brand-dot"></span>
      <span>design:os studio · Native Rust + wgpu · Real Studio Capture</span>
    </div>
    <div class="beat-pill" id="beat-readout">BAR 1.1 · 100.6 BPM</div>
  </div>

  <div id="camera">
    <div id="shape">
      <!-- State 1 & Loop (B00..B02 & B26.85..B28): Compact Launch Button -->
      <div class="state-panel" id="st-1" style="flex-direction:row; align-items:center; justify-content:center; gap:14px; color:#FFF;">
        <span style="width:36px; height:36px; border-radius:50%; background:var(--orange); display:inline-flex; align-items:center; justify-content:center; font-size:22px; font-weight:700;">+</span>
        <span style="font-size:22px; font-weight:600; letter-spacing:-0.02em;">New Design · design:os</span>
      </div>

      <!-- State 2 (B02..B05.5): Real 01-hero-canvas.png + Floating Liquid Tool Bar -->
      <div class="state-panel" id="st-2">
        ${studioHeader('design:os studio — Home · Fintech Mobile Frame', '60 FPS WGPU · RETINA 2X', 'var(--blue)')}
        <div class="studio-viewport">
          <img id="v1-img-2" class="studio-screenshot" src="../screenshots/01-hero-canvas.png" alt="Hero Canvas" />
          <div class="focus-ring-overlay" style="left:210px; top:86px; width:640px; height:510px;">
            <span class="focus-ring-tag">Frame "Home" · 140% Zoom · Rulers Active</span>
          </div>
          <!-- Floating Liquid Tool Switcher -->
          <div style="position:absolute; bottom:20px; left:50%; transform:translateX(-50%); width:580px; z-index:8;">
            <div class="liquid-bar">
              <div class="liquid-indicator" id="v1-tool-hl" style="background:var(--blue);"></div>
              <div class="liquid-tab">↖ Move V</div>
              <div class="liquid-tab"># Frame F</div>
              <div class="liquid-tab">✎ Pen P</div>
              <div class="liquid-tab">✦ AI ⌘K</div>
              <div class="liquid-tab">&lt;/&gt; Dev Mode</div>
            </div>
          </div>
        </div>
      </div>

      <!-- State 3 (B05.5..B09): Real 04-cmdk-ai-synthesizer.png + Interactive Prompt Pill -->
      <div class="state-panel" id="st-3">
        ${studioHeader('design:os studio — ⌘K AI Design Synthesizer', 'PROMPT → NODESPEC JSON', 'var(--orange)')}
        <div class="studio-viewport">
          <img id="v1-img-3" class="studio-screenshot" src="../screenshots/04-cmdk-ai-synthesizer.png" alt="Cmd+K AI Synthesizer" />
          <div class="ui-overlay-card" style="left:50%; bottom:24px; transform:translateX(-50%); width:780px; display:flex; align-items:center; justify-content:space-between;">
            <div style="display:flex; align-items:center; gap:12px;">
              <span style="background:var(--orange); color:#FFF; font-family:var(--font-mono); font-size:13px; font-weight:700; padding:5px 10px; border-radius:7px;">⌘K</span>
              <span id="v1-cmdk-typed" style="font-size:18px; font-weight:600; color:#FFF;"></span>
              <span style="width:2.5px; height:22px; background:var(--orange); display:inline-block;"></span>
            </div>
            <span style="background:var(--blue); color:#FFF; font-family:var(--font-mono); font-size:12.5px; font-weight:700; padding:5px 12px; border-radius:7px;">ENTER ↵</span>
          </div>
        </div>
      </div>

      <!-- State 4 (B09..B13): Real 02-agents-chat.png + Liquid Multi-Agent PTY Switcher -->
      <div class="state-panel" id="st-4">
        ${studioHeader('design:os studio — Multi-Agent Native PTY Bridge', '5 CLI RUNTIMES LIVE', 'var(--orange)')}
        <div class="studio-viewport">
          <img id="v1-img-4" class="studio-screenshot" src="../screenshots/02-agents-chat.png" alt="Multi-Agent Chat" />
          <div style="position:absolute; top:18px; left:24px; width:460px; z-index:8;">
            <div class="liquid-bar">
              <div class="liquid-indicator" id="v1-agent-hl" style="background:var(--orange);"></div>
              <div class="liquid-tab">Claude Code</div>
              <div class="liquid-tab">Codex</div>
              <div class="liquid-tab">Antigravity</div>
            </div>
          </div>
          <div class="ui-overlay-card" style="right:24px; bottom:22px; width:490px;">
            <div style="font-family:var(--font-mono); font-size:12px; color:var(--emerald); font-weight:700;">LIVE PTY STREAM · ZERO-FAKE-UI</div>
            <div style="font-size:16px; font-weight:700; margin-top:4px;">✓ Injected KPI Card onto wgpu Canvas (0.4ms)</div>
          </div>
        </div>
      </div>

      <!-- State 5 (B13..B16.5): Real 03-spec-reader.png + Spec Contract Callout -->
      <div class="state-panel" id="st-5">
        ${studioHeader('design:os studio — PRODUCT:OS v2 Interactive Spec Reader', '12/12 NODES LINKED', 'var(--emerald)')}
        <div class="studio-viewport">
          <img id="v1-img-5" class="studio-screenshot" src="../screenshots/03-spec-reader.png" alt="Spec Reader" />
          <div class="ui-overlay-light" style="right:24px; bottom:24px; width:500px; display:flex; align-items:center; justify-content:space-between;">
            <div>
              <div style="font-family:var(--font-mono); font-size:12px; color:var(--orange); font-weight:700;">DIR-28 · SPEC-TO-CANVAS LINK</div>
              <div style="font-size:16px; font-weight:700; margin-top:3px;">Click any Spec badge → Selects &amp; Zooms Canvas Node</div>
            </div>
            <span style="background:var(--emerald); color:#FFF; font-family:var(--font-mono); font-size:12.5px; font-weight:700; padding:6px 12px; border-radius:999px;">PASS ✓</span>
          </div>
        </div>
      </div>

      <!-- State 6 (B16.5..B19.5): Real 06-variables-tokens.png + Self-Evolution 9/9 ALIVE Gauge -->
      <div class="state-panel" id="st-6">
        ${studioHeader('design:os studio — Design System Variables & Self-Evolution Gate', '9/9 GATES ALIVE', 'var(--emerald)')}
        <div class="studio-viewport">
          <img id="v1-img-6" class="studio-screenshot" src="../screenshots/06-variables-tokens.png" alt="Variables & Tokens" />
          <div class="ui-overlay-card" style="left:24px; bottom:24px; display:flex; align-items:center; gap:18px; padding:16px 22px;">
            <svg width="68" height="68" viewBox="0 0 68 68">
              <circle cx="34" cy="34" r="27" stroke="#27272A" stroke-width="7" fill="none"/>
              <circle id="v1-gate-arc" cx="34" cy="34" r="27" stroke="#14AE5C" stroke-width="7" fill="none" stroke-linecap="round" transform="rotate(-90 34 34)" stroke-dasharray="169.6" stroke-dashoffset="0"/>
              <text x="34" y="39" text-anchor="middle" fill="#FFFFFF" font-family="Geist Mono" font-size="14" font-weight="700">9/9</text>
            </svg>
            <div>
              <div style="font-family:var(--font-mono); font-size:12px; color:var(--emerald); font-weight:700;">SELF-EVOLUTION ENGINE · ALIVE</div>
              <div style="font-size:17px; font-weight:700; margin-top:2px;">15/15 Token Parity · 856/856 E2E Tests PASS</div>
            </div>
          </div>
        </div>
      </div>

      <!-- State 7 (B19.5..B22.5): Real 07-components-variants.png + Variant Matrix Overlay -->
      <div class="state-panel" id="st-7">
        ${studioHeader('design:os studio — Component Sets, Variants & Instance Swap', 'PURPLE DASHED SET', 'var(--blue)')}
        <div class="studio-viewport">
          <img id="v1-img-7" class="studio-screenshot" src="../screenshots/07-components-variants.png" alt="Components & Variants" />
          <div class="ui-overlay-card" style="right:24px; bottom:24px; display:flex; align-items:center; gap:14px;">
            <span style="background:var(--blue); color:#FFF; font-family:var(--font-mono); font-size:12.5px; font-weight:700; padding:5px 11px; border-radius:7px;">◇ COMPONENT SET</span>
            <span style="font-size:15.5px; font-weight:600;">Primary / Secondary / Ghost Variants + Live Override</span>
          </div>
        </div>
      </div>

      <!-- State 8 (B22.5..B25.3): Real 05-dev-mode-codegen.png + Liquid Codegen Switcher -->
      <div class="state-panel" id="st-8">
        ${studioHeader('design:os studio — Dev Mode Multi-Target Codegen', 'READY FOR DEV', 'var(--emerald)')}
        <div class="studio-viewport">
          <img id="v1-img-8" class="studio-screenshot" src="../screenshots/05-dev-mode-codegen.png" alt="Dev Mode Codegen" />
          <div style="position:absolute; bottom:22px; left:50%; transform:translateX(-50%); width:520px; z-index:8;">
            <div class="liquid-bar">
              <div class="liquid-indicator" id="v1-dev-hl" style="background:var(--blue);"></div>
              <div class="liquid-tab">React TSX</div>
              <div class="liquid-tab">Tailwind CSS</div>
              <div class="liquid-tab">SwiftUI</div>
              <div class="liquid-tab">Rust egui</div>
            </div>
          </div>
        </div>
      </div>

      <!-- State 9 (B25.3..B26.85): Compact Marlow .fig Export Toast -->
      <div class="state-panel" id="st-9" style="flex-direction:row; align-items:center; justify-content:center; gap:16px; color:#FFF;">
        <span style="width:36px; height:36px; border-radius:50%; background:var(--emerald); display:inline-flex; align-items:center; justify-content:center; font-size:19px; font-weight:700;">✓</span>
        <span style="font-size:20px; font-weight:600;">Exported Marlow .fig · 100/100 Figma Parity</span>
      </div>
    </div>
  </div>

  ${CURSOR_SVG}

  <div id="hud-bottom">
    <div class="state-caption-pill">
      <span class="state-step-tag" id="step-tag">STATE 01 / 09</span>
      <span id="step-label">One-Shape Morph · + New Design Button</span>
    </div>
    <div class="brand-pill" style="font-family:var(--font-mono); font-size:13.5px;">
      <span>REAL STUDIO ASSETS · 60 FPS · 4× TMIX</span>
    </div>
  </div>
</div>

<script>
${SHARED_JS_ENGINE}

const shapeEl = document.getElementById('shape');
const cameraEl = document.getElementById('camera');
const cursorEl = document.getElementById('cursor');
const keyPillEl = document.getElementById('cursor-key-pill');
const rippleEl = document.getElementById('click-ripple');
const beatReadoutEl = document.getElementById('beat-readout');
const stepTagEl = document.getElementById('step-tag');
const stepLabelEl = document.getElementById('step-label');

const panels = Array.from({ length: 9 }, (_, i) => document.getElementById('st-' + (i + 1)));

function applyImgPan(id, b, b0, s0, s1, tx0, tx1, ty0, ty1) {
  const el = document.getElementById(id);
  if (!el) return;
  const p = M.S((b - b0) * BEAT, 8, 0.92);
  const sc = s0 + (s1 - s0) * p;
  const tx = tx0 + (tx1 - tx0) * p;
  const ty = ty0 + (ty1 - ty0) * p;
  el.style.transform = \`scale(\${sc.toFixed(4)}) translate(\${tx.toFixed(1)}px, \${ty.toFixed(1)}px)\`;
}

window.seek = function(rawT) {
  const t = (rawT >= DURATION - 1e-4 || rawT <= 1e-4) ? 0 : ((rawT % DURATION) + DURATION) % DURATION;
  const b = t / BEAT;

  const bar = Math.min(7, Math.floor(b / 4) + 1);
  const beatInBar = (Math.floor(b) % 4) + 1;
  beatReadoutEl.textContent = \`BAR \${bar}.\${beatInBar} · BEAT \${Math.floor(b) + 1}/28\`;

  // #shape size morph: starts as 340x76 pill -> expands to 1180x760 Studio Window -> collapses back
  const w = M.track(b, 340, [
    [2.0, 1180, M.MORPH],
    [25.0, 540, M.SNAP],
    [27.0, 340, M.MORPH],
  ]);
  const h = M.track(b, 76, [
    [2.0, 760, M.MORPH],
    [25.0, 82, M.SNAP],
    [27.0, 76, M.MORPH],
  ]);
  const rad = M.track(b, 38, [
    [2.0, 20, M.MORPH],
    [25.0, 41, M.SNAP],
    [27.0, 38, M.MORPH],
  ]);

  shapeEl.style.width = \`\${w.toFixed(2)}px\`;
  shapeEl.style.height = \`\${h.toFixed(2)}px\`;
  shapeEl.style.marginLeft = \`\${(-w / 2).toFixed(2)}px\`;
  shapeEl.style.marginTop = \`\${(-h / 2).toFixed(2)}px\`;
  shapeEl.style.borderRadius = \`\${rad.toFixed(2)}px\`;

  const camScale = M.track(b, 1.25, [
    [2.0, 1.04, M.MORPH],
    [5.0, 1.06, M.MORPH],
    [9.0, 1.05, M.MORPH],
    [13.0, 1.04, M.MORPH],
    [16.0, 1.06, M.MORPH],
    [19.0, 1.05, M.MORPH],
    [22.0, 1.06, M.MORPH],
    [25.0, 1.22, M.MORPH],
    [27.0, 1.25, M.MORPH],
  ]);
  cameraEl.style.transform = \`scale(\${camScale.toFixed(4)})\`;

  M.applyVis(panels[0], M.vis(b, 0, 2.0, true));
  M.applyVis(panels[1], M.vis(b, 2.0, 5.0));
  M.applyVis(panels[2], M.vis(b, 5.0, 9.0));
  M.applyVis(panels[3], M.vis(b, 9.0, 13.0));
  M.applyVis(panels[4], M.vis(b, 13.0, 16.0));
  M.applyVis(panels[5], M.vis(b, 16.0, 19.0));
  M.applyVis(panels[6], M.vis(b, 19.0, 22.0));
  M.applyVis(panels[7], M.vis(b, 22.0, 25.0));
  M.applyVis(panels[8], M.vis(b, 25.0, 27.0));

  // Subtle spring-driven focal zoom/pan on each real Retina 2x screenshot
  applyImgPan('v1-img-2', b, 2.2, 1.0, 1.12, 0, -18, 0, -8);
  applyImgPan('v1-img-3', b, 5.7, 1.02, 1.16, 0, 0, 0, 14);
  applyImgPan('v1-img-4', b, 9.2, 1.0, 1.14, 0, -28, 0, 10);
  applyImgPan('v1-img-5', b, 13.2, 1.0, 1.12, 0, 22, 0, -6);
  applyImgPan('v1-img-6', b, 16.7, 1.0, 1.15, 0, -24, 0, -10);
  applyImgPan('v1-img-7', b, 19.7, 1.0, 1.14, 0, 12, 0, -8);
  applyImgPan('v1-img-8', b, 22.7, 1.0, 1.16, 0, -32, 0, 8);

  // Liquid tool indicator in State 2
  const toolHl = M.liquidEdges(b, 4, 116, [
    [3.0, 118, 232],
    [4.2, 346, 460],
  ]);
  const toolHlEl = document.getElementById('v1-tool-hl');
  toolHlEl.style.left = \`\${toolHl.left.toFixed(1)}px\`;
  toolHlEl.style.width = \`\${toolHl.width.toFixed(1)}px\`;

  // State 3: Cmd+K prompt typing
  const promptStr = 'Synthesize fintech KPI card + bind design tokens';
  const typeP = Math.max(0, Math.min(1, (b - 5.9) / 2.2));
  document.getElementById('v1-cmdk-typed').textContent = promptStr.slice(0, Math.floor(typeP * promptStr.length));

  // State 4: Liquid Multi-Agent PTY indicator
  const agentHl = M.liquidEdges(b, 4, 152, [
    [10.0, 154, 302],
    [11.2, 304, 452],
  ]);
  const agentHlEl = document.getElementById('v1-agent-hl');
  agentHlEl.style.left = \`\${agentHl.left.toFixed(1)}px\`;
  agentHlEl.style.width = \`\${agentHl.width.toFixed(1)}px\`;

  // State 8: Liquid Dev Mode Codegen indicator
  const devHl = M.liquidEdges(b, 4, 130, [
    [23.2, 132, 258],
    [24.2, 260, 386],
  ]);
  const devHlEl = document.getElementById('v1-dev-hl');
  devHlEl.style.left = \`\${devHl.left.toFixed(1)}px\`;
  devHlEl.style.width = \`\${devHl.width.toFixed(1)}px\`;

  // Cursor trajectory
  const cx = M.track(b, 720, [
    [1.0, 720, M.MORPH],
    [3.0, 610, M.MORPH],
    [4.2, 830, M.MORPH],
    [6.5, 720, M.MORPH],
    [10.0, 390, M.MORPH],
    [11.2, 530, M.MORPH],
    [14.2, 920, M.MORPH],
    [17.5, 420, M.MORPH],
    [20.5, 860, M.MORPH],
    [23.2, 660, M.MORPH],
    [24.2, 790, M.MORPH],
    [26.5, 720, M.MORPH],
  ]);
  const cy = M.track(b, 720, [
    [1.0, 720, M.MORPH],
    [3.0, 1045, M.MORPH],
    [4.2, 1045, M.MORPH],
    [6.5, 1030, M.MORPH],
    [10.0, 410, M.MORPH],
    [11.2, 410, M.MORPH],
    [14.2, 1020, M.MORPH],
    [17.5, 1020, M.MORPH],
    [20.5, 1030, M.MORPH],
    [23.2, 1045, M.MORPH],
    [24.2, 1045, M.MORPH],
    [26.5, 720, M.MORPH],
  ]);

  const clicks = [1.0, 3.0, 4.2, 8.2, 10.0, 11.2, 14.2, 17.5, 20.5, 23.2, 24.2, 25.3];
  const ce = M.clickEffect(b, clicks);
  cursorEl.style.transform = \`translate(\${cx.toFixed(1)}px, \${cy.toFixed(1)}px) scale(\${ce.scale.toFixed(3)})\`;

  if (ce.rippleOp > 0.01) {
    rippleEl.style.opacity = ce.rippleOp.toFixed(3);
    rippleEl.style.transform = \`translate(\${cx.toFixed(1)}px, \${cy.toFixed(1)}px) scale(\${ce.rippleScale.toFixed(3)})\`;
  } else {
    rippleEl.style.opacity = '0';
  }

  let keyText = '';
  if (b >= 4.0 && b < 8.8) keyText = '⌘K · AI Synthesizer';
  else if (b >= 9.0 && b < 12.8) keyText = 'Native PTY Agent';
  else if (b >= 22.5 && b < 25.1) keyText = 'Dev Mode · Codegen';
  if (keyText) {
    keyPillEl.textContent = keyText;
    keyPillEl.style.opacity = '1';
  } else {
    keyPillEl.style.opacity = '0';
  }

  const captions = [
    [2.0, 'STATE 01 / 09', 'One-Shape Morph · + New Design Button'],
    [5.5, 'STATE 02 / 09', 'Real Studio Capture · 60fps wgpu Canvas & Liquid Toolbar'],
    [9.0, 'STATE 03 / 09', '⌘K AI Synthesizer · Prompt to NodeSpec Generation'],
    [13.0, 'STATE 04 / 09', 'Multi-Agent Native PTY Chat · Claude, Codex & Antigravity'],
    [16.5, 'STATE 05 / 09', 'PRODUCT:OS v2 Spec Reader · 12/12 Linked Canvas Nodes'],
    [19.5, 'STATE 06 / 09', 'Variables & Self-Evolution Gate · 9/9 ALIVE & 15/15 Tokens'],
    [22.5, 'STATE 07 / 09', 'Component Sets, Variants & Instance Overrides'],
    [25.3, 'STATE 08 / 09', 'Dev Mode Multi-Target Codegen · React, Tailwind & SwiftUI'],
    [26.85, 'STATE 09 / 09', 'Native .fig Binary Export · 100/100 Figma Parity'],
    [28.1, 'STATE 01 / 09', 'One-Shape Morph · + New Design Button'],
  ];
  for (const [endB, tag, lbl] of captions) {
    if (b <= endB) {
      stepTagEl.textContent = tag;
      stepLabelEl.textContent = lbl;
      break;
    }
  }
};

window.__readyPromise = Promise.all([
  document.fonts.ready,
  ...Array.from(document.images).map((img) => img.decode().catch(() => {})),
]).then(() => {
  window.seek(0);
  return true;
});
</script>
</body>
</html>`;
}

// ============================================================================
// VIDEO 02: 02-figma-100-micro-interactions.html
// Uses all 8 REAL Micro-Interaction Studio Screenshots (09..16):
// 09-micro-autolayout, 10-micro-css-grid, 11-micro-smart-selection,
// 12-micro-shape-controls, 13-micro-vector-edit, 14-micro-layer-picker,
// 15-micro-dev-boxmodel, 16-micro-scale-tool
// ============================================================================
function buildVideo02Html() {
  return `<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8" />
<title>Design OS Studio — Reel 02: 100/100 Figma Micro-Interactions (Real Studio Assets)</title>
<style>
${SHARED_CSS}
</style>
</head>
<body>
<div id="stage">
  <div id="hud-top">
    <div class="brand-pill">
      <span class="brand-dot" style="background:var(--blue);"></span>
      <span>design:os studio · 100/100 Figma A-to-Z Micro-Interactions · Real Studio Assets</span>
    </div>
    <div class="beat-pill" id="beat-readout">BAR 1.1 · 100.6 BPM</div>
  </div>

  <div id="camera">
    <div id="shape">
      <!-- State 1 & Loop (B00..B02.2 & B26.85..B28): Inspector Numeric Scrub Pill -->
      <div class="state-panel" id="st-1" style="flex-direction:row; align-items:center; justify-content:space-between; color:#FFF; padding:0 28px;">
        <div style="display:flex; align-items:center; gap:14px;">
          <span style="color:var(--orange); font-family:var(--font-mono); font-size:20px; font-weight:700;">↔ W</span>
          <span id="v2-scrub-val" style="font-family:var(--font-mono); font-size:26px; font-weight:700;">280 px</span>
        </div>
        <span style="background:#27272A; color:#A1A1AA; font-family:var(--font-mono); font-size:13px; padding:5px 10px; border-radius:7px;">⇧ ×10 SCRUB</span>
      </div>

      <!-- State 2 (B02.2..B05.4): Real 09-micro-autolayout.png + Pink Gap Scrubber & Hug/Fill Pill -->
      <div class="state-panel" id="st-2">
        ${studioHeader('Showcase 01/08 — Auto-Layout 5.0 · On-Canvas Pink Gap & Padding Handles', 'HUG / FILL CYCLE', 'var(--pink)')}
        <div class="studio-viewport">
          <img id="v2-img-2" class="studio-screenshot" src="../screenshots/09-micro-autolayout.png" alt="Auto-Layout 5.0" />
          <div class="ui-overlay-card" style="left:24px; bottom:22px; display:flex; align-items:center; gap:14px;">
            <span style="background:var(--pink); color:#FFF; font-family:var(--font-mono); font-size:13px; font-weight:700; padding:5px 12px; border-radius:7px;" id="v2-gap-pill">GAP: 20px</span>
            <span style="font-size:15px; font-weight:600;" id="v2-edge-pill">Double-click Edge → Hug Contents</span>
          </div>
        </div>
      </div>

      <!-- State 3 (B05.4..B08.6): Real 10-micro-css-grid.png + Clickable Track Pill Overlay -->
      <div class="state-panel" id="st-3">
        ${studioHeader('Showcase 02/08 — 2D CSS Grid · Track Sizing Pills (1fr / px / Hug) & 2×1 Span', 'CSS GRID 2D', 'var(--blue)')}
        <div class="studio-viewport">
          <img id="v2-img-3" class="studio-screenshot" src="../screenshots/10-micro-css-grid.png" alt="2D CSS Grid" />
          <div class="ui-overlay-card" style="right:24px; bottom:22px; display:flex; align-items:center; gap:14px;">
            <span style="background:var(--blue); color:#FFF; font-family:var(--font-mono); font-size:13px; font-weight:700; padding:5px 12px; border-radius:7px;" id="v2-grid-pill">COL 1 · 1fr</span>
            <span style="font-size:15px; font-weight:600;">Click Track Pill to cycle 1fr → Fixed px → Hug</span>
          </div>
        </div>
      </div>

      <!-- State 4 (B08.6..B11.8): Real 11-micro-smart-selection.png + Pink Swap & Tidy Up -->
      <div class="state-panel" id="st-4">
        ${studioHeader('Showcase 03/08 — Smart Selection · Pink Swap Handles & One-Click Tidy Up', 'TIDY UP 18PX', 'var(--pink)')}
        <div class="studio-viewport">
          <img id="v2-img-4" class="studio-screenshot" src="../screenshots/11-micro-smart-selection.png" alt="Smart Selection" />
          <div class="ui-overlay-card" style="left:24px; bottom:22px; display:flex; align-items:center; gap:14px;">
            <span style="background:var(--pink); color:#FFF; font-family:var(--font-mono); font-size:13px; font-weight:700; padding:5px 12px; border-radius:7px;">● PINK RING SWAP</span>
            <span style="font-size:15px; font-weight:600;">Equalizes 18px spacing &amp; swaps sibling order</span>
          </div>
        </div>
      </div>

      <!-- State 5 (B11.8..B15.0): Real 12-micro-shape-controls.png + Arc Sweep & Star Ratio -->
      <div class="state-panel" id="st-5">
        ${studioHeader('Showcase 04/08 — Parametric On-Canvas Shape Controls (Donut Arc & Star)', 'SWEEP 270° · RATIO 62%', 'var(--orange)')}
        <div class="studio-viewport">
          <img id="v2-img-5" class="studio-screenshot" src="../screenshots/12-micro-shape-controls.png" alt="Parametric Shape Controls" />
          <div class="ui-overlay-card" style="right:24px; bottom:22px; display:flex; align-items:center; gap:14px;">
            <span style="background:var(--orange); color:#FFF; font-family:var(--font-mono); font-size:13px; font-weight:700; padding:5px 12px; border-radius:7px;" id="v2-arc-pill">SWEEP 270° · INNER 62%</span>
            <span style="font-size:15px; font-weight:600;">5-Point Star Ratio 42% · Polygon 6 Sides</span>
          </div>
        </div>
      </div>

      <!-- State 6 (B15.0..B18.0): Real 13-micro-vector-edit.png + Bezier Tangent & Heal Delete -->
      <div class="state-panel" id="st-6">
        ${studioHeader('Showcase 05/08 — Vector Network Editor · Bezier Tangents & Midpoint Split', '⇧⌫ HEAL DELETE', 'var(--blue)')}
        <div class="studio-viewport">
          <img id="v2-img-6" class="studio-screenshot" src="../screenshots/13-micro-vector-edit.png" alt="Vector Network Editor" />
          <div class="ui-overlay-card" style="left:24px; bottom:22px; display:flex; align-items:center; gap:14px;">
            <span style="background:var(--blue); color:#FFF; font-family:var(--font-mono); font-size:13px; font-weight:700; padding:5px 12px; border-radius:7px;">◆ MIDPOINT SPLIT</span>
            <span style="font-size:15px; font-weight:600;">Click midpoint to subdivide · ⌘-Drag breaks tangent</span>
          </div>
        </div>
      </div>

      <!-- State 7 (B18.0..B21.0): Real 14-micro-layer-picker.png + Eyedropper Format Cycle -->
      <div class="state-panel" id="st-7">
        ${studioHeader('Showcase 06/08 — Layer Search, Status Badge Cycle & Eyedropper Loupe (I)', 'TAB FORMAT CYCLE', 'var(--emerald)')}
        <div class="studio-viewport">
          <img id="v2-img-7" class="studio-screenshot" src="../screenshots/14-micro-layer-picker.png" alt="Layer Filter & Eyedropper" />
          <div class="ui-overlay-card" style="right:24px; bottom:22px; display:flex; align-items:center; gap:14px;">
            <span style="background:var(--emerald); color:#FFF; font-family:var(--font-mono); font-size:13px; font-weight:700; padding:5px 12px; border-radius:7px;" id="v2-eye-pill">#E36323 · HEX (TAB ⇥)</span>
            <span style="font-size:15px; font-weight:600;">Tab cycles HEX → RGB → HSL · Double-click icon frames</span>
          </div>
        </div>
      </div>

      <!-- State 8 (B21.0..B24.0): Real 15-micro-dev-boxmodel.png + Emerald Padding & Gap Callout -->
      <div class="state-panel" id="st-8">
        ${studioHeader('Showcase 07/08 — Dev Mode Box-Model Overlay · Emerald Padding & Orange Gap', 'DEV INSPECT', 'var(--emerald)')}
        <div class="studio-viewport">
          <img id="v2-img-8" class="studio-screenshot" src="../screenshots/15-micro-dev-boxmodel.png" alt="Dev Mode Box-Model" />
          <div class="ui-overlay-card" style="left:24px; bottom:22px; display:flex; align-items:center; gap:14px;">
            <span style="background:var(--emerald); color:#FFF; font-family:var(--font-mono); font-size:13px; font-weight:700; padding:5px 12px; border-radius:7px;">PADDING 24PX · GAP 16PX</span>
            <span style="font-size:15px; font-weight:600;">Live CSS / SwiftUI / Rust egui box-model overlay</span>
          </div>
        </div>
      </div>

      <!-- State 9 (B24.0..B26.85): Real 16-micro-scale-tool.png + Liquid Scale Multiplier Bar -->
      <div class="state-panel" id="st-9">
        ${studioHeader('Showcase 08/08 — Proportional Scale Tool (K) · Font, Stroke & Radius Scaling', 'SCALE 1.5X', 'var(--orange)')}
        <div class="studio-viewport">
          <img id="v2-img-9" class="studio-screenshot" src="../screenshots/16-micro-scale-tool.png" alt="Proportional Scale Tool" />
          <div style="position:absolute; bottom:22px; left:50%; transform:translateX(-50%); width:460px; z-index:8;">
            <div class="liquid-bar">
              <div class="liquid-indicator" id="v2-scale-hl" style="background:var(--orange);"></div>
              <div class="liquid-tab">1.0× Base</div>
              <div class="liquid-tab">1.5× Scaled</div>
              <div class="liquid-tab">2.0× Retina</div>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>

  ${CURSOR_SVG}

  <div id="hud-bottom">
    <div class="state-caption-pill">
      <span class="state-step-tag" id="step-tag" style="background:var(--blue);">MICRO 01 / 09</span>
      <span id="step-label">Numeric Scrubbing &amp; Math Input (⇧ Shift ×10)</span>
    </div>
    <div class="brand-pill" style="font-family:var(--font-mono); font-size:13.5px;">
      <span>100/100 FIGMA MICRO-INTERACTIONS · REAL STUDIO CAPTURE</span>
    </div>
  </div>
</div>

<script>
${SHARED_JS_ENGINE}

const shapeEl = document.getElementById('shape');
const cameraEl = document.getElementById('camera');
const cursorEl = document.getElementById('cursor');
const keyPillEl = document.getElementById('cursor-key-pill');
const rippleEl = document.getElementById('click-ripple');
const beatReadoutEl = document.getElementById('beat-readout');
const stepTagEl = document.getElementById('step-tag');
const stepLabelEl = document.getElementById('step-label');

const panels = Array.from({ length: 9 }, (_, i) => document.getElementById('st-' + (i + 1)));

function applyImgPan(id, b, b0, s0, s1, tx0, tx1, ty0, ty1) {
  const el = document.getElementById(id);
  if (!el) return;
  const p = M.S((b - b0) * BEAT, 8, 0.92);
  const sc = s0 + (s1 - s0) * p;
  const tx = tx0 + (tx1 - tx0) * p;
  const ty = ty0 + (ty1 - ty0) * p;
  el.style.transform = \`scale(\${sc.toFixed(4)}) translate(\${tx.toFixed(1)}px, \${ty.toFixed(1)}px)\`;
}

window.seek = function(rawT) {
  const t = (rawT >= DURATION - 1e-4 || rawT <= 1e-4) ? 0 : ((rawT % DURATION) + DURATION) % DURATION;
  const b = t / BEAT;

  const bar = Math.min(7, Math.floor(b / 4) + 1);
  const beatInBar = (Math.floor(b) % 4) + 1;
  beatReadoutEl.textContent = \`BAR \${bar}.\${beatInBar} · BEAT \${Math.floor(b) + 1}/28\`;

  let w = M.track(b, 340, [
    [2.0, 1180, M.MORPH],
    [27.0, 340, M.MORPH],
  ]);
  const h = M.track(b, 80, [
    [2.0, 760, M.MORPH],
    [27.0, 80, M.MORPH],
  ]);

  let scrubPx = 280;
  if (b >= 0.4 && b <= 1.8) {
    const p = Math.sin(Math.min(1, (b - 0.4) / 1.4) * Math.PI);
    scrubPx = Math.round(280 + p * 180);
    w += p * 110;
  }
  document.getElementById('v2-scrub-val').textContent = \`\${scrubPx} px\`;

  const rad = M.track(b, 40, [
    [2.0, 20, M.MORPH],
    [27.0, 40, M.MORPH],
  ]);

  shapeEl.style.width = \`\${w.toFixed(2)}px\`;
  shapeEl.style.height = \`\${h.toFixed(2)}px\`;
  shapeEl.style.marginLeft = \`\${(-w / 2).toFixed(2)}px\`;
  shapeEl.style.marginTop = \`\${(-h / 2).toFixed(2)}px\`;
  shapeEl.style.borderRadius = \`\${rad.toFixed(2)}px\`;

  const camScale = M.track(b, 1.26, [
    [2.0, 1.05, M.MORPH],
    [11.0, 1.06, M.MORPH],
    [21.0, 1.05, M.MORPH],
    [27.0, 1.26, M.MORPH],
  ]);
  cameraEl.style.transform = \`scale(\${camScale.toFixed(4)})\`;

  M.applyVis(panels[0], M.vis(b, 0, 2.0, true));
  M.applyVis(panels[1], M.vis(b, 2.0, 5.0));
  M.applyVis(panels[2], M.vis(b, 5.0, 8.0));
  M.applyVis(panels[3], M.vis(b, 8.0, 11.0));
  M.applyVis(panels[4], M.vis(b, 11.0, 14.0));
  M.applyVis(panels[5], M.vis(b, 14.0, 17.0));
  M.applyVis(panels[6], M.vis(b, 17.0, 21.0));
  M.applyVis(panels[7], M.vis(b, 21.0, 24.0));
  M.applyVis(panels[8], M.vis(b, 24.0, 27.0));

  // Spring-driven focal zoom/pan into the active canvas area of each real micro-interaction capture
  applyImgPan('v2-img-2', b, 2.4, 1.02, 1.18, 0, 16, 0, 10);
  applyImgPan('v2-img-3', b, 5.6, 1.02, 1.18, 0, 14, 0, 10);
  applyImgPan('v2-img-4', b, 8.8, 1.02, 1.18, 0, 16, 0, 8);
  applyImgPan('v2-img-5', b, 12.0, 1.02, 1.18, 0, 16, 0, 8);
  applyImgPan('v2-img-6', b, 15.2, 1.02, 1.18, 0, 14, 0, 8);
  applyImgPan('v2-img-7', b, 18.2, 1.02, 1.16, 0, 28, 0, 6);
  applyImgPan('v2-img-8', b, 21.2, 1.02, 1.18, 0, 14, 0, 8);
  applyImgPan('v2-img-9', b, 24.2, 1.02, 1.18, 0, 14, 0, 8);

  // Dynamic overlay updates
  const gapVal = b < 3.8 ? 20 : 28;
  document.getElementById('v2-gap-pill').textContent = \`GAP: \${gapVal}px\`;
  document.getElementById('v2-edge-pill').textContent =
    b < 4.2 ? 'Double-click Edge → Hug Contents' : '⌥ Double-click Edge → Fill 100%';

  document.getElementById('v2-grid-pill').textContent =
    b < 6.8 ? 'COL 1 · 1fr' : (b < 7.8 ? 'COL 1 · 220px' : 'COL 1 · Hug');

  document.getElementById('v2-eye-pill').textContent =
    b < 19.2 ? '#E36323 · HEX (TAB ⇥)' : (b < 20.1 ? 'rgb(227, 99, 35) · RGB' : 'hsl(20°, 78%, 51%) · HSL');

  const scaleHl = M.liquidEdges(b, 4, 150, [
    [24.8, 152, 302],
    [25.8, 304, 452],
  ]);
  const scaleHlEl = document.getElementById('v2-scale-hl');
  scaleHlEl.style.left = \`\${scaleHl.left.toFixed(1)}px\`;
  scaleHlEl.style.width = \`\${scaleHl.width.toFixed(1)}px\`;

  const cx = M.track(b, 650, [
    [0.8, 820, M.MORPH],
    [3.2, 680, M.MORPH],
    [6.6, 620, M.MORPH],
    [9.8, 740, M.MORPH],
    [13.0, 640, M.MORPH],
    [16.2, 700, M.MORPH],
    [19.2, 780, M.MORPH],
    [22.2, 680, M.MORPH],
    [25.0, 720, M.MORPH],
    [26.6, 650, M.MORPH],
  ]);
  const cy = M.track(b, 720, [
    [0.8, 720, M.MORPH],
    [3.2, 690, M.MORPH],
    [6.6, 660, M.MORPH],
    [9.8, 680, M.MORPH],
    [13.0, 690, M.MORPH],
    [16.2, 680, M.MORPH],
    [19.2, 690, M.MORPH],
    [22.2, 680, M.MORPH],
    [25.0, 1045, M.MORPH],
    [26.6, 720, M.MORPH],
  ]);

  const clicks = [0.6, 3.2, 4.2, 6.8, 7.8, 9.8, 13.0, 16.2, 19.2, 20.1, 22.2, 24.8, 25.8];
  const ce = M.clickEffect(b, clicks);
  cursorEl.style.transform = \`translate(\${cx.toFixed(1)}px, \${cy.toFixed(1)}px) scale(\${ce.scale.toFixed(3)})\`;

  if (ce.rippleOp > 0.01) {
    rippleEl.style.opacity = ce.rippleOp.toFixed(3);
    rippleEl.style.transform = \`translate(\${cx.toFixed(1)}px, \${cy.toFixed(1)}px) scale(\${ce.rippleScale.toFixed(3)})\`;
  } else {
    rippleEl.style.opacity = '0';
  }

  let keyText = '';
  if (b < 2.1) keyText = '⇧ Shift · ×10 Scrub';
  else if (b >= 2.2 && b < 5.3) keyText = '⌥ + Double-Click Edge';
  else if (b >= 15.0 && b < 17.9) keyText = '⇧⌫ Heal Delete';
  else if (b >= 18.0 && b < 20.9) keyText = 'I · Tab Format Cycle';
  else if (b >= 24.0 && b < 26.7) keyText = 'K · Scale Tool';
  if (keyText) {
    keyPillEl.textContent = keyText;
    keyPillEl.style.opacity = '1';
  } else {
    keyPillEl.style.opacity = '0';
  }

  const captions = [
    [2.2, 'MICRO 01 / 09', 'Numeric Scrubbing & Math Input (⇧ Shift ×10)'],
    [5.4, 'MICRO 02 / 09', 'Auto-Layout 5.0 · On-Canvas Pink Gap & Hug/Fill Cycle'],
    [8.6, 'MICRO 03 / 09', '2D CSS Grid · Clickable Track Header Pills & 2×1 Span'],
    [11.8, 'MICRO 04 / 09', 'Smart Selection · Pink Ring Swap Handles & Tidy Up'],
    [15.0, 'MICRO 05 / 09', 'Parametric Shape Controls · Donut Arc Sweep & Star Ratio'],
    [18.0, 'MICRO 06 / 09', 'Vector Network Editor · Bezier Tangents & Midpoint Split'],
    [21.0, 'MICRO 07 / 09', 'Layer Search, Status Badge Cycle & Eyedropper Loupe (I)'],
    [24.0, 'MICRO 08 / 09', 'Dev Mode Box-Model · Emerald Padding & Orange Gap Bands'],
    [26.85, 'MICRO 09 / 09', 'Proportional Scale Tool (K) · Live 1.5× & 2.0× Scaling'],
    [28.1, 'MICRO 01 / 09', 'Numeric Scrubbing & Math Input (⇧ Shift ×10)'],
  ];
  for (const [endB, tag, lbl] of captions) {
    if (b <= endB) {
      stepTagEl.textContent = tag;
      stepLabelEl.textContent = lbl;
      break;
    }
  }
};

window.__readyPromise = Promise.all([
  document.fonts.ready,
  ...Array.from(document.images).map((img) => img.decode().catch(() => {})),
]).then(() => {
  window.seek(0);
  return true;
});
</script>
</body>
</html>`;
}

// ============================================================================
// VIDEO 03: 03-agent-first-specs-and-devmode.html
// Uses real Studio assets: 02-agents-chat, 04-cmdk-ai-synthesizer,
// 03-spec-reader, 06-variables-tokens, 07-components-variants,
// 05-dev-mode-codegen, 15-micro-dev-boxmodel, 08-prototype-flows
// ============================================================================
function buildVideo03Html() {
  return `<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8" />
<title>Design OS Studio — Reel 03: Agent-First Specs &amp; Dev Mode (Real Studio Assets)</title>
<style>
${SHARED_CSS}
</style>
</head>
<body>
<div id="stage">
  <div id="hud-top">
    <div class="brand-pill">
      <span class="brand-dot" style="background:var(--emerald);"></span>
      <span>design:os studio · Agent-First Specs &amp; Dev Mode · Real Studio Capture</span>
    </div>
    <div class="beat-pill" id="beat-readout">BAR 1.1 · 100.6 BPM</div>
  </div>

  <div id="camera">
    <div id="shape">
      <!-- State 1 & Loop (B00..B02.2 & B26.85..B28): Ask Design Agent Pill -->
      <div class="state-panel" id="st-1" style="flex-direction:row; align-items:center; justify-content:center; gap:14px; color:#FFF;">
        <span style="width:36px; height:36px; border-radius:50%; background:var(--orange); display:inline-flex; align-items:center; justify-content:center; font-size:19px;">✦</span>
        <span style="font-size:22px; font-weight:600; letter-spacing:-0.02em;">Ask Design Agent</span>
      </div>

      <!-- State 2 (B02.2..B06.0): Real 02-agents-chat.png + 5-Runtime Liquid Selector -->
      <div class="state-panel" id="st-2">
        ${studioHeader('design:os studio — Native PTY Multi-Agent Runtime Bridge', 'PTY STREAM ACTIVE', 'var(--orange)')}
        <div class="studio-viewport">
          <img id="v3-img-2" class="studio-screenshot" src="../screenshots/02-agents-chat.png" alt="Agents Chat" />
          <div style="position:absolute; bottom:22px; left:50%; transform:translateX(-50%); width:680px; z-index:8;">
            <div class="liquid-bar">
              <div class="liquid-indicator" id="v3-rt-hl" style="background:var(--orange);"></div>
              <div class="liquid-tab">Claude Code</div>
              <div class="liquid-tab">Codex</div>
              <div class="liquid-tab">Gemini</div>
              <div class="liquid-tab">Grok</div>
              <div class="liquid-tab">Antigravity</div>
            </div>
          </div>
        </div>
      </div>

      <!-- State 3 (B06.0..B09.5): Real 04-cmdk-ai-synthesizer.png + NodeSpec Diff Overlay -->
      <div class="state-panel" id="st-3">
        ${studioHeader('design:os studio — ⌘K AI Synthesizer & Live NodeSpec Mutation', 'ZERO-FAKE-UI', 'var(--blue)')}
        <div class="studio-viewport">
          <img id="v3-img-3" class="studio-screenshot" src="../screenshots/04-cmdk-ai-synthesizer.png" alt="Cmd+K AI Synthesizer" />
          <div class="ui-overlay-card" style="left:24px; bottom:22px; display:flex; align-items:center; gap:14px;">
            <span style="background:var(--blue); color:#FFF; font-family:var(--font-mono); font-size:13px; font-weight:700; padding:5px 12px; border-radius:7px;">✦ SYNTHESIZE</span>
            <span style="font-size:15.5px; font-weight:600;">Prompt compiles directly into Auto-Layout 5.0 Canvas Frame</span>
          </div>
        </div>
      </div>

      <!-- State 4 (B09.5..B13.2): Real 03-spec-reader.png + Anchored Spec Comment Thread -->
      <div class="state-panel" id="st-4">
        ${studioHeader('design:os studio — PRODUCT:OS v2 Spec Reader & Review Threads', '12/12 LINKED', 'var(--emerald)')}
        <div class="studio-viewport">
          <img id="v3-img-4" class="studio-screenshot" src="../screenshots/03-spec-reader.png" alt="Spec Reader" />
          <div class="ui-overlay-light" style="left:24px; bottom:22px; display:flex; align-items:center; gap:14px;">
            <span style="background:var(--emerald); color:#FFF; font-family:var(--font-mono); font-size:13px; font-weight:700; padding:5px 12px; border-radius:7px;">THREAD #T4 · RESOLVED ✓</span>
            <span style="font-size:15.5px; font-weight:700;">Bidirectional Spec ↔ Canvas NodeId Verification</span>
          </div>
        </div>
      </div>

      <!-- State 5 (B13.2..B16.8): Real 06-variables-tokens.png + Liquid Light/Dark Mode Toggle -->
      <div class="state-panel" id="st-5">
        ${studioHeader('design:os studio — Semantic Variables & Multi-Mode Tokens', '15/15 TOKEN PARITY', 'var(--orange)')}
        <div class="studio-viewport">
          <img id="v3-img-5" class="studio-screenshot" src="../screenshots/06-variables-tokens.png" alt="Variables & Tokens" />
          <div style="position:absolute; bottom:22px; right:24px; width:320px; z-index:8;">
            <div class="liquid-bar">
              <div class="liquid-indicator" id="v3-mode-hl" style="background:var(--orange);"></div>
              <div class="liquid-tab">☀ Light Mode</div>
              <div class="liquid-tab">☾ Dark Mode</div>
            </div>
          </div>
        </div>
      </div>

      <!-- State 6 (B16.8..B20.2): Real 07-components-variants.png + Liquid Variant Switcher -->
      <div class="state-panel" id="st-6">
        ${studioHeader('design:os studio — Component Set Variants & Instance Overrides', 'VARIANT MATRIX', 'var(--blue)')}
        <div class="studio-viewport">
          <img id="v3-img-6" class="studio-screenshot" src="../screenshots/07-components-variants.png" alt="Components & Variants" />
          <div style="position:absolute; bottom:22px; left:24px; width:380px; z-index:8;">
            <div class="liquid-bar">
              <div class="liquid-indicator" id="v3-var-hl" style="background:var(--blue);"></div>
              <div class="liquid-tab">State=Primary</div>
              <div class="liquid-tab">State=Success</div>
            </div>
          </div>
        </div>
      </div>

      <!-- State 7 (B20.2..B23.6): Real 05-dev-mode-codegen.png + 15-micro-dev-boxmodel.png -->
      <div class="state-panel" id="st-7">
        ${studioHeader('design:os studio — Dev Mode Codegen & Box-Model Inspection', '5 CODEGEN TARGETS', 'var(--emerald)')}
        <div class="studio-viewport">
          <img id="v3-img-7" class="studio-screenshot" src="../screenshots/05-dev-mode-codegen.png" alt="Dev Mode Codegen" />
          <div class="ui-overlay-card" style="right:24px; bottom:22px; display:flex; align-items:center; gap:14px;">
            <span style="background:var(--emerald); color:#FFF; font-family:var(--font-mono); font-size:13px; font-weight:700; padding:5px 12px; border-radius:7px;">REACT · TAILWIND · SWIFTUI · EGUI</span>
            <span style="font-size:15px; font-weight:600;">Zero-hallucination code from bound tokens</span>
          </div>
        </div>
      </div>

      <!-- State 8 (B23.6..B26.85): Real 08-prototype-flows.png + Ready for Dev Badge -->
      <div class="state-panel" id="st-8">
        ${studioHeader('design:os studio — Interactive Prototype Flows & E2E Proof', '856/856 TESTS PASS', 'var(--emerald)')}
        <div class="studio-viewport">
          <img id="v3-img-8" class="studio-screenshot" src="../screenshots/08-prototype-flows.png" alt="Prototype Flows" />
          <div class="ui-overlay-card" style="left:50%; bottom:22px; transform:translateX(-50%); display:flex; align-items:center; gap:14px; white-space:nowrap;">
            <span style="background:var(--emerald); color:#FFF; font-family:var(--font-mono); font-size:13px; font-weight:700; padding:5px 12px; border-radius:999px;">✓ READY FOR DEV</span>
            <span style="font-size:15.5px; font-weight:700;">856/856 E2E Tests · 12/12 Zero-Fake-UI · 9/9 ALIVE</span>
          </div>
        </div>
      </div>
    </div>
  </div>

  ${CURSOR_SVG}

  <div id="hud-bottom">
    <div class="state-caption-pill">
      <span class="state-step-tag" id="step-tag" style="background:var(--emerald);">AGENT 01 / 08</span>
      <span id="step-label">Ask Design Agent · Native PTY Bridge</span>
    </div>
    <div class="brand-pill" style="font-family:var(--font-mono); font-size:13.5px;">
      <span>AGENT-FIRST SPECS &amp; DEV MODE · REAL STUDIO CAPTURE</span>
    </div>
  </div>
</div>

<script>
${SHARED_JS_ENGINE}

const shapeEl = document.getElementById('shape');
const cameraEl = document.getElementById('camera');
const cursorEl = document.getElementById('cursor');
const keyPillEl = document.getElementById('cursor-key-pill');
const rippleEl = document.getElementById('click-ripple');
const beatReadoutEl = document.getElementById('beat-readout');
const stepTagEl = document.getElementById('step-tag');
const stepLabelEl = document.getElementById('step-label');

const panels = Array.from({ length: 8 }, (_, i) => document.getElementById('st-' + (i + 1)));

function applyImgPan(id, b, b0, s0, s1, tx0, tx1, ty0, ty1) {
  const el = document.getElementById(id);
  if (!el) return;
  const p = M.S((b - b0) * BEAT, 8, 0.92);
  const sc = s0 + (s1 - s0) * p;
  const tx = tx0 + (tx1 - tx0) * p;
  const ty = ty0 + (ty1 - ty0) * p;
  el.style.transform = \`scale(\${sc.toFixed(4)}) translate(\${tx.toFixed(1)}px, \${ty.toFixed(1)}px)\`;
}

window.seek = function(rawT) {
  const t = (rawT >= DURATION - 1e-4 || rawT <= 1e-4) ? 0 : ((rawT % DURATION) + DURATION) % DURATION;
  const b = t / BEAT;

  const bar = Math.min(7, Math.floor(b / 4) + 1);
  const beatInBar = (Math.floor(b) % 4) + 1;
  beatReadoutEl.textContent = \`BAR \${bar}.\${beatInBar} · BEAT \${Math.floor(b) + 1}/28\`;

  const w = M.track(b, 310, [
    [2.0, 1180, M.MORPH],
    [27.0, 310, M.MORPH],
  ]);
  const h = M.track(b, 76, [
    [2.0, 760, M.MORPH],
    [27.0, 76, M.MORPH],
  ]);
  const rad = M.track(b, 38, [
    [2.0, 20, M.MORPH],
    [27.0, 38, M.MORPH],
  ]);

  shapeEl.style.width = \`\${w.toFixed(2)}px\`;
  shapeEl.style.height = \`\${h.toFixed(2)}px\`;
  shapeEl.style.marginLeft = \`\${(-w / 2).toFixed(2)}px\`;
  shapeEl.style.marginTop = \`\${(-h / 2).toFixed(2)}px\`;
  shapeEl.style.borderRadius = \`\${rad.toFixed(2)}px\`;

  const camScale = M.track(b, 1.26, [
    [2.0, 1.05, M.MORPH],
    [13.0, 1.06, M.MORPH],
    [24.0, 1.05, M.MORPH],
    [27.0, 1.26, M.MORPH],
  ]);
  cameraEl.style.transform = \`scale(\${camScale.toFixed(4)})\`;

  M.applyVis(panels[0], M.vis(b, 0, 2.0, true));
  M.applyVis(panels[1], M.vis(b, 2.0, 6.0));
  M.applyVis(panels[2], M.vis(b, 6.0, 10.0));
  M.applyVis(panels[3], M.vis(b, 10.0, 13.0));
  M.applyVis(panels[4], M.vis(b, 13.0, 17.0));
  M.applyVis(panels[5], M.vis(b, 17.0, 20.0));
  M.applyVis(panels[6], M.vis(b, 20.0, 24.0));
  M.applyVis(panels[7], M.vis(b, 24.0, 27.0));

  applyImgPan('v3-img-2', b, 2.4, 1.02, 1.15, 0, -24, 0, 8);
  applyImgPan('v3-img-3', b, 6.2, 1.02, 1.15, 0, 0, 0, 12);
  applyImgPan('v3-img-4', b, 9.7, 1.02, 1.14, 0, 20, 0, -6);
  applyImgPan('v3-img-5', b, 13.4, 1.02, 1.16, 0, -22, 0, -8);
  applyImgPan('v3-img-6', b, 17.0, 1.02, 1.15, 0, 14, 0, -6);
  applyImgPan('v3-img-7', b, 20.4, 1.02, 1.16, 0, -28, 0, 8);
  applyImgPan('v3-img-8', b, 23.8, 1.02, 1.15, 0, 12, 0, 6);

  // State 2: 5-Runtime liquid indicator
  const rtHl = M.liquidEdges(b, 4, 136, [
    [3.2, 138, 272],
    [4.4, 540, 674],
  ]);
  const rtEl = document.getElementById('v3-rt-hl');
  rtEl.style.left = \`\${rtHl.left.toFixed(1)}px\`;
  rtEl.style.width = \`\${rtHl.width.toFixed(1)}px\`;

  // State 5: Light/Dark mode liquid indicator
  const modeHl = M.liquidEdges(b, 4, 158, [
    [14.4, 160, 314],
  ]);
  const modeEl = document.getElementById('v3-mode-hl');
  modeEl.style.left = \`\${modeHl.left.toFixed(1)}px\`;
  modeEl.style.width = \`\${modeHl.width.toFixed(1)}px\`;

  // State 6: Variant toggle liquid indicator
  const varHl = M.liquidEdges(b, 4, 188, [
    [18.0, 190, 374],
  ]);
  const varEl = document.getElementById('v3-var-hl');
  varEl.style.left = \`\${varHl.left.toFixed(1)}px\`;
  varEl.style.width = \`\${varHl.width.toFixed(1)}px\`;

  const cx = M.track(b, 720, [
    [1.0, 720, M.MORPH],
    [3.2, 580, M.MORPH],
    [4.4, 980, M.MORPH],
    [7.2, 720, M.MORPH],
    [10.8, 620, M.MORPH],
    [14.4, 1120, M.MORPH],
    [18.0, 420, M.MORPH],
    [21.4, 920, M.MORPH],
    [24.6, 720, M.MORPH],
    [26.6, 720, M.MORPH],
  ]);
  const cy = M.track(b, 720, [
    [1.0, 720, M.MORPH],
    [3.2, 1045, M.MORPH],
    [4.4, 1045, M.MORPH],
    [7.2, 680, M.MORPH],
    [10.8, 1030, M.MORPH],
    [14.4, 1045, M.MORPH],
    [18.0, 1045, M.MORPH],
    [21.4, 1030, M.MORPH],
    [24.6, 1030, M.MORPH],
    [26.6, 720, M.MORPH],
  ]);

  const clicks = [1.0, 3.2, 4.4, 7.2, 10.8, 14.4, 18.0, 21.4, 24.6];
  const ce = M.clickEffect(b, clicks);
  cursorEl.style.transform = \`translate(\${cx.toFixed(1)}px, \${cy.toFixed(1)}px) scale(\${ce.scale.toFixed(3)})\`;

  if (ce.rippleOp > 0.01) {
    rippleEl.style.opacity = ce.rippleOp.toFixed(3);
    rippleEl.style.transform = \`translate(\${cx.toFixed(1)}px, \${cy.toFixed(1)}px) scale(\${ce.rippleScale.toFixed(3)})\`;
  } else {
    rippleEl.style.opacity = '0';
  }

  let keyText = '';
  if (b >= 2.2 && b < 5.8) keyText = '5 Native CLI Runtimes';
  else if (b >= 13.2 && b < 16.6) keyText = 'Variables · Light / Dark';
  else if (b >= 20.2 && b < 23.4) keyText = 'Dev Mode · 5 Targets';
  if (keyText) {
    keyPillEl.textContent = keyText;
    keyPillEl.style.opacity = '1';
  } else {
    keyPillEl.style.opacity = '0';
  }

  const captions = [
    [2.2, 'AGENT 01 / 08', 'Ask Design Agent · Native PTY Bridge'],
    [6.0, 'AGENT 02 / 08', 'Multi-Agent PTY Chat · Claude, Codex, Gemini, Grok & AGY'],
    [9.5, 'AGENT 03 / 08', '⌘K AI Synthesizer · Prompt to Auto-Layout 5.0 Frame'],
    [13.2, 'AGENT 04 / 08', 'PRODUCT:OS v2 Spec Reader · Anchored Review Threads'],
    [16.8, 'AGENT 05 / 08', 'Semantic Variables & Multi-Mode Tokens · Light & Dark'],
    [20.2, 'AGENT 06 / 08', 'Component Set Variants & Liquid Property Switch'],
    [23.6, 'AGENT 07 / 08', 'Dev Mode Multi-Target Codegen · React, Tailwind & SwiftUI'],
    [26.85, 'AGENT 08 / 08', 'Interactive Prototype Flows · 856/856 E2E Tests PASS'],
    [28.1, 'AGENT 01 / 08', 'Ask Design Agent · Native PTY Bridge'],
  ];
  for (const [endB, tag, lbl] of captions) {
    if (b <= endB) {
      stepTagEl.textContent = tag;
      stepLabelEl.textContent = lbl;
      break;
    }
  }
};

window.__readyPromise = Promise.all([
  document.fonts.ready,
  ...Array.from(document.images).map((img) => img.decode().catch(() => {})),
]).then(() => {
  window.seek(0);
  return true;
});
</script>
</body>
</html>`;
}

fs.writeFileSync(path.join(OUT_DIR, '01-designos-studio-launch-hero.html'), buildVideo01Html());
fs.writeFileSync(path.join(OUT_DIR, '02-figma-100-micro-interactions.html'), buildVideo02Html());
fs.writeFileSync(path.join(OUT_DIR, '03-agent-first-specs-and-devmode.html'), buildVideo03Html());

console.log('Generated 3 Real-Asset + One-Shape UI Motion HTML engines in', OUT_DIR);
