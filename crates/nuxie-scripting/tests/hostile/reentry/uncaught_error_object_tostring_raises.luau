--! attacks: an uncaught error object whose __tostring re-raises and calls host bindings when the host formats it; expect: error; within: 10s
local function raiser()
  local p = Path.new()
  p:moveTo(Vector.xy(1, 2))
  error(setmetatable({}, { __tostring = function() error("again") end }))
end
error(setmetatable({}, { __tostring = function()
  local paint = Paint.new()
  paint.color = 0xff112233
  raiser()
  return "unreachable"
end }))
