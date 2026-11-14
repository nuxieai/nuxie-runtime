--! attacks: many DataValue.string values each holding a copy of one 4 MiB string; expect: ok; within: 30s
local s = string.rep("v", 4 * 1024 * 1024)
local kept = {}
local ok, err
for i = 1, 100 do
  ok, err = pcall(function()
    local d = DataValue.string()
    d.value = s
    kept[i] = d
  end)
  if not ok then break end
end
assert(not ok, "100 copies of a 4 MiB string never hit the budget")
assert(string.find(tostring(err), "host memory budget", 1, true), tostring(err))
return {}
