-- Prime Sieve Benchmark - Lua

local function is_prime(n)
    if n <= 1 then return false end
    if n == 2 then return true end
    if n % 2 == 0 then return false end
    local divisor = 3
    while divisor * divisor <= n do
        if n % divisor == 0 then
            return false
        end
        divisor = divisor + 2
    end
    return true
end

local function count_primes(limit)
    local count = 0
    for n = 2, limit do
        if is_prime(n) then
            count = count + 1
        end
    end
    return count
end

print("=== Prime Sieve Benchmark (Lua) ===")
local N = 10000

local start = os.clock()
local result = count_primes(N)
local elapsed = os.clock() - start

print(string.format("Primes up to %d: %d", N, result))
print(string.format("Time: %.3fs", elapsed))
