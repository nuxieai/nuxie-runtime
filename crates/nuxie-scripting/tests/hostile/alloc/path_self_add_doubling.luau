--! attacks: Path:add(self) doubling past the host budget; expect: ok; within: 20s
-- E1 section 5: p:add(p) doubles the path each call, so 30 calls ask for 2^30 commands.
local p = Path.new()
p:moveTo(Vector.xy(0, 0))
for i = 1, 64 do p:lineTo(Vector.xy(i, i)) end
local ok, err
for _ = 1, 40 do
  ok, err = pcall(function() p:add(p) end)
  if not ok then break end
end
assert(not ok, "40 doublings of a path never hit the host memory budget")
assert(string.find(tostring(err), "host memory budget", 1, true), tostring(err))
-- The path stays usable after the refusal.
p:lineTo(Vector.xy(1, 1))
assert(#p > 0)
return {}
