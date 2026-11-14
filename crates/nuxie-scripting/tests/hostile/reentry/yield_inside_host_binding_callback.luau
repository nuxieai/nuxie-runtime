--! attacks: coroutine.yield inside a gradient stop __index run by Gradient.linear, inside a pcall'd host call, and resuming after the host call returned; expect: ok; within: 10s
local function stopsThatYield()
  return {
    setmetatable({}, { __index = function(_, k) coroutine.yield(k); return k == "position" and 0 or 0xff000000 end }),
    { position = 1, color = 0xffffffff },
  }
end
local co = coroutine.create(function()
  return Gradient.linear(Vector.xy(0, 0), Vector.xy(1, 0), stopsThatYield())
end)
local ok, a = coroutine.resume(co)
local guard = 0
while ok and coroutine.status(co) == "suspended" and guard < 10 do
  guard += 1
  ok, a = coroutine.resume(co)
end
assert(coroutine.status(co) == "dead")
-- Same, but the host call is under pcall inside the coroutine.
local co2 = coroutine.create(function()
  local okInner, err = pcall(Gradient.radial, Vector.xy(0, 0), 5, stopsThatYield())
  return okInner, err
end)
ok, a = coroutine.resume(co2)
guard = 0
while ok and coroutine.status(co2) == "suspended" and guard < 10 do
  guard += 1
  ok, a = coroutine.resume(co2)
end
assert(coroutine.status(co2) == "dead")
-- Yield inside a Paint __newindex conversion: a color table whose __tostring yields.
local co3 = coroutine.create(function()
  local p = Paint.new()
  p.color = setmetatable({}, { __tostring = function() coroutine.yield(); return "1" end })
end)
ok = coroutine.resume(co3)
-- A coroutine left suspended inside a host call is dropped with the VM.
local stuck = coroutine.create(function()
  return Gradient.linear(Vector.xy(0, 0), Vector.xy(1, 0), stopsThatYield())
end)
coroutine.resume(stuck)
_G_stuck = nil
return { stuck = stuck }
