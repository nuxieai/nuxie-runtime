--! attacks: coroutines and wrap functions resumed after they died, errored, were closed, or closed while holding host objects mid-call; expect: ok; within: 10s
local co = coroutine.create(function() return 1 end)
assert(coroutine.resume(co))
for _ = 1, 3 do assert(not coroutine.resume(co)) end
assert(coroutine.close(co))
-- Errored with a host userdata error object, then closed and resumed.
local errco = coroutine.create(function() error(Path.new()) end)
local ok, e = coroutine.resume(errco)
assert(not ok and type(e) == "userdata")
local c1, c2 = coroutine.close(errco)
assert(not coroutine.resume(errco))
-- wrap after death raises an ordinary error each time.
local w = coroutine.wrap(function() coroutine.yield(1) end)
w(); w()
for _ = 1, 3 do assert(not pcall(w)) end
-- Closed while suspended inside a gradient stop __index (mid host call), then resumed.
local mid = coroutine.create(function()
  return Gradient.linear(Vector.xy(0, 0), Vector.xy(1, 0), { setmetatable({}, { __index = function(_, k) coroutine.yield(); return k == "position" and 0 or 1 end }) })
end)
coroutine.resume(mid)
coroutine.close(mid)
assert(coroutine.status(mid) == "dead")
assert(not coroutine.resume(mid))
-- Closed while suspended inside a sort comparator (if the yield was allowed at all).
local sorter = coroutine.create(function() table.sort({ 3, 2, 1 }, function(a, b) coroutine.yield(); return a < b end) end)
coroutine.resume(sorter)
coroutine.close(sorter)
assert(not coroutine.resume(sorter))
-- coroutine.running/isyieldable/status on dead threads, many times.
for _ = 1, 100 do
  local t = coroutine.create(function() end)
  coroutine.resume(t)
  coroutine.status(t)
  coroutine.close(t)
end
return {}
