--! attacks: a table.sort comparator that never returns, called under pcall in a retry loop; expect: error(exceeded); within: 15s
local t = {}
for i = 1, 100 do t[i] = 101 - i end
while true do
  pcall(table.sort, t, function(a, b) while true do end end)
end
