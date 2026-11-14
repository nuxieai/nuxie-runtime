--! attacks: GPUTarget (empty readonly table without a host target) and GPUPipeline/GPUBindGroupLayout with hostile descriptors; expect: ok; within: 10s
assert(type(GPUTarget) == "table")
assert(GPUTarget.width == nil and GPUTarget.view == nil)
assert(not pcall(function() GPUTarget.width = 1 end), "GPUTarget is writable")
assert(not pcall(function() return GPUTarget:beginRenderPass({}) end))
assert(not pcall(function() return GPUTarget() end))
assert(not pcall(setmetatable, GPUTarget, {}))
local deep = {}
for i = 1, 5000 do deep = {vertex = deep, fragment = deep} end
local cyc = {}
cyc.vertex = cyc
cyc.fragment = cyc
cyc.layouts = {cyc, cyc}
for _, d in ipairs({deep, cyc, {vertex = 1, fragment = "x", targets = table.create(1000, {format = "nope"})}}) do
  local ok, err = pcall(GPUPipeline.new, d)
  assert(not ok)
  assert(string.find(tostring(err), "GPU context not initialized", 1, true), tostring(err))
  assert(not pcall(GPUBindGroupLayout.new, d))
end
for _, g in ipairs({-1, 4, 2 ^ 32, 0 / 0}) do
  assert(not pcall(GPUBindGroupLayout.new, {groupIndex = g, shader = Path.new()}))
end
return {}
