"""Measures and evens out the loudness of sound effects.

    python tools/loudness.py measure <folder>
    python tools/loudness.py normalize <folder> [--target -20]

Loudness is the RMS of the parts of a sound that are actually playing (the 50 ms blocks within 35 dB of the
loudest block), so a short hit and a long cry are judged by how loud they are while they sound, not by their silence.
`normalize` gives every wav that loudness and, where that would push a peak over 0.95, rounds the peak off smoothly
instead of letting it clip. The files are rewritten in place.
"""

import sys
from pathlib import Path

import numpy as np
from scipy.io import wavfile

PEAK = 0.95


def read(path):
    sr, x = wavfile.read(path)
    x = x.astype(np.float32) / 32768.0
    return sr, x


def loudness_db(x, sr):
    mono = x.mean(axis=1) if x.ndim == 2 else x
    block = max(1, sr // 20)
    n = len(mono) // block
    if n == 0:
        return -90.0
    rms = np.sqrt((mono[: n * block].reshape(n, block) ** 2).mean(axis=1))
    live = rms[rms > rms.max() * 10 ** (-35 / 20)]
    return 20 * np.log10(np.sqrt((live ** 2).mean()) + 1e-9)


def apply_gain(x, db):
    y = x * 10 ** (db / 20)
    over = np.abs(y).max()
    if over > PEAK:
        # round the peaks off (a soft knee from 70 % of the ceiling) rather than clipping them flat
        knee = PEAK * 0.7
        a = np.abs(y)
        y = np.where(a > knee, np.sign(y) * (knee + (PEAK - knee) * np.tanh((a - knee) / (PEAK - knee))), y)
    return y


def main():
    mode, folder = sys.argv[1], Path(sys.argv[2])
    files = sorted(folder.glob("*.wav"))
    target = float(sys.argv[sys.argv.index("--target") + 1]) if "--target" in sys.argv else -20.0
    for f in files:
        sr, x = read(f)
        db = loudness_db(x, sr)
        peak = np.abs(x).max()
        if mode == "measure":
            print(f"{f.name:32s} {db:6.1f} dB   peak {peak:.2f}")
        else:
            y = apply_gain(x, target - db)
            wavfile.write(f, sr, np.round(np.clip(y, -1, 1) * 32767).astype(np.int16))
            print(f"{f.name:32s} {db:6.1f} -> {loudness_db(y, sr):6.1f} dB   peak {peak:.2f} -> {np.abs(y).max():.2f}")


if __name__ == "__main__":
    main()
