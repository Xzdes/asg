#!/usr/bin/env python3
"""Sum Benchmark - Python"""

import time

def sum_to_n(n):
    total = 0
    for i in range(1, n + 1):
        total += i
    return total

if __name__ == "__main__":
    print("=== Sum Benchmark (Python) ===")
    N = 1_000_000

    start = time.time()
    result = sum_to_n(N)
    end = time.time()

    print(f"sum(1..{N}) = {result}")
    print(f"Time: {end - start:.3f}s")
