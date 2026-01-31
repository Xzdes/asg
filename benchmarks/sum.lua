-- Sum Benchmark - Lua

local function sum_to_n(n)
    local total = 0
    for i = 1, n do
        total = total + i
    end
    return total
end

print("=== Sum Benchmark (Lua) ===")
local N = 1000000

local start = os.clock()
local result = sum_to_n(N)
local elapsed = os.clock() - start

print(string.format("sum(1..%d) = %d", N, result))
print(string.format("Time: %.3fs", elapsed))
