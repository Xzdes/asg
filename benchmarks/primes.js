// Prime Sieve Benchmark - Node.js

function isPrime(n) {
    if (n <= 1) return false;
    if (n === 2) return true;
    if (n % 2 === 0) return false;
    let divisor = 3;
    while (divisor * divisor <= n) {
        if (n % divisor === 0) return false;
        divisor += 2;
    }
    return true;
}

function countPrimes(limit) {
    let count = 0;
    for (let n = 2; n <= limit; n++) {
        if (isPrime(n)) count++;
    }
    return count;
}

console.log("=== Prime Sieve Benchmark (Node.js) ===");
const N = 10_000;

const start = performance.now();
const result = countPrimes(N);
const elapsed = (performance.now() - start) / 1000;

console.log(`Primes up to ${N}: ${result}`);
console.log(`Time: ${elapsed.toFixed(3)}s`);
