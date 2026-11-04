"""Makes a sound effect sound like it came off a 16-bit console.

    python tools/chiptune.py "in.wav" "out.wav" [--style snes|crunch|crunch-bright]

snes:   32 kHz out of a 16 kHz sample (the way the SNES stored voices), 8-bit steps, a band-limited, slightly
        nasal tone, and a short echo tail.
crunch: the same, rougher: 11 kHz and 6-bit steps, like a Genesis voice sample, with the fizz filtered off the top.
crunch-bright: the same without that last filter (scratchier).

Both take the left and right channels together (the game's sounds are mono), keep the loudness of the original
and write a 16-bit mono wav, which every player reads.
"""

import sys
from pathlib import Path

import numpy as np
from scipy.io import wavfile
from scipy.signal import butter, sosfilt, sosfiltfilt

STYLES = {
    # sample rate of the "chip", bits per sample, top of the tone, echo (delay s, gain)
    "snes": dict(rate=16000, bits=8, top=6500, echo=(0.045, 0.22)),
    "crunch-bright": dict(rate=11025, bits=6, top=4800, echo=(0.06, 0.12)),
    # the same, then a gentle low-pass over the result, which takes the fizz off the top and keeps the stair-steps
    "crunch": dict(rate=11025, bits=6, top=4800, echo=(0.06, 0.12), post=4000),
}
OUT_RATE = 32000


def load(path):
    sr, x = wavfile.read(path)
    x = x.astype(np.float32)
    if x.dtype != np.float32 or np.abs(x).max() > 1.5:
        x /= 32768.0
    return sr, (x.mean(axis=1) if x.ndim == 2 else x)


def resample_hold(x, sr, rate):
    """Take a sample every 1/rate seconds and hold it (no smoothing), so the high notes fold back as the grit a
    real sampler had."""
    step = sr / rate
    idx = np.floor(np.arange(0, len(x), step)).astype(int)
    return x[idx]


def chiptune(x, sr, rate, bits, top, echo, post=None):
    peak_in = np.abs(x).max()
    # the console's voice was not full-range: keep the middle, drop the rumble and the air
    x = sosfiltfilt(butter(2, 90, "highpass", fs=sr, output="sos"), x)
    x = sosfiltfilt(butter(4, top, "lowpass", fs=sr, output="sos"), x)
    # a little squash, so the quiet parts survive the coarse steps
    x = np.tanh(x * 1.6) / np.tanh(1.6)
    x = resample_hold(x, sr, rate)
    # coarse steps (round to `bits`), with no dither: the stair-steps are the sound
    levels = 2 ** (bits - 1) - 1
    x = np.round(x / max(np.abs(x).max(), 1e-6) * levels) / levels
    # up to the output rate by holding samples again
    x = np.repeat(x, int(round(OUT_RATE / rate)))
    # echo: one repeat, like the console's echo unit, quantized the same way
    delay, gain = echo
    d = int(delay * OUT_RATE)
    tail = np.zeros(len(x) + d * 3, dtype=np.float32)
    tail[: len(x)] += x
    tail[d : d + len(x)] += x * gain
    tail[d * 2 : d * 2 + len(x)] += x * gain * gain
    if post:
        tail = sosfilt(butter(2, post, "lowpass", fs=OUT_RATE, output="sos"), tail)
    # keep the original's loudest moment as the loudest moment
    tail = tail / max(np.abs(tail).max(), 1e-6) * min(peak_in, 0.9)
    return tail


def main():
    src, dst = Path(sys.argv[1]), Path(sys.argv[2])
    style = sys.argv[sys.argv.index("--style") + 1] if "--style" in sys.argv else "snes"
    sr, x = load(src)
    y = chiptune(x, sr, **STYLES[style])
    wavfile.write(dst, OUT_RATE, np.round(np.clip(y, -1, 1) * 32767).astype(np.int16))
    print(f"{dst.name}: {len(y) / OUT_RATE:.2f} s, {style}")


if __name__ == "__main__":
    main()
