// Sum Benchmark - Node.js

function sumToN(n) {
    let total = 0;
    for (let i = 1; i <= n; i++) {
        total += i;
    }
    return total;
}

console.log("=== Sum Benchmark (Node.js) ===");
const N = 1_000_000;

const start = performance.now();
const result = sumToN(N);
const elapsed = (performance.now() - start) / 1000;

console.log(`sum(1..${N}) = ${result}`);
console.log(`Time: ${elapsed.toFixed(3)}s`);
