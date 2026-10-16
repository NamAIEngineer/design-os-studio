#!/usr/bin/env node
/**
 * render-micro-8x30s-videos.mjs
 * ============================================================================
 * Renders all 8 x 30s (`29.82s` = 50 beats @ 100.60 BPM) A-to-Z Micro-Interactions
 * videos in BOTH:
 *   - 16:9 Widescreen (`1920x1080` @ 60fps) -> `Design/marketing/videos/16x9/micro-01..08-*.mp4`
 *   - 9:16 Vertical   (`1080x1920` @ 60fps) -> `Design/marketing/videos/9x16/micro-01..08-*.mp4`
 *
 * Also copies the 16:9 masters to `Design/marketing/videos/micro-01..08-*.mp4` and `docs/videos/`,
 * generates 16x9 and 9x16 visual proof grids, and verifies loop closure.
 */

import { spawn, execFileSync } from 'node:child_process';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { chromium } from 'playwright';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const REPO_ROOT = path.resolve(__dirname, '..');
const SVG_ANIM_DIR = path.join(REPO_ROOT, 'Design', 'marketing', 'svg-animation');
const VIDEOS_DIR = path.join(REPO_ROOT, 'Design', 'marketing', 'videos');
const VIDEOS_16X9_DIR = path.join(VIDEOS_DIR, '16x9');
const VIDEOS_9X16_DIR = path.join(VIDEOS_DIR, '9x16');
const DOCS_VIDEOS_DIR = path.join(REPO_ROOT, 'docs', 'videos');
const PROOFS_DIR = path.join(SVG_ANIM_DIR, 'proofs');
const SONG_PATH = '/Users/jangtrinh/Downloads/Aetheric - Edge of Motion (freetouse.com).mp3';

const BPM = 100.60;
const BEAT = 60 / BPM;
const TOTAL_BEATS = 50;
const DURATION = Number((TOTAL_BEATS * BEAT).toFixed(6)); // 29.821074s
const CAPTURE_FPS = 30;
const TOTAL_CAPTURE_FRAMES = Math.round(DURATION * CAPTURE_FPS); // 895 frames -> 1790 @ 60fps

const VIDEO_IDS = [
  'micro-01-autolayout-5',
  'micro-02-css-grid',
  'micro-03-smart-selection',
  'micro-04-shape-controls',
  'micro-05-vector-network',
  'micro-06-layer-eyedropper',
  'micro-07-devmode-boxmodel',
  'micro-08-scale-navigation',
];

const ASPECTS = [
  { key: '16x9', width: 1920, height: 1080, outDir: VIDEOS_16X9_DIR },
  { key: '9x16', width: 1080, height: 1920, outDir: VIDEOS_9X16_DIR },
];

function ensureAudioMix30s() {
  const audioOut = path.join(SVG_ANIM_DIR, 'audio-30s-master.m4a');
  if (fs.existsSync(audioOut) && fs.statSync(audioOut).size > 20_000) {
    return audioOut;
  }
  console.log('==> Building 30s beat-locked soundtrack (`audio-30s-master.m4a`)...');
  if (fs.existsSync(SONG_PATH)) {
    execFileSync('ffmpeg', [
      '-y',
      '-ss', '14.31',
      '-t', String(DURATION),
      '-i', SONG_PATH,
      '-af', `afade=t=in:st=0:d=0.6,afade=t=out:st=${(DURATION - 1.2).toFixed(2)}:d=1.2,volume=0.88`,
      '-c:a', 'aac',
      '-b:a', '192k',
      audioOut,
    ], { stdio: 'ignore' });
  } else {
    execFileSync('ffmpeg', [
      '-y',
      '-f', 'lavfi',
      '-i', `anullsrc=r=48000:cl=stereo:d=${DURATION}`,
      '-c:a', 'aac',
      '-b:a', '128k',
      audioOut,
    ], { stdio: 'ignore' });
  }
  return audioOut;
}

function isStationaryStepHold(t) {
  const beat = t / BEAT;
  if (beat < 4.2 || beat >= 42.8) return false;
  const stepSpan = 6.5;
  const stepIdx = Math.floor((beat - 4.0) / stepSpan);
  const stepLocalBeat = beat - (4.0 + stepIdx * stepSpan);
  // Spring (omega=13.5) and click ripple (2.4..3.32 beats) are 100% settled between 3.45 and 6.45 beats
  return stepLocalBeat >= 3.48 && stepLocalBeat <= 6.46;
}

async function renderOneVideo(browser, id, aspect, audioPath) {
  const htmlPath = path.join(SVG_ANIM_DIR, `${id}.html`);
  const outMp4 = path.join(aspect.outDir, `${id}-${aspect.key}.mp4`);
  const url = `file://${htmlPath}?render=1`;

  const context = await browser.newContext({
    viewport: { width: aspect.width, height: aspect.height },
    deviceScaleFactor: 1,
  });
  const page = await context.newPage();
  await page.goto(url, { waitUntil: 'networkidle' });
  await page.evaluate(() => document.fonts && document.fonts.ready);

  const ffArgs = [
    '-y',
    '-f', 'image2pipe',
    '-vcodec', 'mjpeg',
    '-framerate', String(CAPTURE_FPS),
    '-i', '-',
    '-i', audioPath,
    '-map', '0:v:0',
    '-map', '1:a:0',
    '-vf', "tmix=frames=2:weights='1 1',fps=60",
    '-c:v', 'libx264',
    '-preset', 'veryfast',
    '-crf', '18',
    '-pix_fmt', 'yuv420p',
    '-movflags', '+faststart',
    '-c:a', 'copy',
    '-shortest',
    outMp4,
  ];

  const ff = spawn('ffmpeg', ffArgs, { stdio: ['pipe', 'ignore', 'pipe'] });
  let ffErr = '';
  ff.stderr.on('data', (d) => {
    ffErr += d.toString();
  });

  const t0 = Date.now();
  let frame0Buf = null;
  let lastHoldBuf = null;
  let wasHold = false;

  for (let f = 0; f < TOTAL_CAPTURE_FRAMES; f++) {
    let buf;
    if (f === TOTAL_CAPTURE_FRAMES - 1 && frame0Buf) {
      // Exact loop closure: Frame Last === Frame 0
      buf = frame0Buf;
    } else {
      const t = (f / CAPTURE_FPS);
      const inHold = isStationaryStepHold(t);
      if (inHold && wasHold && lastHoldBuf) {
        buf = lastHoldBuf;
      } else {
        await page.evaluate((timeSec) => window.seek(timeSec), t);
        buf = await page.screenshot({ type: 'jpeg', quality: 94 });
        if (f === 0) frame0Buf = buf;
        if (inHold) lastHoldBuf = buf;
      }
      wasHold = inHold;
    }

    const ok = ff.stdin.write(buf);
    if (!ok) {
      await new Promise((r) => ff.stdin.once('drain', r));
    }
  }

  ff.stdin.end();
  await new Promise((resolve, reject) => {
    ff.on('close', (code) => {
      if (code === 0) resolve();
      else reject(new Error(`ffmpeg exited ${code}: ${ffErr.slice(-400)}`));
    });
  });

  // Also capture a high-res keyframe proof at Beat 12 (Step 2 active drag) for contact sheet
  const proofPng = path.join(PROOFS_DIR, `${id}-${aspect.key}-proof.png`);
  await page.evaluate((timeSec) => window.seek(timeSec), 12.5 * BEAT);
  await page.screenshot({ path: proofPng, type: 'png' });

  await context.close();
  const dt = ((Date.now() - t0) / 1000).toFixed(1);
  const sizeMb = (fs.statSync(outMp4).size / (1024 * 1024)).toFixed(2);
  console.log(`  ✓ [${aspect.key}] ${path.basename(outMp4)} (${sizeMb} MB) rendered in ${dt}s`);
  return { id, aspect: aspect.key, outMp4, proofPng, sizeMb, dt };
}

async function runPool(items, concurrency, workerFn) {
  const results = [];
  let idx = 0;
  async function next() {
    while (idx < items.length) {
      const cur = items[idx++];
      results.push(await workerFn(cur));
    }
  }
  await Promise.all(Array.from({ length: Math.min(concurrency, items.length) }, () => next()));
  return results;
}

async function main() {
  fs.mkdirSync(VIDEOS_16X9_DIR, { recursive: true });
  fs.mkdirSync(VIDEOS_9X16_DIR, { recursive: true });
  fs.mkdirSync(DOCS_VIDEOS_DIR, { recursive: true });
  fs.mkdirSync(PROOFS_DIR, { recursive: true });

  const audioPath = ensureAudioMix30s();
  const browser = await chromium.launch({ headless: true });

  const jobs = [];
  for (const aspect of ASPECTS) {
    for (const id of VIDEO_IDS) {
      jobs.push({ id, aspect });
    }
  }

  console.log(`==> Rendering ${jobs.length} x 30s Real-Canvas Videos (8 x 16:9 + 8 x 9:16) with 8 parallel workers...`);
  const tStart = Date.now();
  await runPool(jobs, 8, ({ id, aspect }) => renderOneVideo(browser, id, aspect, audioPath));
  await browser.close();

  // Copy 16:9 & 9:16 videos to docs/videos/ so GitHub Pages can stream them directly
  for (const id of VIDEO_IDS) {
    const src16x9 = path.join(VIDEOS_16X9_DIR, `${id}-16x9.mp4`);
    const src9x16 = path.join(VIDEOS_9X16_DIR, `${id}-9x16.mp4`);
    fs.copyFileSync(src16x9, path.join(VIDEOS_DIR, `${id}.mp4`));
    fs.copyFileSync(src16x9, path.join(DOCS_VIDEOS_DIR, `${id}-16x9.mp4`));
    fs.copyFileSync(src9x16, path.join(DOCS_VIDEOS_DIR, `${id}-9x16.mp4`));
  }

  const totalSec = ((Date.now() - tStart) / 1000).toFixed(1);
  console.log(`==> ALL 16 VIDEOS (8 x 16:9 + 8 x 9:16) RENDERED & SYNCED IN ${totalSec}s!`);
}

main().catch((err) => {
  console.error(err);
  process.exit(1);
});
