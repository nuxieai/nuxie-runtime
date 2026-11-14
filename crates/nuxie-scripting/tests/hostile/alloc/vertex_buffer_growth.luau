--! attacks: VertexBuffer:add with thousands of vectors per call past the host budget; expect: ok; within: 30s
local batch = {}
for i = 1, 7000 do batch[i] = Vector.xy(i, -i) end
local vb = VertexBuffer()
local ok, err
for _ = 1, 10000 do
  ok, err = pcall(function() vb:add(table.unpack(batch)) end)
  if not ok then break end
end
assert(not ok, "70M vertices never hit the budget")
assert(string.find(tostring(err), "host memory budget", 1, true), tostring(err))
vb:reset()
vb:add(Vector.xy(1, 1))
return {}
