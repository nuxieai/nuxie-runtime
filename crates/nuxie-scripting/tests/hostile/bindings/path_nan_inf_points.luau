--! attacks: Path commands, measure, contours and extract with NaN/inf/huge points and inf distances; expect: ok; within: 20s
local nan, inf = 0 / 0, math.huge
local p = Path.new()
p:moveTo(Vector.xy(nan, inf))
p:lineTo(Vector.xy(-inf, nan))
p:quadTo(Vector.xy(nan, nan), Vector.xy(inf, -inf))
p:cubicTo(Vector.xy(3e38, -3e38), Vector.xy(nan, 0), Vector.xy(1e-45, -1e-45))
p:close()
p:lineTo(Vector.xy(0, 0)) -- after close, no moveTo
p:cubicTo(Vector.xy(1, 1), Vector.xy(2, 2), Vector.xy(3e38, 3e38))
local m = p:measure()
local _ = m.length
-- NaN distances are path_measure_nan_distance / contour_measure_nan_distance.
for _, d in ipairs({inf, -inf, -1, 0, 3e38, -3e38}) do
  local dst = Path.new()
  m:extract(d, -d, dst)
  m:extract(-inf, inf, dst, true)
end
local c = p:contours()
local guard = 0
while c and guard < 100 do
  local _ = c.length, c.isClosed
  c:extract(-inf, inf, Path.new(), false)
  c = c.next
  guard += 1
end
-- A path of only huge curves, doubled, then measured.
local q = Path.new()
q:moveTo(Vector.xy(-3e38, -3e38))
q:cubicTo(Vector.xy(3e38, -3e38), Vector.xy(-3e38, 3e38), Vector.xy(3e38, 3e38))
for _ = 1, 8 do q:add(q) end
local qm = q:measure()
qm:extract(0, qm.length, Path.new())
return {}
