--! attacks: GPUBuffer.new with huge/zero/negative/NaN sizes, 2^16/2^32 edges, bad usages and data, without an ORE context; expect: ok; within: 10s
local sizes = {0, -1, 0 / 0, math.huge, -math.huge, 0.5, 1, 2 ^ 16, 2 ^ 16 + 1, 2 ^ 32 - 1, 2 ^ 32, 2 ^ 53, "64", true}
local usages = {"uniform", "vertex", "index", "storage", "", {}, {"vertex", "index"}, {"nope"}, 7}
local data = buffer.create(16)
for _, size in ipairs(sizes) do
  for _, usage in ipairs(usages) do
    for _, d in ipairs({false, true}) do
      local ok, err = pcall(GPUBuffer.new, {size = size, usage = usage, data = d and data or nil, immutable = d, label = string.rep("l", 4096)})
      assert(not ok, "GPUBuffer.new accepted size " .. tostring(size))
      assert(string.find(tostring(err), "GPU context not initialized", 1, true), tostring(err))
    end
  end
end
assert(not pcall(GPUBuffer.new))
assert(not pcall(GPUBuffer.new, "size"))
assert(not pcall(GPUBuffer.new, 64))
return {}
