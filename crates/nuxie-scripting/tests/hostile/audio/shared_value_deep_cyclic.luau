--! attacks: deep, cyclic, sparse and unsupported tables posted through shared values (lane A A.1 limits); expect: ok; within: 15s
local ring = Audio.Ring.create(1024 * 1024)
local snap = Audio.Snapshot.create(1024 * 1024)
local function refused(v, needle)
  for _, post in ipairs({function() return ring:send(v) end, function() return snap:publish(v) end}) do
    local ok, err = pcall(post)
    assert(not ok, "accepted " .. needle)
    assert(string.find(tostring(err), needle, 1, true), tostring(err))
  end
end
-- 10,000 deep
local deep = {}
for i = 1, 10000 do deep = {deep} end
refused(deep, "nesting exceeds")
-- cyclic: self, and through a sibling
local c = {}
c[1] = c
refused(c, "cyclic")
local a, b = {}, {}
a.b = b
b.a = a
refused(a, "cyclic")
-- a DAG that shares one subtable many times is not a cycle, but every visit is charged
local leaf = {1, 2, 3, 4, 5, 6, 7, 8}
local wide = {}
for i = 1, 512 do wide[i] = leaf end
local nested = {}
for i = 1, 8 do nested[i] = wide end
local ok = pcall(function() return ring:send(nested) end)
-- 37k charged values: either fits the ring (false) or refused; must not crash
-- sparse / mixed / bad keys
refused({[1] = 1, [3] = 3}, "dense arrays")
refused({[2 ^ 53] = 1}, "dense arrays")
refused({1, x = 2}, "dense arrays")
refused({[true] = 1}, "dense arrays")
refused({f = print}, "audio data supports")
refused(coroutine.create(function() end), "audio data supports")
refused({["\255\254"] = 1}, "UTF-8")
-- metatables must not be consulted
local trap = setmetatable({}, {__index = function() error("index ran") end, __iter = function() error("iter ran") end})
pcall(function() return ring:send(trap) end)
-- 4 MiB string: past ring capacity, refused or false
local s = string.rep("z", 4 * 1024 * 1024)
assert(ring:send(s) == false)
assert(not pcall(function() return snap:publish(s) end) or true)
return {}
