-- Returns into every kind of caller (docs/luau-safe-vm-call-path.md, 4.3):
-- a Lua frame waiting on a metamethod, a native waiting on its
-- continuation, a host boundary, a thread base; `__call`; varargs; stack
-- growth inside a call; call depth at the limit.

local function show(...)
    local parts = {}
    for i = 1, select("#", ...) do
        parts[i] = tostring((select(i, ...)))
    end
    return table.concat(parts, " ")
end

-- A Lua frame waiting on a metamethod that is a Lua function.
local mt = {}
mt.__index = function(t, k) return k .. "!" end
mt.__add = function(a, b) return 100 + (if type(b) == "number" then b else 0) end
mt.__lt = function(a, b) return true end
mt.__le = function(a, b) return false end
mt.__eq = function(a, b) return true end
mt.__concat = function(a, b) return "cat" end
mt.__len = function(t) return 7, 8 end
mt.__unm = function(t) return "neg" end
mt.__newindex = function(t, k, v) rawset(t, k, v * 2) end
mt.__iter = function(t)
    local i = 0
    return function()
        i += 1
        if i <= 3 then return i, i * i end
        return nil
    end
end
local a, b = setmetatable({}, mt), setmetatable({}, mt)
print(a.x, a + 1, 1 + a, a < b, a <= b, a == b, a .. "s", "s" .. a, #a, -a)
a.y = 21
print(rawget(a, "y"))
for k, v in a do
    print("iter", k, v)
end
-- Metamethods that call further, and one that raises under pcall.
mt.__index = function(t, k)
    local function inner(x) return x .. "?" end
    return inner(k) .. #k
end
print(a.deep, a.deeper)
mt.__index = function(t, k) error("index " .. k) end
print(pcall(function() return a.z end))
mt.__index = function(t, k) return nil end
print(a.none)

-- A native waiting on its continuation.
print(pcall(function(...) return ... end, 1, nil, 3))
print(pcall(function() return end))
print(select("#", pcall(function() return nil, nil end)))
print(xpcall(function(x) return x * 2, "two" end, function(e) return "h:" .. e end, 21))
print(xpcall(function() error("boom", 0) end, function(e) return "handled " .. e, "extra" end))
local sorted = {5, 3, 9, 1, 7}
table.sort(sorted, function(x, y) return x > y end)
print(table.concat(sorted, ","))
print(string.gsub("abc", "%w", function(c) return c:upper() .. "." end))
print(tostring(setmetatable({}, {__tostring = function() return "custom" end})))
print(string.format("%*|%s", setmetatable({}, {__tostring = function() return "f1" end}), "x"))

-- A host boundary: the `xpcall` handler runs from Rust (`call_value`).
print(xpcall(error, function(e)
    local function nested(n) if n == 0 then return "deep" end return nested(n - 1) end
    return nested(50) .. ":" .. tostring(e)
end, "msg"))

-- A thread base: a coroutine's body returning, wrapped and resumed.
local co = coroutine.create(function(x, y)
    local z = coroutine.yield(x + y)
    return z, "done", nil
end)
print(coroutine.resume(co, 1, 2))
print(coroutine.resume(co, "z"))
print(coroutine.status(co), coroutine.resume(co))
local gen = coroutine.wrap(function(...)
    for i = 1, select("#", ...) do
        coroutine.yield((select(i, ...)))
    end
    return "end"
end)
print(gen("p", "q"), gen(), gen(), pcall(gen))
local outer = coroutine.wrap(function()
    local inner = coroutine.wrap(function() coroutine.yield(1); return 2 end)
    return inner() + inner()
end)
print(outer())

-- `__call`.
local callable = setmetatable({}, {__call = function(self, x, ...) return "called", x, select("#", ...) end})
print(callable(1, 2, 3))
print(callable())
print(pcall(callable, "p"))
local through = setmetatable({}, {__index = function(t, k) return callable(k) end})
print(through.key)
local chained = setmetatable({}, {__call = callable})
print(pcall(chained, 1))
print(pcall(setmetatable({}, {__call = 5})))

-- Varargs with fewer and more arguments than parameters.
local function fixed(p, q, ...)
    return show(p, q, select("#", ...), ...)
end
print(fixed())
print(fixed(1))
print(fixed(1, 2))
print(fixed(1, 2, 3, nil, 5))
local function onlyva(...) local x, y, z = ...; return show(x, y, z, select("#", ...)) end
print(onlyva(), onlyva(1), onlyva(1, 2, 3, 4))
local function many(a1, a2, a3, a4, a5, a6, a7, a8) return show(a1, a2, a3, a4, a5, a6, a7, a8) end
print(many(1, 2), many(1, 2, 3, 4, 5, 6, 7, 8, 9, 10))

-- Stack growth inside a call: wide frames, deep, and long result lists.
local function wide(n)
    local l1, l2, l3, l4, l5, l6, l7, l8, l9, l10 = n, n, n, n, n, n, n, n, n, n
    local m1, m2, m3, m4, m5, m6, m7, m8, m9, m10 = n, n, n, n, n, n, n, n, n, n
    if n == 0 then return 0 end
    return wide(n - 1) + l1 + l10 - m1 - m10 + 1
end
print(wide(3000))
local long = {}
for i = 1, 5000 do long[i] = i end
local function spread() return table.unpack(long) end
local function count(...) return select("#", ...), (select(5000, ...)) end
print(count(spread()))
local function relay(...) return ... end
print(count(relay(relay(spread()))))

-- Call depth at the limit and one below it: the deepest non-tail
-- recursion that succeeds, and the error one past it.
local function depth(n)
    if n == 0 then return 0 end
    return 1 + depth(n - 1)
end
local lo, hi = 1, 40000
while lo < hi do
    local mid = (lo + hi + 1) // 2
    if pcall(depth, mid) then lo = mid else hi = mid - 1 end
end
print("deepest", lo, pcall(depth, lo))
print(pcall(depth, lo + 1))
local function depthv(...)
    local n = ...
    if n == 0 then return 0 end
    return 1 + depthv(n - 1, n)
end
lo, hi = 1, 40000
while lo < hi do
    local mid = (lo + hi + 1) // 2
    if pcall(depthv, mid) then lo = mid else hi = mid - 1 end
end
print("deepest vararg", lo, pcall(depthv, lo + 1))
