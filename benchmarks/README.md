# ASG Performance Benchmarks

Comparison of ASG interpreter performance against Python, Lua, and Node.js.

## Benchmark Categories

### 1. Recursive Algorithms
- **Fibonacci** - Classic recursive computation
- **Factorial** - Tail-recursive multiplication

### 2. Iteration
- **Sum** - Sum of 1 to N
- **Prime Sieve** - Count primes up to N

### 3. Higher-Order Functions
- **Map/Filter/Reduce** - Functional operations on arrays
- **Compose** - Function composition chains

### 4. Data Structures
- **Array Operations** - Creation, access, modification
- **Dict Operations** - Hash table performance

## Running Benchmarks

### Prerequisites

```bash
# Build ASG in release mode
cargo build --release

# Ensure you have installed:
# - Python 3.x
# - Lua 5.4+
# - Node.js 18+
```

### Run All Benchmarks

```bash
cd benchmarks
./run_all.sh        # Linux/Mac
run_all.bat         # Windows
```

### Run Individual Benchmarks

```bash
# ASG
cargo run --release --bin asg -- benchmarks/fib.asg

# Python
python benchmarks/fib.py

# Lua
lua benchmarks/fib.lua

# Node.js
node benchmarks/fib.js
```

## Benchmark Files

| Benchmark | ASG | Python | Lua | Node.js |
|-----------|---------|--------|-----|---------|
| Fibonacci (35) | [fib.asg](fib.asg) | [fib.py](fib.py) | [fib.lua](fib.lua) | [fib.js](fib.js) |
| Factorial (20) | [fact.asg](fact.asg) | [fact.py](fact.py) | [fact.lua](fact.lua) | [fact.js](fact.js) |
| Sum (1M) | [sum.asg](sum.asg) | [sum.py](sum.py) | [sum.lua](sum.lua) | [sum.js](sum.js) |
| Primes (10K) | [primes.asg](primes.asg) | [primes.py](primes.py) | [primes.lua](primes.lua) | [primes.js](primes.js) |
| Array Ops | [array.asg](array.asg) | [array.py](array.py) | [array.lua](array.lua) | [array.js](array.js) |

## Expected Results

Performance comparison (lower is better):

| Benchmark | ASG | Python | Lua | Node.js |
|-----------|---------|--------|-----|---------|
| fib(35) | TBD | ~2.5s | ~1.5s | ~0.3s |
| fact(20) x 100K | TBD | ~0.5s | ~0.3s | ~0.1s |
| sum(1M) | TBD | ~0.1s | ~0.05s | ~0.01s |
| primes(10K) | TBD | ~0.3s | ~0.2s | ~0.05s |

## Notes

- All benchmarks use the interpreter (not LLVM/WASM compilation)
- Times are approximate and depend on hardware
- Python uses default CPython (no PyPy)
- Node.js uses V8's JIT compilation
- Lua uses standard Lua (not LuaJIT)

## Optimization Opportunities

1. **Tail Call Optimization** - Would significantly improve recursive benchmarks
2. **JIT Compilation** - Could approach Node.js performance
3. **Native Math** - LLVM backend for compute-heavy tasks
4. **Memoization** - Built-in caching for pure functions
