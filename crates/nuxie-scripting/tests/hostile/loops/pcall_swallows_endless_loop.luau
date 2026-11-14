--! attacks: while true do pcall(function() while true do end end) end (does the budget error escape pcall or get swallowed forever); expect: error(exceeded); within: 15s
while true do
  pcall(function() while true do end end)
end
