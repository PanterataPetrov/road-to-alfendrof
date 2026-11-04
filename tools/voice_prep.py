"""Cleans up a folder of voice recordings and turns them into game sounds.

    python tools/voice_prep.py <in folder> <out folder> <prefix> [--bass 0.2] [--target -18]

For every wav: noise reduction (a spectral gate), a bass boost, the crunchy 16-bit treatment of tools/chiptune.py,
then loudness normalization (tools/loudness.py). The result is written to <out folder> as `<prefix>_<name>_<n>.wav`
(the name's digit, counted from 1 in the recording, comes out counted from 0, as the game's groups are).
"""

import re
import sys
from pathlib import Path

import numpy as np
from scipy.io import wavfile
from scipy.signal import butter, istft, sosfiltfilt, stft

sys.path.insert(0, str(Path(__file__).resolve().parent))
import chiptune  # noqa: E402
import loudness  # noqa: E402


def denoise(x, sr, reduction=1.8, floor=0.08):
    """Spectral gate: the noise in each frequency is its level in the quietest frames of the recording; subtract
    `reduction` times that from every frame, never going below `floor` of the original, and smooth the gate over
    neighbouring bins and frames so it doesn't warble."""
    f, t, z = stft(x, sr, nperseg=1024, noverlap=768)
    mag, phase = np.abs(z), np.angle(z)
    # the noise estimate: the 15th percentile over time of each bin (speech and hits are mostly above it)
    noise = np.percentile(mag, 15, axis=1, keepdims=True)
    gain = np.maximum(mag - reduction * noise, floor * mag) / np.maximum(mag, 1e-9)
    kernel = np.ones((3, 3)) / 9
    from scipy.signal import convolve2d

    gain = convolve2d(gain, kernel, mode="same", boundary="symm")
    _, y = istft(mag * gain * np.exp(1j * phase), sr, nperseg=1024, noverlap=768)
    return y[: len(x)].astype(np.float32)


def bass_boost(x, sr, amount=0.2, corner=220):
    """Add `amount` of the low end (below `corner` Hz) back on top of the sound: 0.2 is 20 % more bass."""
    low = sosfiltfilt(butter(2, corner, "lowpass", fs=sr, output="sos"), x)
    return x + amount * low


def main():
    src, dst, prefix = Path(sys.argv[1]), Path(sys.argv[2]), sys.argv[3]
    bass = float(sys.argv[sys.argv.index("--bass") + 1]) if "--bass" in sys.argv else 0.2
    target = float(sys.argv[sys.argv.index("--target") + 1]) if "--target" in sys.argv else -18.0
    dst.mkdir(parents=True, exist_ok=True)
    for f in sorted(src.glob("*.wav")):
        m = re.match(r"^(.*?)[ _]*(\d+)$", f.stem)
        name, number = (m.group(1), int(m.group(2))) if m else (f.stem, 1)
        name = re.sub(r"[^a-z0-9]+", "_", name.lower()).strip("_")
        sr, x = chiptune.load(f)
        x = denoise(x, sr)
        x = bass_boost(x, sr, bass)
        y = chiptune.chiptune(x, sr, **chiptune.STYLES["crunch"])
        y = loudness.apply_gain(y, target - loudness.loudness_db(y, chiptune.OUT_RATE))
        out = dst / f"{prefix}_{name}_{number - 1}.wav"
        wavfile.write(out, chiptune.OUT_RATE, np.round(np.clip(y, -1, 1) * 32767).astype(np.int16))
        print(f"{out.name:36s} {len(y) / chiptune.OUT_RATE:4.2f} s")


if __name__ == "__main__":
    main()
