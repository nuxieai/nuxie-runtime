--! attacks: many GPU resource creations in a loop (budget), each refused, without an ORE context; expect: ok; within: 15s
local ctors = {
  function() return GPUBuffer.new({size = 65536, usage = "uniform"}) end,
  function() return GPUTexture.new({width = 4096, height = 4096, format = "rgba32float"}) end,
  function() return GPUSampler.new({}) end,
  function() return GPUBindGroupLayout.new({groupIndex = 0}) end,
  function() return GPUPipeline.new({}) end,
}
local refused = 0
for i = 1, 8000 do
  local ok = pcall(ctors[(i % #ctors) + 1])
  if not ok then refused += 1 end
end
assert(refused == 8000, "a GPU constructor succeeded without a context: " .. refused)
return {}
