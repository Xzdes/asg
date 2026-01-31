#!/usr/bin/env python3
"""Fibonacci Benchmark - Python"""

import time

def fib(n):
    if n <= 1:
        return n
    return fib(n - 1) + fib(n - 2)

if __name__ == "__main__":
    print("=== Fibonacci Benchmark (Python) ===")

    start = time.time()
    result = fib(35)
    end = time.time()

    print(f"fib(35) = {result}")
    print(f"Time: {end - start:.3f}s")
