-- Factorial Benchmark - Lua

local function factorial(n)
    if n <= 1 then return 1 end
    return n * factorial(n - 1)
end

print("=== Factorial Benchmark (Lua) ===")
local iterations = 100000

local start = os.clock()
local result = 0
for _ = 1, iterations do
    result = factorial(20)
end
local elapsed = os.clock() - start

print(string.format("factorial(20) x %d = %d", iterations, result))
print(string.format("Time: %.3fs", elapsed))
