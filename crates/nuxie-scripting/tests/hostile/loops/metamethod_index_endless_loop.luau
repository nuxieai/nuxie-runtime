--! attacks: an __index function that loops forever, reached under pcall in a retry loop; expect: error(exceeded); within: 15s
local v = setmetatable({}, { __index = function() while true do end end })
while true do pcall(function() return v.x end) end
