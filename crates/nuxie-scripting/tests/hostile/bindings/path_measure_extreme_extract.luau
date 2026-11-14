--! attacks: PathMeasure:extract and positionAndTangent at inf/negative/huge (non-NaN) distances; expect: ok; within: 10s
local function safe(f, ...)
  local ok, err = pcall(f, ...)
  assert(ok or not string.find(tostring(err), "rust panic", 1, true), tostring(err))
end
local p = Path.new()
p:moveTo(Vector.xy(0, 0))
p:quadTo(Vector.xy(5, 10), Vector.xy(10, 0))
p:close()
p:moveTo(Vector.xy(20, 20)) p:lineTo(Vector.xy(30, 30))
local m = p:measure()
for _, d in ipairs({math.huge, -math.huge, -1, 1e30, -1e30, 0}) do
  safe(function() m:positionAndTangent(d) end)
  safe(function() m:warp(Vector.xy(d, 1)) end)
  m:extract(d, -d, Path.new())
  m:extract(-d, d, Path.new(), false)
end
return {}
