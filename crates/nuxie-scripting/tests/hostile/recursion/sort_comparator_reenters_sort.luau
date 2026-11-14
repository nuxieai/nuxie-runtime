--! attacks: table.sort comparator that recursively sorts (unbounded host re-entry) and one that sorts the table being sorted; expect: ok; within: 20s
local function nopanic(e) assert(not string.find(tostring(e), "rust panic", 1, true), tostring(e)) end
local depth = 0
local function cmp(a, b)
  depth = depth + 1
  table.sort({ 3, 2, 1 }, cmp)
  return a < b
end
local ok, e = pcall(table.sort, { 5, 4, 3, 2, 1 }, cmp)
assert(not ok, "unbounded sort recursion returned normally")
nopanic(e)
-- comparator mutates and re-sorts the very table being sorted
local t = {}
for i = 1, 200 do t[i] = (i * 7919) % 200 end
local guard = 0
local ok2, e2 = pcall(table.sort, t, function(a, b)
  guard = guard + 1
  if guard < 50 then table.sort(t) end
  t[(guard % #t) + 1] = guard
  return (a or 0) < (b or 0)
end)
nopanic(e2)
-- inconsistent comparator
local ok3, e3 = pcall(table.sort, t, function() return true end)
nopanic(e3)
return {}
