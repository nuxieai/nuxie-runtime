-- The embedding API (phase 2 lane 6) against luaur-rt: the same host
-- functions, userdata type, interrupt and memory limit on both VMs
-- (tools/luau-vm-diff/src/host_surface.rs).
--
-- Two texts differ by design and are not compared here (lane 6 evidence
-- doc): luaur-rt renders a non-string error object coming back through a
-- host call as "<non-string error>" (this VM: "(error object is a table
-- value)", C++ lua_tostring's failure), and a method called on the wrong
-- receiver as "userdata type mismatch" (this VM: luaL_typeerror).

-- Host functions: typed arguments, calls back into Lua, errors.
print(hostadd(1, 2.5), hostadd("3", 4))
print(hostcall(function(x) return x * 2, "two" end, 21))
print(select("#", hostcall(function() end)))
print(pcall(hostcall, function() error("inner") end))
print(pcall(hosterror, "outer"))
local function nest(n)
  if n == 0 then return "bottom" end
  return hostcall(nest, n - 1)
end
print(nest(6))
print(pcall(function() return hostcall(coroutine.yield, 1) end))
local co = coroutine.wrap(function() return hostcall(coroutine.yield, 1) end)
print(pcall(co))

-- A userdata type: fields, methods, metamethods.
local c = newcounter(3)
print(c.value, c:get(), #c, tostring(c), type(c), typeof(c))
c.value = 10
print(c:add(5), c.value, c.missing)
local add = c.add
print(add(c, 1), c.value)
print((pcall(function() c.nope = 1 end)))
local d = newcounter(1)
print(rawequal(c, d), c == d, c == c)

-- Destructors run once each, at collection.
local function churn() local _ = newcounter(0) end
for _ = 1, 10 do churn() end
c, d, add = nil, nil, nil
collectgarbage()
print(drops())
collectgarbage()
print(drops())

-- The interrupt: the same safepoints on both VMs.
local before = interruptcount()
for _ = 1, 10 do end
print("numeric for", interruptcount() - before)
before = interruptcount()
for _ in pairs({1, 2, 3}) do end
print("generic for", interruptcount() - before)
before = interruptcount()
local n = 0
while n < 5 do n += 1 end
print("while", interruptcount() - before)
before = interruptcount()
local function f() return 1 end
for _ = 1, 3 do f() end
print("calls", interruptcount() - before)
before = interruptcount()
string.find(string.rep("a", 40), "a*b")
print("pattern", interruptcount() - before)
before = interruptcount()
string.gsub("hello world", "(%w+)", "<%1>")
print("gsub", interruptcount() - before)
print(pcall(function()
  setinterruptlimit(50)
  while true do end
end))
print(pcall(function()
  setinterruptlimit(3)
  return string.find(string.rep("a", 100), "a-b")
end))
print("still runs", hostadd(1, 1))

-- The memory limit: an out-of-memory error a script can catch.
memlimit(64 * 1024)
local ok, e = pcall(string.rep, "x", 1000000)
memlimit()
print(ok, e)
print(#string.rep("y", 1000))
