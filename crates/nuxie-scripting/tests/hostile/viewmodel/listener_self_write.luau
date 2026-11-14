--! attacks: a property listener that writes its own property (re-entrant notification); expect: ok; within: 15s; viewmodels: script_create_viewmodel_instance.riv
local inst = Data.SubChild.new()
local p = inst:getNumber("num")
local calls = 0
p:addListener(function()
  calls += 1
  p.value = p.value + 1
end)
p.value = 1
assert(calls < 1000, "listener ran " .. calls .. " times")
-- string listener appending to itself
local s = inst:getString("str")
local scalls = 0
s:addListener(function()
  scalls += 1
  if #s.value < 1024 * 1024 then s.value = s.value .. s.value end
end)
s.value = "ab"
assert(scalls < 1000, "string listener ran " .. scalls .. " times")
return {}
