@echo off
REM ASG Benchmark Runner - Windows
REM Run all benchmarks and compare results

echo ===============================================
echo        ASG Performance Benchmarks
echo ===============================================
echo.

cd /d %~dp0\..

echo Building ASG in release mode...
cargo build --release
echo.

echo ===============================================
echo Benchmark 1: Fibonacci(35)
echo ===============================================
echo.

echo --- ASG ---
powershell -Command "Measure-Command { cargo run --release --bin asg -- benchmarks\fib.asg 2>$null } | Select-Object -ExpandProperty TotalSeconds"
echo.

echo --- Python ---
python --version
powershell -Command "Measure-Command { python benchmarks\fib.py } | Select-Object -ExpandProperty TotalSeconds"
echo.

echo --- Node.js ---
node --version
powershell -Command "Measure-Command { node benchmarks\fib.js } | Select-Object -ExpandProperty TotalSeconds"
echo.

echo ===============================================
echo Benchmark 2: Sum(1..1000000)
echo ===============================================
echo.

echo --- ASG ---
powershell -Command "Measure-Command { cargo run --release --bin asg -- benchmarks\sum.asg 2>$null } | Select-Object -ExpandProperty TotalSeconds"
echo.

echo --- Python ---
powershell -Command "Measure-Command { python benchmarks\sum.py } | Select-Object -ExpandProperty TotalSeconds"
echo.

echo --- Node.js ---
powershell -Command "Measure-Command { node benchmarks\sum.js } | Select-Object -ExpandProperty TotalSeconds"
echo.

echo ===============================================
echo Benchmark 3: Primes up to 10000
echo ===============================================
echo.

echo --- ASG ---
powershell -Command "Measure-Command { cargo run --release --bin asg -- benchmarks\primes.asg 2>$null } | Select-Object -ExpandProperty TotalSeconds"
echo.

echo --- Python ---
powershell -Command "Measure-Command { python benchmarks\primes.py } | Select-Object -ExpandProperty TotalSeconds"
echo.

echo --- Node.js ---
powershell -Command "Measure-Command { node benchmarks\primes.js } | Select-Object -ExpandProperty TotalSeconds"
echo.

echo ===============================================
echo Benchmark 4: Factorial(20) x 100000
echo ===============================================
echo.

echo --- ASG ---
powershell -Command "Measure-Command { cargo run --release --bin asg -- benchmarks\fact.asg 2>$null } | Select-Object -ExpandProperty TotalSeconds"
echo.

echo --- Python ---
powershell -Command "Measure-Command { python benchmarks\fact.py } | Select-Object -ExpandProperty TotalSeconds"
echo.

echo --- Node.js ---
powershell -Command "Measure-Command { node benchmarks\fact.js } | Select-Object -ExpandProperty TotalSeconds"
echo.

echo ===============================================
echo        Benchmarks Complete!
echo ===============================================
