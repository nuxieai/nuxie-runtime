--! attacks: PathMeasure:positionAndTangent and warp at a NaN distance on an ordinary path; expect: ok; within: 10s
local function safe(f, ...)
  local ok, err = pcall(f, ...)
  assert(ok or not string.find(tostring(err), "rust panic", 1, true), tostring(err))
end
local p = Path.new()
p:moveTo(Vector.xy(0, 0))
p:lineTo(Vector.xy(10, 0))
local m = p:measure()
assert(m.length == 10)
-- Any of: a result, or an ordinary error. Never a host panic.
safe(function() m:positionAndTangent(0 / 0) end)
safe(function() m:warp(Vector.xy(0 / 0, 1)) end)
return {}
