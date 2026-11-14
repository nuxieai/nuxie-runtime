--! attacks: table.sort comparators that mutate, shrink, grow or re-sort the array being sorted, and inconsistent orders; expect: ok; within: 15s
local function fresh(n)
  local t = {}
  for i = 1, n do t[i] = (i * 7919) % 101 end
  return t
end
-- Comparator clears the table mid-sort.
local t = fresh(200)
local ok = pcall(table.sort, t, function(a, b)
  for i = #t, 1, -1 do t[i] = nil end
  return (a or 0) < (b or 0)
end)
-- Comparator grows the table mid-sort (bounded).
t = fresh(200)
local grown = 0
ok = pcall(table.sort, t, function(a, b)
  if grown < 5000 then grown += 1; t[#t + 1] = grown end
  return a < b
end)
-- Comparator replaces elements with non-comparable values.
t = fresh(200)
ok = pcall(table.sort, t, function(a, b)
  t[1] = {}
  t[#t] = Path.new()
  return a < b
end)
-- Comparator sorts the same table again (nested sort on the same array).
t = fresh(16)
local depth = 0
local nested = 0
ok = pcall(table.sort, t, function(a, b)
  if depth < 1 and nested < 20 then
    nested += 1
    depth += 1
    table.sort(t, function(x, y) return x > y end)
    depth -= 1
  end
  return a < b
end)
-- Inconsistent comparator: must end with an error or a permutation, never hang or crash.
t = fresh(500)
ok = pcall(table.sort, t, function(a, b) return true end)
t = fresh(500)
local flip = false
ok = pcall(table.sort, t, function(a, b) flip = not flip; return flip end)
-- __lt metamethod that sorts the array it is being compared in.
local arr = {}
local mt = {}
mt.__lt = function(a, b)
  if #arr > 1 and depth < 1 and nested < 40 then
    nested += 1
    depth += 1
    table.sort(arr)
    depth -= 1
  end
  return a.v < b.v
end
for i = 1, 20 do arr[i] = setmetatable({ v = (i * 37) % 53 }, mt) end
ok = pcall(table.sort, arr)
-- Comparator that raises an error object with a __tostring calling sort.
t = fresh(20)
ok = pcall(table.sort, t, function()
  error(setmetatable({}, { __tostring = function() table.sort(t); return "x" end }))
end)
assert(not ok)
return {}
