--! attacks: pcall(error, t) where t's __tostring calls host bindings and raises again; expect: ok; within: 10s
local calls = 0
local obj = setmetatable({}, { __tostring = function()
  calls += 1
  local p = Path.new()
  p:moveTo(Vector.xy(0, 0))
  p:lineTo(Vector.xy(calls, calls))
  local paint = Paint.with({ color = 0xff00ff00, thickness = calls })
  local g = Gradient.linear(Vector.xy(0, 0), Vector.xy(1, 1), { { position = 0, color = 0xff000000 }, { position = 1, color = 0xffffffff } })
  paint.gradient = g
  return "obj#" .. #p .. ":" .. paint.thickness
end })
local ok, err = pcall(error, obj)
assert(not ok and rawequal(err, obj), "pcall(error, t) returns t itself")
assert(string.match(tostring(err), "^obj#2:%d+$"), tostring(err))
-- Error object whose __tostring raises an error object whose __tostring raises.
local deep = setmetatable({}, { __tostring = function()
  error(setmetatable({}, { __tostring = function() error("innermost") end }))
end })
ok, err = pcall(error, deep)
assert(not ok and rawequal(err, deep))
local ok2, err2 = pcall(tostring, err)
assert(not ok2)
-- xpcall whose handler tostrings the object, and the handler raises.
local ok3, err3 = xpcall(error, function(e) return tostring(e) end, obj)
assert(not ok3 and string.match(err3, "^obj#2:%d+$"), tostring(err3))
local ok4 = xpcall(error, function(e) return tostring(deep) end, obj)
assert(not ok4)
return {}
