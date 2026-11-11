-- Re-entry into the interpreter, up to the C-call limit: __index
-- functions, __call, pcall and tostring nesting, sort comparators, gsub
-- callbacks and host callbacks (print rendering a __tostring that prints).
-- Each must end in the same error text and depth, never in a host stack
-- overflow.
local depth = 0
local deep = setmetatable({}, {__index = function(t, k)
    depth = depth + 1
    return t[k]
end})
local ok, err = pcall(function() return deep.x end)
print(ok, err, depth)

depth = 0
local function nest()
    depth = depth + 1
    return pcall(nest)
end
-- `nest` captures itself by value: NEWCLOSURE must store the closure in
-- its register before the captures run.
local results = {nest()}
print(#results, results[1], results[#results], depth)

local callee = setmetatable({}, {__call = function(self, n)
    if n == 0 then return "bottom" end
    return self(n - 1)
end})
print(pcall(callee, 150))
print(pcall(callee, 10000))

local named = {}
setmetatable(named, {__tostring = function(self) return tostring(self) end})
ok, err = pcall(tostring, named)
print(ok, err)

local function lessthan(a, b) return a < b end
local cmp = setmetatable({}, {__lt = function(a, b) return lessthan(a, b) end})
ok, err = pcall(lessthan, cmp, cmp)
print(ok, err)
-- A sort comparator that sorts: each level is a counted call.
depth = 0
local function nested_sort(a, b)
    depth = depth + 1
    table.sort({3, 1, 2}, nested_sort)
    return a < b
end
ok, err = pcall(table.sort, {2, 1}, nested_sort)
print(ok, err, depth)
-- Comparator errors and results keep their order (C++'s introsort).
local order = {}
local data = {5, 3, 8, 1, 9, 2, 7, 4, 6, 10, 15, 12, 11, 14, 13}
table.sort(data, function(a, b)
    table.insert(order, a .. "<" .. b)
    return a < b
end)
print(table.concat(data, ","), #order, order[1], order[#order])
local lt = setmetatable({}, {__lt = function(a, b) return a.v < b.v end})
local objs = {}
for i, v in ipairs({4, 2, 5, 1, 3}) do objs[i] = setmetatable({v = v}, getmetatable(lt)) end
table.sort(objs)
print(objs[1].v, objs[2].v, objs[3].v, objs[4].v, objs[5].v)
print(pcall(table.sort, {1, 2, 3}, function(a, b) error("cmp") end))
print(pcall(table.sort, {3, 2, 1, 5, 4}, function(a, b) return true end))

-- A gsub callback that calls gsub.
depth = 0
local function nested_gsub(c)
    depth = depth + 1
    return (string.gsub("x", "x", nested_gsub))
end
ok, err = pcall(string.gsub, "a", "a", nested_gsub)
print(ok, err, depth)
print(string.gsub("hello world", "%w+", function(w) return w:upper() end))
print(string.gsub("abc", "%w", setmetatable({}, {__index = function(t, k) return k .. k end})))
print(string.gsub("abc", "(b)", {b = false}))
print(pcall(string.gsub, "abc", "b", function() return {} end))
print(string.gsub("a,b,,c", ",", function() end, 2))

-- The host's print renders a __tostring that prints: host re-entry.
depth = 0
local loud = setmetatable({}, {__tostring = function(self)
    depth = depth + 1
    print(self)
    return "loud"
end})
ok, err = pcall(print, loud)
print(ok, err)
local quiet = setmetatable({}, {__tostring = function() return "quiet" end})
print(quiet, tostring(quiet))

-- Generic for over a __call iterator and an __iter metamethod.
local calls = setmetatable({n = 0}, {__call = function(self)
    self.n = self.n + 1
    if self.n <= 3 then return self.n end
end})
local last = 0
for i in calls do last = i end
print(calls.n, last)
local iterable = setmetatable({}, {__iter = function() return next, {a = 1} end})
for k, v in iterable do print(k, v) end
return "done"
