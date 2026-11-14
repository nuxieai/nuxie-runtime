--! attacks: holding many Path:contours() iterators (each copies the path); expect: ok; within: 30s
local p = Path.new()
for c = 1, 50 do
  p:moveTo(Vector.xy(c, 0))
  for i = 1, 40 do p:quadTo(Vector.xy(c + i, 10), Vector.xy(c + i, 0)) end
  p:close()
end
for _ = 1, 6 do p:add(p) end -- 3,200 contours, 128k quads
local kept = {}
local ok, err
for i = 1, 20000 do
  ok, err = pcall(function()
    local first = p:contours()
    kept[i] = first
    local second = first.next -- each contour is admitted on its own
    kept[-i] = second
  end)
  if not ok then break end
end
assert(not ok, "20000 contour iterators of a large path never hit the budget")
assert(string.find(tostring(err), "host memory budget", 1, true), tostring(err))
return {}
