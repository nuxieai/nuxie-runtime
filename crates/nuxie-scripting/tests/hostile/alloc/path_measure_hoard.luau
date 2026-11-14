--! attacks: holding many Path:measure() tables of one large path; expect: ok; within: 30s
-- E2b left measures uncharged; G4 charges each contour's tables. A script keeps
-- every measure alive so only the budget can stop it.
local p = Path.new()
p:moveTo(Vector.xy(0, 0))
for i = 1, 200 do p:cubicTo(Vector.xy(i * 3, -50), Vector.xy(i * 3 + 1, 50), Vector.xy(i * 3 + 2, 0)) end
for _ = 1, 6 do p:add(p) end -- 12,800 cubics
local kept = {}
local ok, err
for i = 1, 20000 do
  ok, err = pcall(function() kept[i] = p:measure() end)
  if not ok then break end
end
assert(not ok, "20000 measures of a 12,800-cubic path never hit the budget")
assert(string.find(tostring(err), "host memory budget", 1, true), tostring(err))
assert(kept[1].length > 0)
return {}
