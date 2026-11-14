--! attacks: property and list-item handles used after their instance is replaced, popped, cleared or collected; expect: ok; within: 20s; viewmodels: script_create_viewmodel_instance.riv
local main = Data.Main.new()
local chi = main:getViewModel("chi")
local old = chi.value
local oldLabel = old:getString("label")
oldLabel:addListener(function() end)
-- replace the nested instance, then use the old handles
chi.value = Data.child.new()
oldLabel.value = "stale"
local _ = oldLabel.value
local sub = old:getViewModel("subch")
sub.value = Data.SubChild.new()
-- pop and clear list items, then use them
local settings = Data.VMSettings.new()
local list = settings:getList("lis")
for i = 1, 4 do list:push(Data.child.new()) end
local first = list[1]
local popped = list:pop()
list:clear()
for _, item in ipairs({first, popped}) do
  if type(item) == "table" then
    pcall(function() item:getString("label").value = "after clear" end)
    pcall(function() return item:getString("label").value end)
    pcall(function() list:push(item) end)
  end
end
-- drop the instance and churn the heap so it is collected, then use the wrapper
local orphan = Data.SubChild.new():getNumber("num")
orphan:addListener(function() end)
for i = 1, 2000 do local garbage = table.create(512, i) end
orphan.value = 7
local _ = orphan.value
-- a list wrapper outliving its instance
local lw = Data.VMSettings.new():getList("lis")
for i = 1, 2000 do local garbage = table.create(512, i) end
pcall(function() lw:push(Data.child.new()) end)
local _ = lw.length
pcall(function() return lw:pop() end)
return {}
