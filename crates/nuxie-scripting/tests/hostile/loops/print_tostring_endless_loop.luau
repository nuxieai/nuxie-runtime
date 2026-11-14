--! attacks: print of a value whose __tostring loops forever (loop inside a host callback's script call), from a draw callback; expect: error(exceeded); within: 15s; harness: instance
local v = setmetatable({}, { __tostring = function() while true do end end })
return function()
  return {
    draw = function(self, renderer) pcall(print, "a", v, "b") end,
  }
end
