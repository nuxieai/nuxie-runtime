--! attacks: coroutine.resume nested inside resume without bound, and coroutine.wrap chains; expect: ok; within: 20s
local function nopanic(e) assert(not string.find(tostring(e), "rust panic", 1, true), tostring(e)) end
local reached = 0
local function nest(n)
  reached = n
  local co = coroutine.create(nest)
  local ok, e = coroutine.resume(co, n + 1)
  if not ok then nopanic(e) error(e, 0) end
  return e
end
local ok, e = pcall(nest, 1)
assert(not ok, "unbounded resume nesting returned normally")
nopanic(e)
-- wrap chain: each generator pulls from the previous one, 10,000 long
local gen = coroutine.wrap(function() while true do coroutine.yield(1) end end)
for i = 1, 10000 do
  local prev = gen
  gen = coroutine.wrap(function() while true do coroutine.yield(prev() + 1) end end)
end
local okw, v = pcall(gen)
nopanic(v)
-- a coroutine that resumes itself, and a wrap that calls itself
local self_co
self_co = coroutine.create(function() return coroutine.resume(self_co) end)
local oks, a, b = coroutine.resume(self_co)
nopanic(a) nopanic(b)
local selfwrap
selfwrap = coroutine.wrap(function() return selfwrap() end)
local okw2, ew2 = pcall(selfwrap)
assert(not okw2) nopanic(ew2)
return {}
