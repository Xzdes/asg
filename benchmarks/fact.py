#!/usr/bin/env python3
"""Factorial Benchmark - Python"""

import time

def factorial(n):
    if n <= 1:
        return 1
    return n * factorial(n - 1)

if __name__ == "__main__":
    print("=== Factorial Benchmark (Python) ===")
    iterations = 100_000

    start = time.time()
    result = 0
    for _ in range(iterations):
        result = factorial(20)
    end = time.time()

    print(f"factorial(20) x {iterations} = {result}")
    print(f"Time: {end - start:.3f}s")
