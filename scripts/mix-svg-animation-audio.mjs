#!/usr/bin/env node
/**
 * Pure Node.js Float32Array audio & peak-aligned SFX mixer for Design OS Studio Launch Videos.
 * - Song: /Users/jangtrinh/Downloads/Aetheric - Edge of Motion (freetouse.com).mp3
 * - Tempo: 100.60 BPM -> 28 beats (7 bars) = 16.70s (BEAT = 0.5964285714285714s)
 * - Places every Mixkit UI sound by its measured transient peak: t_start = beat * BEAT - peak_sec
 */
import { execFileSync } from 'node:child_process';
import fs from 'node:fs';
import path from 'node:path';

const SR = 44100;
const CHANNELS = 2;
const DURATION = 16.70;
const BEAT = DURATION / 28.0;
const FRAMES = Math.round(DURATION * SR);

const SONG_PATH = '/Users/jangtrinh/Downloads/Aetheric - Edge of Motion (freetouse.com).mp3';
const SFX_DIR = '/Users/jangtrinh/Products/design-os-svg-animation/promo/nowa-launch/audio/sfx';
const OUT_DIR = '/Users/jangtrinh/Products/design-os-studio/Design/marketing/svg-animation/audio';

fs.mkdirSync(OUT_DIR, { recursive: true });

function decodeStereoF32(filePath) {
  const buf = execFileSync('ffmpeg', [
    '-v', 'error',
    '-i', filePath,
    '-f', 'f32le',
    '-ac', String(CHANNELS),
    '-ar', String(SR),
    'pipe:1',
  ], { maxBuffer: 128 * 1024 * 1024 });
  return new Float32Array(buf.buffer, buf.byteOffset, Math.floor(buf.byteLength / 4));
}

function measurePeakSec(interleaved) {
  const totalFrames = Math.floor(interleaved.length / 2);
  let maxVal = -1;
  let maxFrame = 0;
  for (let i = 0; i < totalFrames; i++) {
    const amp = 0.5 * (Math.abs(interleaved[i * 2]) + Math.abs(interleaved[i * 2 + 1]));
    if (amp > maxVal) {
      maxVal = amp;
      maxFrame = i;
    }
  }
  return maxFrame / SR;
}

function writeWav16(filePath, interleaved) {
  const numSamples = interleaved.length;
  const dataBytes = numSamples * 2;
  const header = Buffer.alloc(44);
  header.write('RIFF', 0);
  header.writeUInt32LE(36 + dataBytes, 4);
  header.write('WAVE', 8);
  header.write('fmt ', 12);
  header.writeUInt32LE(16, 16); // PCM chunk size
  header.writeUInt16LE(1, 20);  // PCM format
  header.writeUInt16LE(CHANNELS, 22);
  header.writeUInt32LE(SR, 24);
  header.writeUInt32LE(SR * CHANNELS * 2, 28);
  header.writeUInt16LE(CHANNELS * 2, 32);
  header.writeUInt16LE(16, 34);
  header.write('data', 36);
  header.writeUInt32LE(dataBytes, 40);

  const pcm = Buffer.alloc(dataBytes);
  for (let i = 0; i < numSamples; i++) {
    const clamped = Math.max(-0.98, Math.min(0.98, interleaved[i]));
    pcm.writeInt16LE(Math.round(clamped * 32767), i * 2);
  }
  fs.writeFileSync(filePath, Buffer.concat([header, pcm]));
}

console.log('Decoding song:', SONG_PATH);
const song = decodeStereoF32(SONG_PATH);

const sfxFiles = {
  click_ui: 'click-ui-2568.mp3',
  pop_spring: 'pop-spring-2358.mp3',
  pop_clean: 'pop-clean-2354.mp3',
  click_soft: 'click-soft-1109.mp3',
  whoosh_fast: 'whoosh-fast-1491.mp3',
  chime_delivered: 'chime-delivered-2869.mp3',
  chime_order: 'chime-order-2870.mp3',
};

const sfxData = {};
const sfxPeaks = {};
for (const [key, fname] of Object.entries(sfxFiles)) {
  const arr = decodeStereoF32(path.join(SFX_DIR, fname));
  const peak = measurePeakSec(arr);
  sfxData[key] = arr;
  sfxPeaks[key] = peak;
  console.log(`  SFX ${key.padEnd(16)} (${fname}): peak = ${(peak * 1000).toFixed(1)} ms`);
}

const commonEvents = [
  [1.0, 'click_ui', 0.50],
  [2.0, 'pop_spring', 0.45],
  [4.0, 'chime_order', 0.48],
  [6.0, 'whoosh_fast', 0.35],
  [8.0, 'pop_clean', 0.45],
  [9.0, 'click_ui', 0.48],
  [10.0, 'click_soft', 0.38],
  [11.0, 'click_soft', 0.38],
  [12.0, 'pop_spring', 0.44],
  [13.85, 'pop_clean', 0.45],
  [15.0, 'click_ui', 0.50],
  [17.0, 'pop_spring', 0.44],
  [18.0, 'click_soft', 0.42],
  [19.0, 'click_soft', 0.42],
  [20.0, 'whoosh_fast', 0.35],
  [22.0, 'click_ui', 0.48],
  [23.2, 'click_soft', 0.38],
  [23.8, 'click_ui', 0.50],
  [24.0, 'chime_delivered', 0.46],
  [26.85, 'pop_spring', 0.42],
];

const videos = [
  { id: '01-designos-studio-launch-hero', songOffsetSec: 14.896, events: commonEvents },
  { id: '02-figma-100-micro-interactions', songOffsetSec: 17.281, events: commonEvents },
  { id: '03-agent-first-specs-and-devmode', songOffsetSec: 5.358, events: commonEvents },
];

for (const vid of videos) {
  const startFrame = Math.round(vid.songOffsetSec * SR);
  const out = new Float32Array(FRAMES * 2);

  const fadeInFrames = Math.round(0.08 * SR);
  const fadeOutFrames = Math.round(0.22 * SR);

  for (let f = 0; f < FRAMES; f++) {
    let env = 0.72;
    if (f < fadeInFrames) env *= f / fadeInFrames;
    else if (f > FRAMES - fadeOutFrames) env *= (FRAMES - f) / fadeOutFrames;

    const srcIdx = (startFrame + f) * 2;
    out[f * 2] = (song[srcIdx] || 0) * env;
    out[f * 2 + 1] = (song[srcIdx + 1] || 0) * env;
  }

  for (const [beatIdx, sfxKey, gain] of vid.events) {
    const targetSec = beatIdx * BEAT;
    const startSec = targetSec - sfxPeaks[sfxKey];
    const sFrame = Math.max(0, Math.round(startSec * SR));
    const sfx = sfxData[sfxKey];
    const sfxFrames = Math.floor(sfx.length / 2);
    for (let i = 0; i < sfxFrames && sFrame + i < FRAMES; i++) {
      out[(sFrame + i) * 2] += sfx[i * 2] * gain;
      out[(sFrame + i) * 2 + 1] += sfx[i * 2 + 1] * gain;
    }
  }

  const outWav = path.join(OUT_DIR, `${vid.id}-master.wav`);
  writeWav16(outWav, out);
  console.log(`Saved peak-aligned master audio -> ${outWav}`);
}
