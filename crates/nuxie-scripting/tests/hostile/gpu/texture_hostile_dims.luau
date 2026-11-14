--! attacks: GPUTexture.new with huge/zero/negative/NaN dimensions, layers, mipmaps, sample counts, unknown formats/types, without an ORE context; expect: ok; within: 10s
local dims = {0, -1, 0 / 0, math.huge, 1, 8192, 8193, 2 ^ 16, 2 ^ 32 - 1, 2 ^ 32, 2 ^ 53}
local formats = {nil, "rgba8unorm", "bc7unorm", "astc8x8", "depth24plusStencil8", "rgba32float", "nope", ""}
local n = 0
for _, w in ipairs(dims) do
  for _, h in ipairs(dims) do
    for fi = 1, #formats + 1 do
      local ok, err = pcall(GPUTexture.new, {
        width = w, height = h, format = formats[fi],
        type = (fi % 2 == 0) and "cube" or "3d-volume",
        layers = 2 ^ 32 - 1, mipmaps = 2 ^ 32 - 1, sampleCount = 0 / 0,
        renderTarget = true,
      })
      assert(not ok, "GPUTexture.new accepted " .. tostring(w) .. "x" .. tostring(h))
      assert(string.find(tostring(err), "GPU context not initialized", 1, true), tostring(err))
      n += 1
    end
  end
end
assert(not pcall(GPUTexture.new))
assert(not pcall(GPUTexture.new, {}))
return {}
