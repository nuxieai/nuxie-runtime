--! attacks: Path methods given Vector-like tables whose __index re-enters the same path, Path.add of a path to itself, and extract into the measured path; expect: ok; within: 10s
local path = Path.new()
path:moveTo(Vector.xy(0, 0))
path:lineTo(Vector.xy(10, 0))
local proxy = setmetatable({}, { __index = function(_, k)
  path:reset()
  path:moveTo(Vector.xy(1, 1))
  path:lineTo(Vector.xy(5, 5))
  return 3
end })
local ok = pcall(path.lineTo, path, proxy)
assert(not ok, "a table is not a Vector")
ok = pcall(path.moveTo, path, proxy)
assert(not ok)
-- Path.add of itself, repeatedly (doubles; bounded).
for _ = 1, 8 do path:add(path) end
assert(#path > 0)
-- Path.add with a transform argument of the wrong kind whose metamethods add to the path.
ok = pcall(path.add, path, path, setmetatable({}, { __index = function() path:add(path) end }))
assert(not ok)
-- Measure a path and extract its segments into the path being measured.
local small = Path.new()
small:moveTo(Vector.xy(0, 0))
small:lineTo(Vector.xy(100, 0))
small:lineTo(Vector.xy(100, 100))
local contour = small:contours()
local n = 0
while contour and n < 20 do
  n += 1
  contour:extract(0, 50, small)
  contour = contour.next
end
local measure = small:measure()
ok = pcall(measure.extract, measure, 0, 10, small)
-- __len via the host while the path is used as its own add argument inside a sort comparator.
local paths = { small, path, small }
table.sort(paths, function(a, b) a:add(b); return #a < #b end)
return {}
