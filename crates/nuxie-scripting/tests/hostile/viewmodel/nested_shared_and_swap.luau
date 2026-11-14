--! attacks: nested view models: wrong-type values, one child under two parents, self-assign from a slot listener, swapping slots between parents; expect: ok; within: 15s; viewmodels: script_create_viewmodel_instance.riv
local main = Data.Main.new()
local chi = main:getViewModel("chi")
for _, v in ipairs({Data.SubChild.new(), {}, setmetatable({}, {__index = function() error("trap") end})}) do
  pcall(function() chi.value = v end)
end
assert(not pcall(function() chi.value = 1 end))
pcall(function() chi.value = nil end)
-- the same child under two parents
local c = Data.child.new()
local m2 = Data.Main.new()
main:getViewModel("chi").value = c
m2:getViewModel("chi").value = c
-- self assign repeatedly from a listener on the slot
local n = 0
chi:addListener(function() n += 1 chi.value = chi.value end)
chi.value = c
assert(n < 1000, "self-assign listener ran " .. n)
-- swap settings between two parents
local s1, s2 = main:getViewModel("settings"), m2:getViewModel("settings")
s1.value, s2.value = s2.value, s1.value
main, m2, c = nil, nil, nil
return {}
