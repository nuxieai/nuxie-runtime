--! attacks: listeners that add more listeners while being notified (unbounded delegate growth, G4 part 2), across one wrapper and many wrappers; expect: ok; within: 20s; viewmodels: script_create_viewmodel_instance.riv
local inst = Data.SubChild.new()
local p = inst:getNumber("num")
local total = 0
local function spawn()
  total += 1
  pcall(function() p:addListener(spawn) end)
  pcall(function() p:addListener(function() total += 1 end) end)
end
p:addListener(spawn)
for i = 1, 12 do p.value = i end
-- the per-property cap holds: the next add is an ordinary error
local ok, err = pcall(function() p:addListener(function() end) end)
assert(not ok and string.find(tostring(err), "at most"), tostring(err))
-- many wrappers of one property, each adding a listener on a fresh wrapper when notified
local q = inst:getBoolean("bol")
local wrappers = {}
local function grow()
  local w = inst:getBoolean("bol")
  table.insert(wrappers, w)
  pcall(function() w:addListener(grow) end)
end
q:addListener(grow)
for i = 1, 14 do q.value = (i % 2 == 0) end
assert(#wrappers < 100000, "wrappers " .. #wrappers)
return {}
