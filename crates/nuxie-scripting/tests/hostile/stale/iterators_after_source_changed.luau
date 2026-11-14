--! attacks: pairs/next/ipairs/generalized iteration, gmatch and utf8.codes iterators used after their source changed or was cleared; expect: ok; within: 10s
-- next() with a key removed and the table rehashed between steps.
local t = {}
for i = 1, 64 do t["k" .. i] = i end
local k = next(t)
t[k] = nil
for i = 1, 512 do t["new" .. i] = i end
local ok = pcall(next, t, k)
-- pairs while assigning new keys (undefined order, must not crash or hang).
local t2 = {}
for i = 1, 32 do t2[i] = i; t2["s" .. i] = i end
local steps = 0
pcall(function()
  for key, value in pairs(t2) do
    steps += 1
    if steps > 5000 then break end
    t2["x" .. steps] = steps
    t2[key] = nil
  end
end)
-- generalized iteration with the table cleared mid-loop.
local t3 = { 1, 2, 3, 4, 5, a = 1, b = 2 }
for key in t3 do
  table.clear(t3)
end
-- a stashed pairs iterator triple used after the table was replaced by table.move/clear.
local f, s, ctl = pairs({ a = 1, b = 2, c = 3 })
local ka = f(s, ctl)
table.clear(s)
pcall(f, s, ka)
-- gmatch iterator kept after its loop, advanced from another coroutine and after exhaustion.
local it = string.gmatch("a1b2c3", "%a(%d)")
assert(it() == "1")
local co = coroutine.wrap(function() return it() end)
assert(co() == "2")
assert(it() == "3")
assert(it() == nil)
assert(it() == nil)
-- utf8.codes over a string, iterator triple reused with a different (shorter, invalid) string.
local uf, us, uc = utf8.codes("h\u{e9}llo")
local i1 = uf(us, uc)
pcall(uf, "\xff\xfe", i1)
pcall(uf, "", 100)
pcall(uf, "abc", -5)
pcall(uf, "abc", 2 ^ 40)
-- utf8.codes on invalid UTF-8 raises on reaching the bad byte.
assert(not pcall(function() for _ in utf8.codes("ab\xffcd") do end end))
-- ipairs over a table whose array part shrinks mid-iteration.
local arr = {}
for i = 1, 100 do arr[i] = i end
for i, v in ipairs(arr) do arr[#arr] = nil end
return {}
