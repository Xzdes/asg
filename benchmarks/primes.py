#!/usr/bin/env python3
"""Prime Sieve Benchmark - Python"""

import time

def is_prime(n):
    if n <= 1:
        return False
    if n == 2:
        return True
    if n % 2 == 0:
        return False
    divisor = 3
    while divisor * divisor <= n:
        if n % divisor == 0:
            return False
        divisor += 2
    return True

def count_primes(limit):
    count = 0
    for n in range(2, limit + 1):
        if is_prime(n):
            count += 1
    return count

if __name__ == "__main__":
    print("=== Prime Sieve Benchmark (Python) ===")
    N = 10_000

    start = time.time()
    result = count_primes(N)
    end = time.time()

    print(f"Primes up to {N}: {result}")
    print(f"Time: {end - start:.3f}s")
