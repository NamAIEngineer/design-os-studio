#!/usr/bin/env node
/**
 * Renders Design OS Studio's 3 Dribbble One-Shape UI Motion videos:
 *  1. 28-beat Contact Sheets (7x4 grid, 1 frame per beat) + exact Loop Verification (Frame 0 === Frame Last)
 *  2. Full 60fps 1440x1440 render with 4 subframes per frame (180-deg shutter) blended via ffmpeg tmix=frames=4
 */
import { createRequire } from 'node:module';
import { spawn, execFileSync } from 'node:child_process';
import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';

const require = createRequire('/Users/jangtrinh/Products/design-os-svg-animation/package.json');
const puppeteer = require('puppeteer-core');

const ROOT = '/Users/jangtrinh/Products/design-os-studio';
const SVG_DIR = path.join(ROOT, 'Design/marketing/svg-animation');
const PROOFS_DIR = path.join(SVG_DIR, 'proofs');
const AUDIO_DIR = path.join(SVG_DIR, 'audio');
const VIDEOS_DIR = path.join(ROOT, 'Design/marketing/videos');

fs.mkdirSync(PROOFS_DIR, { recursive: true });
fs.mkdirSync(VIDEOS_DIR, { recursive: true });

const CHROME_PATH = '/Applications/Google Chrome.app/Contents/MacOS/Google Chrome';
const WIDTH = 1440;
const HEIGHT = 1440;
const FPS = 60;
const DURATION = 16.70;
const BEATS = 28;
const BEAT = DURATION / BEATS;
const TOTAL_FRAMES = Math.round(DURATION * FPS); // 1002 frames
const SUBFRAMES = 4;
const SHUTTER = 0.5; // 180-deg shutter

const VIDEOS = [
  '01-designos-studio-launch-hero',
  '02-figma-100-micro-interactions',
  '03-agent-first-specs-and-devmode',
];

async function renderBeatsAndVerifyLoop(browser, id) {
  const htmlPath = path.join(SVG_DIR, `${id}.html`);
  const page = await browser.newPage();
  await page.setViewport({ width: WIDTH, height: HEIGHT, deviceScaleFactor: 1 });
  await page.goto(`file://${htmlPath}?render=1`, { waitUntil: 'load' });
  await page.evaluate(async () => {
    await window.__readyPromise;
    window.seek(0);
    document.body.offsetHeight;
    await new Promise((r) => requestAnimationFrame(() => requestAnimationFrame(r)));
  });

  // 1. Verify exact loop closure: frame(0) === frame(DURATION)
  await page.evaluate(async () => {
    window.seek(0);
    document.body.offsetHeight;
    await new Promise((r) => requestAnimationFrame(r));
  });
  const bufFirst = await page.screenshot({ type: 'png' });
  await page.evaluate(async (d) => {
    window.seek(d);
    document.body.offsetHeight;
    await new Promise((r) => requestAnimationFrame(r));
  }, DURATION);
  const bufLast = await page.screenshot({ type: 'png' });

  const hashFirst = crypto.createHash('sha256').update(bufFirst).digest('hex');
  const hashLast = crypto.createHash('sha256').update(bufLast).digest('hex');
  const loopIdentical = hashFirst === hashLast;
  console.log(`[${id}] Loop check: first=${hashFirst.slice(0, 12)} last=${hashLast.slice(0, 12)} identical=${loopIdentical}`);
  if (!loopIdentical) {
    throw new Error(`[${id}] Loop closure failed! Frame 0 and Frame Last differ.`);
  }

  // 2. Render 1 frame per beat (28 beats, sampled at beat + 0.32 so post-spring state is crisp)
  const beatDir = path.join(PROOFS_DIR, `${id}-beats`);
  fs.mkdirSync(beatDir, { recursive: true });

  for (let b = 0; b < BEATS; b++) {
    const sampleBeat = b === 0 ? 0.0 : b + 0.32;
    const t = sampleBeat * BEAT;
    await page.evaluate((timeSec) => window.seek(timeSec), t);
    const framePath = path.join(beatDir, `beat_${String(b).padStart(2, '0')}.png`);
    await page.screenshot({ path: framePath, type: 'png' });
  }

  // Assemble 7x4 contact sheet using ffmpeg tile filter
  const contactSheetPath = path.join(PROOFS_DIR, `${id}-beat-contact-sheet.png`);
  execFileSync('ffmpeg', [
    '-y',
    '-framerate', '1',
    '-i', path.join(beatDir, 'beat_%02d.png'),
    '-vf', 'scale=360:360,tile=7x4:margin=8:padding=6:color=0x18181B',
    '-frames:v', '1',
    contactSheetPath,
  ], { stdio: 'ignore' });

  console.log(`[${id}] Saved 28-beat contact sheet -> ${contactSheetPath}`);
  await page.close();
}

async function renderFullVideo(browser, id) {
  const htmlPath = path.join(SVG_DIR, `${id}.html`);
  const wavPath = path.join(AUDIO_DIR, `${id}-master.wav`);
  const outMp4 = path.join(VIDEOS_DIR, `${id}.mp4`);

  const NUM_WORKERS = 6;
  const pages = [];
  for (let w = 0; w < NUM_WORKERS; w++) {
    const p = await browser.newPage();
    await p.setViewport({ width: WIDTH, height: HEIGHT, deviceScaleFactor: 1 });
    await p.goto(`file://${htmlPath}?render=1`, { waitUntil: 'load' });
    await p.evaluate(() => window.__readyPromise);
    pages.push(p);
  }

  const vf = `tmix=frames=${SUBFRAMES}:weights='1 1 1 1',select='eq(mod(n\\,${SUBFRAMES})\\,${SUBFRAMES - 1})',setpts=N/${FPS}/TB`;
  const ffArgs = [
    '-y',
    '-v', 'error',
    '-f', 'image2pipe',
    '-vcodec', 'mjpeg',
    '-framerate', String(FPS * SUBFRAMES),
    '-i', '-',
    '-i', wavPath,
    '-vf', vf,
    '-r', String(FPS),
    '-c:v', 'libx264',
    '-preset', 'fast',
    '-crf', '16',
    '-pix_fmt', 'yuv420p',
    '-c:a', 'aac',
    '-b:a', '192k',
    '-shortest',
    '-movflags', '+faststart',
    outMp4,
  ];

  const ff = spawn('ffmpeg', ffArgs, { stdio: ['pipe', 'inherit', 'inherit'] });
  const ffDone = new Promise((resolve, reject) => {
    ff.on('close', (code) => (code === 0 ? resolve() : reject(new Error(`ffmpeg exited ${code}`))));
  });

  // Build list of all subframe timestamps (1002 * 4 = 4008 subframes)
  // Ensure the final frame's subframes lock to DURATION so Frame Last === Frame First
  const subTimes = [];
  for (let f = 0; f < TOTAL_FRAMES; f++) {
    for (let s = 0; s < SUBFRAMES; s++) {
      if (f === 0) {
        subTimes.push(0);
      } else if (f === TOTAL_FRAMES - 1) {
        subTimes.push(DURATION);
      } else {
        const dt = (s / SUBFRAMES) * (SHUTTER / FPS);
        subTimes.push(Math.min(DURATION - 1e-3, f / FPS + dt));
      }
    }
  }

  const t0 = Date.now();
  for (let idx = 0; idx < subTimes.length; idx += NUM_WORKERS) {
    const batch = subTimes.slice(idx, idx + NUM_WORKERS);
    const buffers = await Promise.all(
      batch.map(async (tSec, wIdx) => {
        const page = pages[wIdx];
        await page.evaluate((t) => window.seek(t), tSec);
        return page.screenshot({ type: 'jpeg', quality: 94 });
      })
    );
    for (const buf of buffers) {
      if (!ff.stdin.write(buf)) {
        await new Promise((r) => ff.stdin.once('drain', r));
      }
    }
    if ((idx / SUBFRAMES) % 120 === 0) {
      const doneFrames = Math.min(TOTAL_FRAMES, Math.floor((idx + batch.length) / SUBFRAMES));
      const elapsed = ((Date.now() - t0) / 1000).toFixed(1);
      console.log(`[${id}] Rendered ${doneFrames}/${TOTAL_FRAMES} frames (${SUBFRAMES}x tmix subframes) in ${elapsed}s`);
    }
  }

  ff.stdin.end();
  await ffDone;
  for (const p of pages) await p.close();

  const stat = fs.statSync(outMp4);
  console.log(`[${id}] Saved 60fps 4x-tmix MP4 -> ${outMp4} (${(stat.size / 1024 / 1024).toFixed(2)} MB)`);
}

async function main() {
  const args = process.argv.slice(2);
  const beatsOnly = args.includes('--beats-only');
  const fullOnly = args.includes('--full-only');

  const browser = await puppeteer.launch({
    executablePath: CHROME_PATH,
    headless: 'new',
    args: ['--no-sandbox', '--disable-gpu-vsync', '--font-render-hinting=none'],
  });

  try {
    if (!fullOnly) {
      for (const id of VIDEOS) {
        await renderBeatsAndVerifyLoop(browser, id);
      }
    }
    if (!beatsOnly) {
      for (const id of VIDEOS) {
        await renderFullVideo(browser, id);
      }
      // Also generate animated GIF preview from Video 01 for README.md & GitHub Pages
      const heroMp4 = path.join(VIDEOS_DIR, '01-designos-studio-launch-hero.mp4');
      const outGif = path.join(VIDEOS_DIR, 'designos-studio-launch-preview.gif');
      execFileSync('ffmpeg', [
        '-y',
        '-v', 'error',
        '-i', heroMp4,
        '-vf', 'fps=20,scale=720:720:flags=lanczos,split[s0][s1];[s0]palettegen=max_colors=128[p];[s1][p]paletteuse=dither=bayer:bayer_scale=3',
        outGif,
      ]);
      console.log(`Generated animated GIF preview -> ${outGif}`);
    }
  } finally {
    await browser.close();
  }
}

main().catch((err) => {
  console.error(err);
  process.exit(1);
});
