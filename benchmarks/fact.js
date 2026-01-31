// Factorial Benchmark - Node.js

function factorial(n) {
    if (n <= 1) return 1;
    return n * factorial(n - 1);
}

console.log("=== Factorial Benchmark (Node.js) ===");
const iterations = 100_000;

const start = performance.now();
let result = 0;
for (let i = 0; i < iterations; i++) {
    result = factorial(20);
}
const elapsed = (performance.now() - start) / 1000;

console.log(`factorial(20) x ${iterations} = ${result}`);
console.log(`Time: ${elapsed.toFixed(3)}s`);
