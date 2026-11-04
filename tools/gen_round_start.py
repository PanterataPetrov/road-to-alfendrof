"""The round-start sound: a deep drum hit under a struck gong. Writes assets/sfx/round_start.wav."""
import wave
import numpy as np

SR = 44100
t = np.arange(int(SR * 1.3)) / SR
rng = np.random.default_rng(7)

# the gong: inharmonic partials, each ringing out at its own speed, with a slight wobble
gong = np.zeros_like(t)
for ratio, amp, decay in [(1.0, 1.0, 2.6), (1.51, 0.7, 3.2), (2.09, 0.55, 3.8), (2.76, 0.4, 4.6), (3.42, 0.25, 6.0), (4.8, 0.15, 8.0)]:
    f = 150 * ratio
    wobble = 1 + 0.004 * np.sin(2 * np.pi * 5.5 * t)
    gong += amp * np.exp(-decay * t) * np.sin(2 * np.pi * f * t * wobble)
# the strike: a burst of noise right at the start
gong += 1.4 * rng.standard_normal(len(t)) * np.exp(-90 * t)

# the drum under it: a sine dropping in pitch
sweep = 110 * np.exp(-9 * t) + 45
drum = np.sin(2 * np.pi * np.cumsum(sweep) / SR) * np.exp(-6 * t) * 1.8

x = gong * 0.35 + drum
x *= np.minimum(1, t / 0.002)
x *= np.minimum(1, (t[-1] - t) / 0.15)
x = np.tanh(x * 0.9)
x = x / np.abs(x).max() * 0.85
with wave.open("assets/sfx/round_start.wav", "wb") as w:
    w.setnchannels(1)
    w.setsampwidth(2)
    w.setframerate(SR)
    w.writeframes((x * 32767).astype("<i2").tobytes())
