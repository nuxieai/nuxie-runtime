--! attacks: contour iterators, contour measures and path measures used after their path was reset, extended or dropped; expect: ok; within: 10s
local path = Path.new()
path:moveTo(Vector.xy(0, 0))
path:lineTo(Vector.xy(100, 0))
path:moveTo(Vector.xy(0, 10))
path:lineTo(Vector.xy(50, 10))
local contour = path:contours()
local measure = path:measure()
assert(contour ~= nil)
path:reset()
-- The iterator and measure copied the path: still usable, never a dangling read.
local n = 0
local c = contour
while c and n < 10 do
  n += 1
  c:positionAndTangent(5)
  c:warp(Vector.xy(1, 1))
  c:extract(0, 10, path)
  c = c.next
end
measure:positionAndTangent(10)
measure:extract(0, 20, path)
-- Advance a contour iterator twice from two handles to the same iterator.
local p2 = Path.new()
for i = 1, 6 do p2:moveTo(Vector.xy(0, i)); p2:lineTo(Vector.xy(10, i)) end
local first = p2:contours()
local a, b = first.next, first.next
-- Drop the source path and every other reference, churn, then use the handles.
local orphan
do
  local p3 = Path.new()
  p3:moveTo(Vector.xy(0, 0)); p3:lineTo(Vector.xy(5, 5)); p3:close()
  orphan = { p3:contours(), p3:measure() }
end
for _ = 1, 10 do local junk = {}; for j = 1, 300 do junk[j] = { j } end end
orphan[1]:positionAndTangent(1)
orphan[2]:positionAndTangent(1)
local _ = orphan[1].next
-- Extract a segment into a path that is being iterated by its own contours.
local self = Path.new()
self:moveTo(Vector.xy(0, 0)); self:lineTo(Vector.xy(100, 0))
local sc = self:contours()
for _ = 1, 5 do sc:extract(0, 100, self) end
local after = self:contours()
local count = 0
while after and count < 50 do count += 1; after = after.next end
return {}
