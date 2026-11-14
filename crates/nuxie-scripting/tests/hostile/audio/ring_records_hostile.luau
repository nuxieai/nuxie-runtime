--! attacks: ring send of oversized/many records, receive on empty, receiveInto hostile targets; expect: ok; within: 15s
local ring = Audio.Ring.create(64)
-- empty ring
assert(ring:receive() == nil)
assert(ring:receiveInto({}) == false)
-- oversized record: refused (false), not stored
assert(ring:send(string.rep("x", 1000)) == false)
assert(ring:send(table.create(100, 1)) == false)
assert(ring:receive() == nil)
-- fill to the brim with small records; every send either fits or reports false
local sent = 0
for i = 1, 100 do
  if ring:send(i) then sent += 1 else break end
end
assert(sent > 0 and sent < 100)
for i = 1, sent do assert(ring:receive() == i) end
assert(ring:receive() == nil)
-- many records through a bigger ring, interleaved
local big = Audio.Ring.create(1024 * 1024)
for i = 1, 8000 do
  assert(big:send({i, "r"}))
  if i % 2 == 0 then
    local v = big:receive()
    assert(type(v) == "table")
  end
end
local drained = 0
while big:receive() ~= nil do drained += 1 end
assert(drained == 4000, drained)
-- receiveInto: frozen target, target with a metatable, non-table record
assert(big:send({a = 1}))
local frozen = table.freeze({x = 1})
local ok, err = pcall(function() return big:receiveInto(frozen) end)
assert(not ok, "wrote into a frozen table")
assert(big:send(7))
local ok2 = pcall(function() return big:receiveInto(setmetatable({}, {__newindex = function() error("no") end})) end)
assert(not ok2, "a number record filled a table")
-- argument errors
assert(not pcall(function() return ring:send() end))
assert(not pcall(function() return ring:send(1, 2) end))
assert(not pcall(function() return ring:receive(1) end))
assert(not pcall(function() return ring.receive(nil) end))
return {}
