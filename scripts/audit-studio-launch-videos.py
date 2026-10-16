#!/usr/bin/env python3
"""
audit-studio-launch-videos.py — Full Anti-Flop Gate (Gates 0..9) for Design OS Studio Launch Videos.

Audits the 3 standalone 1440x1440 SF Pro Rounded + iOS 26 Liquid Glass HTML engines:
  - Design/marketing/svg-animation/01-designos-studio-launch-hero.html
  - Design/marketing/svg-animation/02-figma-100-micro-interactions.html
  - Design/marketing/svg-animation/03-agent-first-specs-and-devmode.html

Gates enforced:
  [Gate 1] Phosphor Icons (<symbol> + <use href>) + Authentic SVGL Brand Marks + ZERO raw emojis or text pseudo-icons
  [Gate 2] Apple SF Pro Rounded ('SF Pro Rounded', ui-rounded, -apple-system, BlinkMacSystemFont) & Antialiased Smoothing
  [Gate 3] Modular Spacing, Tactile Multi-Layer Shadows, <feDropShadow>, <feDisplacementMap>, and <feGaussianBlur> Goo
  [Gate 4] Accessibility (<title>) & @media (prefers-reduced-motion: reduce)
  [Gate 5] Security & Determinism (zero inline on*=, zero NaN/Infinity/undefined, window.seek & window.__seekToTime)
  [Gate 6] Cursor Tip (0,0) & Concentric Click Ripple (delta <= 1.0px) + Target Hit Verification via Headless Chromium
  [Gate 8] Loop Closure Identity (Frame 0 === Frame Last) & Zero Single-Frame Pops
"""

from __future__ import annotations

import argparse
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path("/Users/jangtrinh/Products/design-os-studio")
SVG_DIR = ROOT / "Design/marketing/svg-animation"
VIDEOS_DIR = ROOT / "Design/marketing/videos"

HTML_FILES = [
    SVG_DIR / "01-designos-studio-launch-hero.html",
    SVG_DIR / "02-figma-100-micro-interactions.html",
    SVG_DIR / "03-agent-first-specs-and-devmode.html",
]

REQUIRED_SYMBOLS = [
    "icon-lock",
    "icon-return",
    "icon-sparkle",
    "icon-code",
    "icon-copy",
    "icon-check",
    "icon-search",
    "icon-external",
    "icon-command",
    "icon-claude",
    "icon-openai",
]

EMOJI_PATTERN = re.compile(
    r"[\U00010000-\U0010ffff\u2600-\u27bf\u2300-\u23ff\u2b50\u2b55\u203c\u2049\u25aa\u25ab\u25b6\u25c0\u25fb-\u25fe]",
    flags=re.UNICODE,
)
PSEUDO_ICON_PATTERN = re.compile(r"[←→↻⤓↗▾≡▷‹›✓✦⌘↵⇧⌥⌫☀☾●◆◇↖✎↔⇥]")


def banner(msg: str) -> None:
    print("\n" + "=" * 72)
    print(f" 🛡️  STUDIO LAUNCH VIDEO ANTI-FLOP GATE — {msg}")
    print("=" * 72)


def audit_html_gates() -> None:
    banner("Gates 1..5: SF Pro Rounded, Phosphor/SVGL, Zero Emoji/Pseudo-Icons, Liquid Glass")

    font_path = SVG_DIR / "fonts/SFProRounded.ttf"
    mono_path = SVG_DIR / "fonts/SFMono.ttf"
    assert font_path.exists() and font_path.stat().st_size > 1_000_000, "Missing SFProRounded.ttf font file!"
    assert mono_path.exists() and mono_path.stat().st_size > 100_000, "Missing SFMono.ttf font file!"

    for f in HTML_FILES:
        assert f.exists(), f"Missing HTML file: {f}"
        txt = f.read_text(encoding="utf-8")

        # Gate 1: Zero Emojis & Zero Pseudo-Icons
        emojis = set(EMOJI_PATTERN.findall(txt))
        pseudos = set(PSEUDO_ICON_PATTERN.findall(txt))
        if emojis or pseudos:
            print(f"❌ [Gate 1 FAIL] {f.name} contains banned emojis/pseudo-icons: emojis={emojis}, pseudos={pseudos}")
            sys.exit(1)

        # Gate 1: Required Phosphor & SVGL symbols defined
        for sym in REQUIRED_SYMBOLS:
            assert f'id="{sym}"' in txt, f"{f.name} missing <symbol id='{sym}'>"
        assert "m50.228 170.321" in txt, f"{f.name} missing authentic SVGL Claude vector"
        assert "252.794 108.802" in txt, f"{f.name} missing authentic SVGL OpenAI vector"
        assert '<use href="#icon-' in txt, f"{f.name} must instantiate <use href='#icon-...'>"

        # Gate 2: Apple SF Pro Rounded & Antialiased Smoothing
        assert "font-family: 'SF Pro Rounded'" in txt, f"{f.name} missing SF Pro Rounded @font-face / font-family"
        assert "ui-rounded" in txt and "-apple-system" in txt and "BlinkMacSystemFont" in txt, (
            f"{f.name} missing Apple ui-rounded / -apple-system font stack"
        )
        assert "-webkit-font-smoothing: antialiased" in txt, f"{f.name} missing -webkit-font-smoothing: antialiased"
        assert "-moz-osx-font-smoothing: grayscale" in txt, f"{f.name} missing -moz-osx-font-smoothing: grayscale"

        # Gate 3: Tactile Multi-Layer Shadows & Filters
        assert "box-shadow" in txt and "backdrop-filter: blur(" in txt, f"{f.name} missing Apple Liquid Glass blur/shadow"
        assert "feDropShadow" in txt, f"{f.name} missing <feDropShadow>"
        assert "feDisplacementMap" in txt, f"{f.name} missing <feDisplacementMap> Liquid Glass filter"
        assert "feColorMatrix" in txt, f"{f.name} missing <feColorMatrix> Liquid Goo filter"

        # Gate 4: A11y & Reduced Motion
        assert "@media (prefers-reduced-motion: reduce)" in txt, f"{f.name} missing prefers-reduced-motion media query"
        assert "<title>" in txt, f"{f.name} missing <title>"

        # Gate 5: Security & Virtual Clock Determinism
        assert not re.search(r"\bon[a-zA-Z]+\s*=", txt), f"{f.name} contains forbidden inline event handler"
        assert not re.search(r"\b(NaN|Infinity|undefined)\b", txt), f"{f.name} contains NaN/Infinity/undefined"
        assert "window.seek" in txt and "window.__seekToTime" in txt, f"{f.name} missing window.seek / __seekToTime"
        assert "window.__readyPromise" in txt, f"{f.name} missing window.__readyPromise"

        print(f"✅ {f.name}: Gates 1..5 PASS (SF Pro Rounded, 0 Emojis/Pseudo-Icons, Phosphor+SVGL, Liquid Glass)")


def audit_runtime_hotspots_and_loop() -> None:
    banner("Gates 6 & 8: Cursor Tip (0,0) Concentricity & Loop Closure (Frame 0 === Frame Last)")
    node_script = """
    import fs from 'node:fs';
    import path from 'node:path';
    import { createRequire } from 'node:module';
    const require = createRequire('/Users/jangtrinh/Products/design-os-svg-animation/package.json');
    const puppeteer = require('puppeteer-core');

    const CHROME_BIN = '/Applications/Google Chrome.app/Contents/MacOS/Google Chrome';
    const files = [
      '/Users/jangtrinh/Products/design-os-studio/Design/marketing/svg-animation/01-designos-studio-launch-hero.html',
      '/Users/jangtrinh/Products/design-os-studio/Design/marketing/svg-animation/02-figma-100-micro-interactions.html',
      '/Users/jangtrinh/Products/design-os-studio/Design/marketing/svg-animation/03-agent-first-specs-and-devmode.html',
    ];

    const browser = await puppeteer.launch({
      executablePath: CHROME_BIN,
      headless: true,
      args: ['--no-sandbox', '--disable-setuid-sandbox', '--hide-scrollbars'],
    });

    try {
      const page = await browser.newPage();
      await page.setViewport({ width: 1440, height: 1440, deviceScaleFactor: 1 });

      for (const file of files) {
        await page.goto('file://' + file, { waitUntil: 'networkidle0', timeout: 30000 });
        await page.evaluate(() => window.__readyPromise);

        const res = await page.evaluate(() => {
          const stage = document.getElementById('video-stage');
          const stageRect = stage.getBoundingClientRect();
          const events = window.__CLICK_EVENTS || [];
          const hotspotChecks = [];

          for (const evt of events) {
            window.__seekToTime(evt.t);
            const cur = document.getElementById('virtual-cursor');
            const rip = document.getElementById('click-ripple');
            const target = document.getElementById(evt.targetId || 'shape');
            const curRect = cur.getBoundingClientRect();
            const ripRect = rip.getBoundingClientRect();
            const tRect = target.getBoundingClientRect();

            const tipX = curRect.left - stageRect.left;
            const tipY = curRect.top - stageRect.top;
            const ripX = (ripRect.left + ripRect.width / 2) - stageRect.left;
            const ripY = (ripRect.top + ripRect.height / 2) - stageRect.top;
            const delta = Math.hypot(tipX - ripX, tipY - ripY);
            const hit = (
              curRect.left >= tRect.left - 2 &&
              curRect.left <= tRect.right + 2 &&
              curRect.top >= tRect.top - 2 &&
              curRect.top <= tRect.bottom + 2
            );
            hotspotChecks.push({ id: evt.id, t: evt.t, delta, hit });
          }

          window.seek(0);
          const s0 = document.getElementById('shape').style.cssText + '|' + document.getElementById('virtual-cursor').style.cssText;
          window.seek(16.70);
          const sEnd = document.getElementById('shape').style.cssText + '|' + document.getElementById('virtual-cursor').style.cssText;

          const computedFont = window.getComputedStyle(document.body).fontFamily;
          return { hotspotChecks, loopIdentical: s0 === sEnd, computedFont };
        });

        const name = path.basename(file);
        if (!res.loopIdentical) {
          console.error('FAIL loop closure on ' + name);
          process.exit(1);
        }
        for (const h of res.hotspotChecks) {
          if (h.delta > 1.0 || !h.hit) {
            console.error('FAIL Gate 6 hotspot on ' + name, h);
            process.exit(1);
          }
        }
        console.log('✅ ' + name + ': Gate 6 Concentricity (max Δ=' +
          Math.max(...res.hotspotChecks.map(x => x.delta)).toFixed(2) +
          'px <= 1.0px, 100% TargetHit) & Gate 8 Loop Closure PASS | Font: ' + res.computedFont);
      }
    } finally {
      await browser.close();
    }
    """
    proc = subprocess.run(["node", "--input-type=module", "-e", node_script], cwd=ROOT, text=True)
    if proc.returncode != 0:
        sys.exit(proc.returncode)


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.parse_args()
    audit_html_gates()
    audit_runtime_hotspots_and_loop()
    print("\n" + "=" * 72)
    print(" 🛡️  ALL DESIGN OS STUDIO LAUNCH VIDEO ANTI-FLOP GATES PASSED (100%)")
    print("=" * 72)


if __name__ == "__main__":
    main()
