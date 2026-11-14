--! attacks: Path:add(other) and add with a transform growing one path past the host budget; expect: ok; within: 20s
local src = Path.new()
src:moveTo(Vector.xy(0, 0))
for i = 1, 100 do src:cubicTo(Vector.xy(i, 0), Vector.xy(0, i), Vector.xy(i, i)) end
for _ = 1, 10 do src:add(src) end -- about 100k commands
local dst = Path.new()
local m = Mat2D.values(2, 0, 0, 2, 5, 5)
local ok, err
for i = 1, 5000 do
  ok, err = pcall(function()
    if i % 2 == 0 then dst:add(src, m) else dst:add(src) end
  end)
  if not ok then break end
end
assert(not ok, "adding a 100k-command path 5000 times never hit the budget")
assert(string.find(tostring(err), "host memory budget", 1, true), tostring(err))
return {}
