// Fibonacci Benchmark - Node.js

function fib(n) {
    if (n <= 1) return n;
    return fib(n - 1) + fib(n - 2);
}

console.log("=== Fibonacci Benchmark (Node.js) ===");

const start = performance.now();
const result = fib(35);
const elapsed = (performance.now() - start) / 1000;

console.log(`fib(35) = ${result}`);
console.log(`Time: ${elapsed.toFixed(3)}s`);
