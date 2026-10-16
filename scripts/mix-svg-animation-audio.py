#!/usr/bin/env python3
"""
Mixes the 3 peak-aligned master audio tracks for Design OS Studio Launch Videos (1440x1440 @ 60fps).
- Song: /Users/jangtrinh/Downloads/Aetheric - Edge of Motion (freetouse.com).mp3
- Tempo: 100.60 BPM -> 28 beats (7 bars) = 16.70s (BEAT = 0.5964285714285714s)
- SFX placed by measured peak: t_start = max(0, beat * BEAT - peak_sec)
"""
import os
import subprocess
import numpy as np
import wave

SR = 44100
DURATION = 16.70
BEAT = DURATION / 28.0
N_SAMPLES = int(round(DURATION * SR))

SONG_PATH = "/Users/jangtrinh/Downloads/Aetheric - Edge of Motion (freetouse.com).mp3"
SFX_DIR = "/Users/jangtrinh/Products/design-os-svg-animation/promo/nowa-launch/audio/sfx"
OUT_DIR = "/Users/jangtrinh/Products/design-os-studio/Design/marketing/svg-animation/audio"

os.makedirs(OUT_DIR, exist_ok=True)


def load_audio_stereo(path: str) -> np.ndarray:
    cmd = [
        "ffmpeg", "-v", "error", "-i", path,
        "-f", "f32le", "-ac", "2", "-ar", str(SR), "pipe:1"
    ]
    raw = subprocess.check_output(cmd)
    arr = np.frombuffer(raw, dtype=np.float32).reshape(-1, 2)
    return arr.copy()


def measure_peak_sec(arr: np.ndarray) -> float:
    mono = np.abs(arr).mean(axis=1)
    idx = int(np.argmax(mono))
    return idx / SR


def save_wav(path: str, stereo: np.ndarray):
    clipped = np.clip(stereo, -0.98, 0.98)
    pcm16 = (clipped * 32767.0).astype(np.int16)
    with wave.open(path, "wb") as wf:
        wf.setnchannels(2)
        wf.setsampwidth(2)
        wf.setframerate(SR)
        wf.writeframes(pcm16.tobytes())


def main():
    print("Loading song:", SONG_PATH)
    song = load_audio_stereo(SONG_PATH)

    sfx_files = {
        "click_ui": "click-ui-2568.mp3",
        "pop_spring": "pop-spring-2358.mp3",
        "pop_clean": "pop-clean-2354.mp3",
        "click_soft": "click-soft-1109.mp3",
        "whoosh_fast": "whoosh-fast-1491.mp3",
        "chime_delivered": "chime-delivered-2869.mp3",
        "chime_order": "chime-order-2870.mp3",
    }
    sfx_data = {}
    sfx_peaks = {}
    for key, fname in sfx_files.items():
        arr = load_audio_stereo(os.path.join(SFX_DIR, fname))
        peak = measure_peak_sec(arr)
        sfx_data[key] = arr
        sfx_peaks[key] = peak
        print(f"  SFX {key:16s} ({fname}): peak = {peak*1000:.1f} ms")

    videos = [
        {
            "id": "01-designos-studio-launch-hero",
            "song_offset_sec": 14.896,
            "events": [
                (1.0, "click_ui", 0.50),
                (2.0, "pop_spring", 0.45),
                (4.0, "chime_order", 0.48),
                (6.0, "whoosh_fast", 0.35),
                (8.0, "pop_clean", 0.45),
                (9.0, "click_ui", 0.48),
                (10.0, "click_soft", 0.38),
                (11.0, "click_soft", 0.38),
                (12.0, "pop_spring", 0.44),
                (13.85, "pop_clean", 0.45),
                (15.0, "click_ui", 0.50),
                (17.0, "pop_spring", 0.44),
                (18.0, "click_soft", 0.42),
                (19.0, "click_soft", 0.42),
                (20.0, "whoosh_fast", 0.35),
                (22.0, "click_ui", 0.48),
                (23.2, "click_soft", 0.38),
                (23.8, "click_ui", 0.50),
                (24.0, "chime_delivered", 0.46),
                (26.85, "pop_spring", 0.42),
            ],
        },
        {
            "id": "02-figma-100-micro-interactions",
            "song_offset_sec": 17.281,
            "events": [
                (1.0, "click_ui", 0.50),
                (2.0, "pop_spring", 0.45),
                (4.0, "chime_order", 0.48),
                (6.0, "whoosh_fast", 0.35),
                (8.0, "pop_clean", 0.45),
                (9.0, "click_ui", 0.48),
                (10.0, "click_soft", 0.38),
                (11.0, "click_soft", 0.38),
                (12.0, "pop_spring", 0.44),
                (13.85, "pop_clean", 0.45),
                (15.0, "click_ui", 0.50),
                (17.0, "pop_spring", 0.44),
                (18.0, "click_soft", 0.42),
                (19.0, "click_soft", 0.42),
                (20.0, "whoosh_fast", 0.35),
                (22.0, "click_ui", 0.48),
                (23.2, "click_soft", 0.38),
                (23.8, "click_ui", 0.50),
                (24.0, "chime_delivered", 0.46),
                (26.85, "pop_spring", 0.42),
            ],
        },
        {
            "id": "03-agent-first-specs-and-devmode",
            "song_offset_sec": 5.358,
            "events": [
                (1.0, "click_ui", 0.50),
                (2.0, "pop_spring", 0.45),
                (4.0, "chime_order", 0.48),
                (6.0, "whoosh_fast", 0.35),
                (8.0, "pop_clean", 0.45),
                (9.0, "click_ui", 0.48),
                (10.0, "click_soft", 0.38),
                (11.0, "click_soft", 0.38),
                (12.0, "pop_spring", 0.44),
                (13.85, "pop_clean", 0.45),
                (15.0, "click_ui", 0.50),
                (17.0, "pop_spring", 0.44),
                (18.0, "click_soft", 0.42),
                (19.0, "click_soft", 0.42),
                (20.0, "whoosh_fast", 0.35),
                (22.0, "click_ui", 0.48),
                (23.2, "click_soft", 0.38),
                (23.8, "click_ui", 0.50),
                (24.0, "chime_delivered", 0.46),
                (26.85, "pop_spring", 0.42),
            ],
        },
    ]

    for vid in videos:
        start_idx = int(round(vid["song_offset_sec"] * SR))
        track = song[start_idx : start_idx + N_SAMPLES].copy() * 0.72

        # Smooth 80ms fade in & 220ms fade out for clean looping
        fade_in = int(0.08 * SR)
        fade_out = int(0.22 * SR)
        track[:fade_in] *= np.linspace(0.0, 1.0, fade_in)[:, None]
        track[-fade_out:] *= np.linspace(1.0, 0.0, fade_out)[:, None]

        # Mix peak-aligned SFX
        for beat_idx, sfx_key, gain in vid["events"]:
            target_sec = beat_idx * BEAT
            start_sec = target_sec - sfx_peaks[sfx_key]
            s_idx = max(0, int(round(start_sec * SR)))
            sfx_arr = sfx_data[sfx_key] * gain
            end_idx = min(N_SAMPLES, s_idx + len(sfx_arr))
            if end_idx > s_idx:
                track[s_idx:end_idx] += sfx_arr[: end_idx - s_idx]

        out_wav = os.path.join(OUT_DIR, f"{vid['id']}-master.wav")
        save_wav(out_wav, track)
        print(f"Saved master audio: {out_wav} ({DURATION:.2f}s, {len( vid['events'] )} peak-aligned SFX)")


if __name__ == "__main__":
    main()
