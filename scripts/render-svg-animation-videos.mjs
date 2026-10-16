#!/usr/bin/env node
/**
 * Renders Design OS Studio's 3 Apple-Grade SF Pro Rounded + Liquid Glass UI Motion videos:
 *  1. 28-beat Contact Sheets (7x4 grid, 1 frame per beat) + exact Loop Verification (Frame 0 === Frame Last)
 *  2. Full 60fps 1440x1440 render with 4 subframes per frame (180-deg shutter) blended via ffmpeg tmix=frames=4
 *     using 8 parallel Chromium browser workers for ~4x faster rendering.
 */
import { createRequire } from 'node:module';
import { spawn, execFileSync } from 'node:child_process';
import fs from 'node:fs';
import os from 'node:os';
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
const WORKERS = Number(process.env.WORKERS || 8);

const CHROME_ARGS = [
  '--no-sandbox',
  '--disable-setuid-sandbox',
  '--hide-scrollbars',
  '--disable-gpu-vsync',
  '--font-render-hinting=none',
  '--disable-background-timer-throttling',
  '--disable-backgrounding-occluded-windows',
  '--disable-renderer-backgrounding',
  '--enable-gpu-rasterization',
  '--ignore-gpu-blocklist',
];

const VIDEOS = [
  '01-designos-studio-launch-hero',
  '02-figma-100-micro-interactions',
  '03-agent-first-specs-and-devmode',
];

async function launchBrowser() {
  return puppeteer.launch({
    executablePath: CHROME_PATH,
    headless: 'new',
    protocolTimeout: 180000,
    args: CHROME_ARGS,
  });
}

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
    '-vf', 'scale=360:360,tile=7x4:margin=8:padding=6:color=0x1D1D1F',
    '-frames:v', '1',
    contactSheetPath,
  ], { stdio: 'ignore' });

  console.log(`[${id}] Saved 28-beat contact sheet -> ${contactSheetPath}`);
  await page.close();
}

async function renderVideoChunk(htmlPath, startFrame, endFrame, chunkPath) {
  const browser = await launchBrowser();
  try {
    const page = await browser.newPage();
    await page.setViewport({ width: WIDTH, height: HEIGHT, deviceScaleFactor: 1 });
    await page.goto(`file://${htmlPath}?render=1`, { waitUntil: 'load', timeout: 30000 });
    await page.evaluate(() => window.__readyPromise);

    const vf = `tmix=frames=${SUBFRAMES}:weights='1 1 1 1',select='eq(mod(n\\,${SUBFRAMES})\\,${SUBFRAMES - 1})',setpts=N/${FPS}/TB`;
    const ffArgs = [
      '-y',
      '-v', 'error',
      '-f', 'image2pipe',
      '-vcodec', 'mjpeg',
      '-framerate', String(FPS * SUBFRAMES),
      '-i', '-',
      '-vf', vf,
      '-r', String(FPS),
      '-c:v', 'libx264',
      '-profile:v', 'high',
      '-level', '4.2',
      '-preset', 'fast',
      '-crf', '16',
      '-pix_fmt', 'yuv420p',
      '-an',
      chunkPath,
    ];

    const ff = spawn('ffmpeg', ffArgs, { stdio: ['pipe', 'ignore', 'pipe'] });
    let errLog = '';
    ff.stderr.on('data', (c) => { errLog = (errLog + c).slice(-2000); });
    const ffDone = new Promise((resolve, reject) => {
      ff.once('close', (code) => (code === 0 ? resolve() : reject(new Error(`ffmpeg chunk exit ${code}: ${errLog}`))));
    });

    for (let f = startFrame; f < endFrame; f++) {
      for (let s = 0; s < SUBFRAMES; s++) {
        let tSec;
        if (f === 0) {
          tSec = 0;
        } else if (f === TOTAL_FRAMES - 1) {
          tSec = DURATION;
        } else {
          const dt = (s / SUBFRAMES) * (SHUTTER / FPS);
          tSec = Math.min(DURATION - 1e-3, f / FPS + dt);
        }
        await page.evaluate((t) => {
          window.seek(t);
          void document.body.offsetHeight;
        }, tSec);
        const buf = await page.screenshot({ type: 'jpeg', quality: 95, optimizeForSpeed: true });
        if (!ff.stdin.write(buf)) {
          await new Promise((r) => ff.stdin.once('drain', r));
        }
      }
    }

    ff.stdin.end();
    await ffDone;
  } finally {
    await browser.close();
  }
}

async function renderFullVideo(id) {
  const htmlPath = path.join(SVG_DIR, `${id}.html`);
  const wavPath = path.join(AUDIO_DIR, `${id}-master.wav`);
  const outMp4 = path.join(VIDEOS_DIR, `${id}.mp4`);

  const tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), `designos-render-${id}-`));
  const t0 = Date.now();
  const framesPerWorker = Math.ceil(TOTAL_FRAMES / WORKERS);
  const chunkPaths = [];
  const tasks = [];

  for (let w = 0; w < WORKERS; w++) {
    const startFrame = w * framesPerWorker;
    const endFrame = Math.min(TOTAL_FRAMES, (w + 1) * framesPerWorker);
    if (startFrame >= endFrame) break;
    const chunkPath = path.join(tmpDir, `chunk-${String(w).padStart(2, '0')}.mp4`);
    chunkPaths.push(chunkPath);
    tasks.push(renderVideoChunk(htmlPath, startFrame, endFrame, chunkPath));
  }

  console.log(`[${id}] Rendering ${TOTAL_FRAMES} frames (${TOTAL_FRAMES * SUBFRAMES} subframes) across ${tasks.length} parallel Chromium workers...`);
  await Promise.all(tasks);

  const concatListPath = path.join(tmpDir, 'concat.txt');
  fs.writeFileSync(
    concatListPath,
    chunkPaths.map((p) => `file '${p}'`).join('\n') + '\n',
    'utf8'
  );

  execFileSync('ffmpeg', [
    '-y',
    '-v', 'error',
    '-f', 'concat',
    '-safe', '0',
    '-i', concatListPath,
    '-i', wavPath,
    '-c:v', 'copy',
    '-c:a', 'aac',
    '-b:a', '192k',
    '-shortest',
    '-movflags', '+faststart',
    outMp4,
  ]);

  fs.rmSync(tmpDir, { recursive: true, force: true });
  const stat = fs.statSync(outMp4);
  const elapsed = ((Date.now() - t0) / 1000).toFixed(1);
  console.log(`[${id}] Saved 60fps 4x-tmix MP4 -> ${outMp4} (${(stat.size / 1024 / 1024).toFixed(2)} MB) in ${elapsed}s`);
}

async function main() {
  const args = process.argv.slice(2);
  const beatsOnly = args.includes('--beats-only');
  const fullOnly = args.includes('--full-only');

  if (!fullOnly) {
    const browser = await launchBrowser();
    try {
      for (const id of VIDEOS) {
        await renderBeatsAndVerifyLoop(browser, id);
      }
    } finally {
      await browser.close();
    }
  }

  if (!beatsOnly) {
    for (const id of VIDEOS) {
      await renderFullVideo(id);
    }
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
}

main().catch((err) => {
  console.error(err);
  process.exit(1);
});
