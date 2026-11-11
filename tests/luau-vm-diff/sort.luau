-- table.sort: results, the predicate's call sequence, and errors
local function show(t) local p = {} for i = 1, #t do p[i] = tostring(t[i]) end return table.concat(p, " ") end
local a = {5, 3, 8, 1, 9, 2, 7, 4, 6, 0}
table.sort(a)
print(show(a))
local b = {"pear", "apple", "fig", "banana", "cherry"}
table.sort(b)
print(show(b))
table.sort(b, function(x, y) return #x < #y end)
print(show(b))
local calls = {}
local c = {3, 1, 2, 5, 4, 9, 7, 8, 6, 10, 15, 11, 14, 12, 13}
table.sort(c, function(x, y) calls[#calls + 1] = x .. "<" .. y; return x > y end)
print(show(c))
print(#calls, table.concat(calls, ",", 1, 12))
local big = {}
local seed = 7
for i = 1, 2000 do seed = (seed * 1103515245 + 12345) % 2147483648; big[i] = seed % 1000 end
table.sort(big)
local okorder = true
for i = 2, #big do if big[i - 1] > big[i] then okorder = false end end
print(okorder, big[1], big[1000], big[2000])
-- adversarial: all equal, already sorted, reversed
local eq = {} for i = 1, 300 do eq[i] = 1 end
table.sort(eq)
local rev = {} for i = 1, 300 do rev[i] = 301 - i end
table.sort(rev)
print(eq[1], eq[300], rev[1], rev[300])
print(pcall(table.sort, {1, "x", 2}))
print(pcall(table.sort, {3, 2, 1}, function(x, y) return true end))
print(pcall(table.sort, {1, 2}, 5))
print(pcall(table.sort, "x"))
local mt = setmetatable({}, {__lt = function(x, y) return x.v < y.v end})
local objs = {}
for i, v in ipairs({4, 2, 3, 1}) do objs[i] = setmetatable({v = v}, getmetatable(mt)) end
table.sort(objs)
print(objs[1].v, objs[2].v, objs[3].v, objs[4].v)
local grow = {3, 2, 1}
print(pcall(table.sort, grow, function(x, y) grow[#grow + 1] = 0; return x < y end))
table.sort({})
table.sort({1})
return show(a)
