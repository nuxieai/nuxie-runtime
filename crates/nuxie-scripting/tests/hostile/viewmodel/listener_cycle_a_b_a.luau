--! attacks: listener chains A->B->A across two properties and across two instances (G4 delegate cycle); expect: ok; within: 15s; viewmodels: script_create_viewmodel_instance.riv
local inst = Data.SubChild.new()
local a = inst:getNumber("num")
local b = inst:getString("str")
local ca, cb = 0, 0
a:addListener(function() ca += 1 b.value = tostring(ca) end)
b:addListener(function() cb += 1 a.value = cb end)
a.value = 42
assert(ca < 100 and cb < 100, "cycle ran " .. ca .. "/" .. cb)
-- across instances, three hops
local x, y, z = Data.SubChild.new(), Data.SubChild.new(), Data.SubChild.new()
local hops = 0
x:getNumber("num"):addListener(function() hops += 1 y:getNumber("num").value = hops end)
y:getNumber("num"):addListener(function() hops += 1 z:getNumber("num").value = hops end)
z:getNumber("num"):addListener(function() hops += 1 x:getNumber("num").value = hops end)
x:getNumber("num").value = 1
assert(hops < 100, "three-hop cycle ran " .. hops)
return {}
