--! attacks: listeners that remove themselves, remove others not yet called, and remove-then-re-add during one notification; expect: ok; within: 10s; viewmodels: script_create_viewmodel_instance.riv
local inst = Data.SubChild.new()
local p = inst:getNumber("num")
local log = {}
local f1, f2, f3
f1 = function() table.insert(log, 1) p:removeListener(f1) end
f2 = function() table.insert(log, 2) p:removeListener(f3) p:removeListener(f2) p:addListener(f2) end
f3 = function() table.insert(log, 3) end
p:addListener(f1)
p:addListener(f2)
p:addListener(f3)
for i = 1, 5 do p.value = i end
assert(#log < 100)
-- removing with junk arguments
assert(not pcall(function() p:removeListener() end))
assert(not pcall(function() p:removeListener(1) end))
p:removeListener(function() end)
-- a listener that drops every reference to its own wrapper and instance
local inst2 = Data.SubChild.new()
local w = inst2:getNumber("num")
w:addListener(function() inst2 = nil w = nil end)
w.value = 3
return {}
