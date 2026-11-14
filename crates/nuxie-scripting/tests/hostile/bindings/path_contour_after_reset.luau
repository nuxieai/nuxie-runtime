--! attacks: contour and measure handles used after their source path is reset and regrown; expect: ok; within: 10s
local p = Path.new()
for i = 1, 5 do p:moveTo(Vector.xy(i, 0)) p:lineTo(Vector.xy(i, 10)) end
local c = p:contours()
local m = p:measure()
p:reset()
p:moveTo(Vector.xy(100, 100))
for _ = 1, 10 do p:add(p) end
local n = 0
while c do
  c:positionAndTangent(1)
  c:extract(0, 5, p)
  c = c.next
  n += 1
end
assert(n == 5, "iterator saw " .. n .. " contours")
m:extract(0, m.length, p)
p:reset()
m:positionAndTangent(3)
return {}
