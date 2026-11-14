--! attacks: ContourMeasure positionAndTangent, warp and extract at NaN/inf distances on an ordinary path; expect: ok; within: 10s
local function safe(f, ...)
  local ok, err = pcall(f, ...)
  assert(ok or not string.find(tostring(err), "rust panic", 1, true), tostring(err))
end
local p = Path.new()
p:moveTo(Vector.xy(0, 0))
p:lineTo(Vector.xy(10, 0))
p:cubicTo(Vector.xy(20, 10), Vector.xy(30, -10), Vector.xy(40, 0))
local c = p:contours()
for _, d in ipairs({0 / 0, math.huge, -math.huge, -1, 1e30}) do
  safe(function() c:positionAndTangent(d) end)
  safe(function() c:warp(Vector.xy(d, d)) end)
  safe(function() c:extract(d, -d, Path.new()) end)
  safe(function() c:extract(0, d, Path.new(), false) end)
end
return {}
