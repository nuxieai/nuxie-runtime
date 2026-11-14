--! attacks: many large MeshInstances held together; expect: ok; within: 30s
local kept = {}
local ok, err
for i = 1, 100 do
  ok, err = pcall(function() kept[i] = MeshInstances(200000) end)
  if not ok then break end
end
assert(not ok, "100 x 200k MeshInstances were all admitted")
local e = tostring(err)
assert(string.find(e, "host memory budget", 1, true) or string.find(e, "MeshInstances may hold at most", 1, true), e)
-- Growing an admitted one past the cap through resize is refused too.
local ok2, err2 = pcall(function() kept[1]:resize(4294967295) end)
assert(not ok2, "resize to 2^32-1 instances was admitted")
local e2 = tostring(err2)
assert(string.find(e2, "host memory budget", 1, true) or string.find(e2, "MeshInstances may hold at most", 1, true), e2)
return {}
