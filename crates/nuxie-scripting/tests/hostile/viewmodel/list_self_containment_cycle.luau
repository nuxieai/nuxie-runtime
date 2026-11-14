--! attacks: a view model pushed into its own list, two nodes in each other's lists, then length/charge walks and drop; expect: ok; within: 15s; viewmodels: data_bind_blob_test.riv
local a = Data.XMLNodeVM.new()
local b = Data.XMLNodeVM.new()
for i = 1, 3 do pcall(function() a:getList("children"):push(a) end) end
pcall(function() a:getList("children"):push(b) end)
pcall(function() b:getList("children"):push(a) end)
pcall(function() b:getList("attributes"):push(b) end)
-- every growth charge walks the item (bounded walk); a cyclic item must still terminate
for i = 1, 20 do pcall(function() b:getList("children"):push(a) end) end
local l = a:getList("children").length + b:getList("children").length
-- read items back through the cycle
local cur = a
for i = 1, 1000 do
  local list = cur:getList("children")
  if list.length == 0 then break end
  local ok, item = pcall(function() return list[1] end)
  if not ok or type(item) ~= "table" then break end
  cur = item
end
a, b, cur = nil, nil, nil
return {}
