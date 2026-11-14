--! attacks: GPUBindGroup.new (no context check before descriptor reads): __index that recurses into the constructor, yields, or errors, and wrong userdata as layout; expect: ok; within: 15s
-- Unbounded host recursion through descriptor __index must end in an ordinary error.
local depth = 0
local recursive
recursive = setmetatable({}, {__index = function(t, k)
  depth += 1
  return GPUBindGroup.new(recursive)
end})
local ok, err = pcall(GPUBindGroup.new, recursive)
assert(not ok, "recursive descriptor accepted")
-- a yielding __index
local co = coroutine.create(function()
  return GPUBindGroup.new(setmetatable({}, {__index = function() coroutine.yield(1) return nil end}))
end)
local okc = coroutine.resume(co)
co = nil
-- an erroring __index
assert(not pcall(GPUBindGroup.new, setmetatable({}, {__index = function() error("descriptor trap") end})))
-- non-layout values in `layout`
for _, layout in ipairs({nil, 1, "x", {}, Audio.Ring.create(8), Path.new(), buffer.create(4)}) do
  assert(not pcall(GPUBindGroup.new, {layout = layout, ubos = {{slot = 2 ^ 32, buffer = layout}}, textures = {}, samplers = {}}))
end
assert(not pcall(GPUBindGroup.new))
assert(not pcall(GPUBindGroup.new, 1))
-- Same recursion through the other constructors' descriptors (they check the context first)
for _, ctor in ipairs({GPUBuffer.new, GPUTexture.new, GPUPipeline.new, GPUBindGroupLayout.new}) do
  local r
  r = setmetatable({}, {__index = function() return ctor(r) end})
  assert(not pcall(ctor, r))
end
return {}
