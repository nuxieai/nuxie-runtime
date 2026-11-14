--! attacks: GPUSampler.new with hostile LODs, anisotropy, filters and non-table descriptors, without an ORE context; expect: ok; within: 10s
for _, d in ipairs({
  nil, 1, "s", {},
  {minLod = 0 / 0, maxLod = -math.huge},
  {minLod = 1e300, maxLod = -1e300},
  {maxAnisotropy = 2 ^ 32},
  {maxAnisotropy = -1},
  {maxAnisotropy = 0 / 0},
  {min = "cubic", mag = {}, mipmap = 3, wrapU = "spiral", wrapV = "", wrapW = string.rep("w", 65536), compare = "sometimes"},
}) do
  local ok, err = pcall(GPUSampler.new, d)
  assert(not ok, "GPUSampler.new accepted")
end
-- The context check comes first for every descriptor shape.
local ok, err = pcall(GPUSampler.new, {})
assert(string.find(tostring(err), "GPU context not initialized", 1, true), tostring(err))
return {}
