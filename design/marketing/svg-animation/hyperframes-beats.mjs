import { BEAT_KINDS, buildHyperFramesDraft } from './runtime/hyperframes-engine.mjs';

export const HYPERFRAME_DURATION = 16.70;
export const BPM = 100.60;
export const BEAT_S = 60 / BPM; // 0.596421s

export const STUDIO_BRAND_KIT = {
  tokens: {
    bg: '#f4f3ef',
    fg: '#111111',
    accent: '#0d99ff',
    onAccent: '#ffffff',
    muted: '#71717a',
    line: '#d4d4d0',
    fill: '#e8e7e2',
    card: '#ffffff',
  },
  fontFamily: 'Geist, Inter, -apple-system, system-ui, sans-serif',
  energy: 'punchy',
  motion: 'spring',
};

const b = (beat) => Number((beat * BEAT_S).toFixed(4));

export const HYPERFRAME_STORYBOARDS = {
  '01-designos-studio-launch-hero': [
    { id: 's1-launch-pill', start: b(0), end: b(2), kind: 'logo', text: 'DESIGN:OS Studio', options: { sub: 'Open .fig v106' } },
    { id: 's2-kiwi-loader', start: b(2), end: b(4), kind: 'bars', text: '723,136 Kiwi Records' },
    { id: 's3-gpu-check', start: b(4), end: b(5), kind: 'notify', text: '1.54s Metal · 0.0% Idle CPU' },
    { id: 's4-studio-hero', start: b(5), end: b(9), kind: 'window', text: '01-hero-canvas.png · UI3 Dark Chrome' },
    { id: 's5-agents-pty', start: b(9), end: b(13), kind: 'chat', text: 'Claude Code · Codex · Antigravity PTY' },
    { id: 's6-spec-reader', start: b(13), end: b(16), kind: 'list', text: 'US-01..US-15|Gherkin Criteria|g1..g5 PASS' },
    { id: 's7-cmdk-ai', start: b(16), end: b(19), kind: 'input', text: 'fintech dark checkout card', options: { h: '⌘K Actions & AI Synth' } },
    { id: 's8-variables-components', start: b(19), end: b(23), kind: 'cards', text: 'Variables & 3x2 Button Variants' },
    { id: 's9-devmode-proto', start: b(23), end: b(26), kind: 'split', text: 'Dev Mode 5-Target Codegen & Flows' },
    { id: 's10-toast-loop', start: b(26), end: HYPERFRAME_DURATION, kind: 'notify', text: '387/387 Parity · 100/100 Micro-UX' },
  ],
  '02-figma-100-micro-interactions': [
    { id: 's1-micro-pill', start: b(0), end: b(2), kind: 'logo', text: '100 / 100 Micro-UX', options: { sub: 'Figma A-to-Z Tactile Engine' } },
    { id: 's2-autolayout', start: b(2), end: b(5), kind: 'window', text: '09-micro-autolayout.png · Fixed -> Hug -> Fill' },
    { id: 's3-css-grid', start: b(5), end: b(8), kind: 'cards', text: '10-micro-css-grid.png · 1fr Track Pills' },
    { id: 's4-smart-select', start: b(8), end: b(11), kind: 'bars', text: '11-micro-smart-selection.png · Pink Gaps' },
    { id: 's5-shape-controls', start: b(11), end: b(14), kind: 'shape', text: '12-micro-shape-controls.png · Corner & Arc' },
    { id: 's6-vector-edit', start: b(14), end: b(17), kind: 'chart', text: '13-micro-vector-edit.png · Cubic Tangents' },
    { id: 's7-layer-picker', start: b(17), end: b(20), kind: 'list', text: 'Hero Card|Action Row|Primary CTA Button' },
    { id: 's8-dev-boxmodel', start: b(20), end: b(23), kind: 'split', text: '15-micro-dev-boxmodel.png · Redlines' },
    { id: 's9-scale-tool', start: b(23), end: b(26), kind: 'window', text: '16-micro-scale-tool.png · Scale Tool (K) 1.25x' },
    { id: 's10-verified-loop', start: b(26), end: HYPERFRAME_DURATION, kind: 'notify', text: '100 / 100 Micro-Interactions PASS' },
  ],
  '03-agent-first-specs-and-devmode': [
    { id: 's1-mcp-pill', start: b(0), end: b(2), kind: 'logo', text: 'designos-mcp · 22 Tools', options: { sub: 'Agent-First Local Workflow' } },
    { id: 's2-pty-chat', start: b(2), end: b(6), kind: 'chat', text: '02-agents-chat.png · Zero Extra API Keys' },
    { id: 's3-cmdk-synth', start: b(6), end: b(9), kind: 'input', text: 'Generate SaaS pricing grid', options: { h: '04-cmdk-ai-synthesizer.png' } },
    { id: 's4-spec-reader', start: b(9), end: b(13), kind: 'list', text: '01-intake|05-spec (US-01..15)|11-gates PASS' },
    { id: 's5-variables', start: b(13), end: b(16), kind: 'cards', text: '06-variables-tokens.png · Light / Dark Modes' },
    { id: 's6-components', start: b(16), end: b(19), kind: 'window', text: '07-components-variants.png · Master Matrix' },
    { id: 's7-devmode', start: b(19), end: b(22), kind: 'split', text: '05-dev-mode-codegen.png · CSS / SwiftUI / Compose' },
    { id: 's8-boxmodel', start: b(22), end: b(24), kind: 'window', text: '15-micro-dev-boxmodel.png · Option Redlines' },
    { id: 's9-proto-flows', start: b(24), end: b(26), kind: 'chart', text: '08-prototype-flows.png · Bezier Noodles' },
    { id: 's10-toast-loop', start: b(26), end: HYPERFRAME_DURATION, kind: 'notify', text: '856 Tests · 12/12 Zero-Fake-UI PASS' },
  ],
};

export function buildStudioHyperFrameDraft(root, slug) {
  const storyboard = HYPERFRAME_STORYBOARDS[slug];
  if (!storyboard) throw new Error(`Unknown Studio HyperFrames storyboard: ${slug}`);
  for (const beat of storyboard) {
    if (!Object.hasOwn(BEAT_KINDS, beat.kind)) {
      throw new Error(`Missing HyperFrames preset for ${beat.id}: ${beat.kind}`);
    }
  }
  const beats = storyboard.map(({ kind, text, options }) => [kind, text, options || {}]);
  const starts = storyboard.map((beat) => beat.start);
  return buildHyperFramesDraft(root, beats, HYPERFRAME_DURATION, STUDIO_BRAND_KIT, starts);
}
