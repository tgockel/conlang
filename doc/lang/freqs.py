#!/usr/bin/env python3
"""
Program to show the frequencies for a given turnout of the curve (in ``frequencies.md``).
"""

import math

def fi(x: float, a: float = 0.0) -> float:
    # Integral of `cos x + a`:
    return math.sin(x) + a * x

a = 0
src = "pbɣʂʐɟkgqʕβt"
diff = math.pi / (2 * len(src))
total = fi(math.pi / 2, a) - fi(0, a)
sum = 0
for (idx, sound) in enumerate(src):
    moment = fi((idx + 1) * diff, a) - fi(idx * diff, a)
    moment /= total
    sum += moment
    print(f"{sound} -> {moment * 100:.1f}%")

# Error check -- should add up to 1
print(f"{sum}")
