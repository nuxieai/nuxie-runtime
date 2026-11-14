--! attacks: ContourMeasure/PathMeasure extract appending into one destination past the budget; expect: ok; within: 30s
local p = Path.new()
p:moveTo(Vector.xy(0, 0))
for i = 1, 400 do p:cubicTo(Vector.xy(i, -40), Vector.xy(i + 0.5, 40), Vector.xy(i + 1, 0)) end
local measure = p:measure()
local contour = p:contours()
local dst = Path.new()
local ok, err
for i = 1, 40000 do
  ok, err = pcall(function()
    if i % 2 == 0 then
      measure:extract(0, measure.length, dst)
    else
      contour:extract(0, contour.length, dst, false)
    end
  end)
  if not ok then break end
end
assert(not ok, "40000 full extracts into one path never hit the budget")
assert(string.find(tostring(err), "host memory budget", 1, true), tostring(err))
return {}
