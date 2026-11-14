--! attacks: list insert/removeAt/swap at negative, zero and huge indices, pop/shift/remove on an empty list; expect: ok; within: 10s; viewmodels: data_bind_blob_test.riv
local root = Data.Main.new()
local list = root:getList("children")
-- empty list
assert(select("#", list:pop()) == 0)
assert(select("#", list:shift()) == 0)
list:remove(Data.XMLNodeVM.new())
list:removeAllOf(Data.XMLNodeVM.new())
list:clear()
for _, i in ipairs({0, 1, 2, 2 ^ 31, 2 ^ 32, 2 ^ 53}) do
  assert(not pcall(function() list:removeAt(i) end), "removeAt " .. i .. " on empty")
end
for _, i in ipairs({-1, 0 / 0, -math.huge, math.huge}) do
  pcall(function() list:removeAt(i) end)
  pcall(function() list:insert(Data.XMLNodeVM.new(), i) end)
  pcall(function() list:swap(i, 1) end)
end
for _, i in ipairs({0, 1, 2, 1000, 2 ^ 31, 2 ^ 32, 2 ^ 53}) do
  pcall(function() list:insert(Data.XMLNodeVM.new(), i) end)
  pcall(function() list:swap(i, i + 1) end)
  pcall(function() list:swap(1, i) end)
end
local n = list.length
assert(n <= 11, "length " .. n)
-- wrong item types
-- table.pack keeps the nil that ipairs would stop at.
local args1 = table.pack(nil, 1, "x", {}, setmetatable({}, {__index = function() error("trap") end}), Data.Main.new())
for i1 = 1, args1.n do local v = args1[i1]
  pcall(function() list:push(v) end)
  pcall(function() list:insert(v, 1) end)
end
while list.length > 0 do list:pop() end
assert(select("#", list:pop()) == 0)
return {}
