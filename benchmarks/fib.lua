-- Fibonacci Benchmark - Lua

local function fib(n)
    if n <= 1 then
        return n
    end
    return fib(n - 1) + fib(n - 2)
end

print("=== Fibonacci Benchmark (Lua) ===")

local start = os.clock()
local result = fib(35)
local elapsed = os.clock() - start

print(string.format("fib(35) = %d", result))
print(string.format("Time: %.3fs", elapsed))
