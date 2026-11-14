--! attacks: a string.gsub replacement callback that never returns, under pcall; expect: error(exceeded); within: 15s
pcall(string.gsub, "abc", ".", function() while true do end end)
error("gsub callback never hit the budget: exceeded nothing")
