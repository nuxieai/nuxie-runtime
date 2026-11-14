--! attacks: Path:add with NaN/inf/singular Mat2D transforms, self and other, then measured; expect: ok; within: 20s
local nan, inf = 0 / 0, math.huge
local p = Path.new()
p:moveTo(Vector.xy(0, 0))
p:cubicTo(Vector.xy(10, 0), Vector.xy(0, 10), Vector.xy(10, 10))
p:close()
for _, m in ipairs({
  Mat2D.values(nan, nan, nan, nan, nan, nan),
  Mat2D.values(inf, -inf, inf, -inf, inf, -inf),
  Mat2D.values(0, 0, 0, 0, 0, 0),
  Mat2D.values(3e38, 3e38, 3e38, 3e38, 3e38, 3e38),
}) do
  p:add(p, m)
  local other = Path.new()
  other:add(p, m)
  other:add(other, m)
  local measure = other:measure()
  local _ = measure.length
  measure:extract(0, inf, Path.new())
end
local c = p:contours()
while c do c = c.next end
return {}
