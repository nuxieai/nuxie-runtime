--! attacks: buffer.create with huge, negative, NaN, inf and 2^32 sizes, and buffer.fromstring/tostring round trips near the limits; expect: error(not enough memory); within: 20s
local LOG = {}
local function try(name, f, ...)
  local r = table.pack(pcall(f, ...))
  local err = tostring(r[2])
  assert(not string.find(err, "rust panic", 1, true), name .. ": " .. err)
  table.insert(LOG, name .. " => " .. tostring(r[1]) .. " " .. string.sub(err, 1, 120))
  return r[1], r[2], r
end
local function refused(name, f, ...)
  local ok, err = try(name, f, ...)
  assert(not ok, name .. ": accepted")
  return tostring(err)
end
local nan, inf = 0 / 0, math.huge
refused("create 2^31", buffer.create, 2 ^ 31)
refused("create 2^32", buffer.create, 2 ^ 32)
refused("create 2^32+1", buffer.create, 2 ^ 32 + 1)
refused("create 2^53", buffer.create, 2 ^ 53)
refused("create -1", buffer.create, -1)
refused("create -inf", buffer.create, -inf)
refused("create inf", buffer.create, inf)
try("create nan", buffer.create, nan)
try("create -0", buffer.create, -0)
local b = buffer.create(1024 * 1024)
assert(buffer.len(b) == 1048576)
assert(#buffer.tostring(b) == 1048576)
assert(buffer.len(buffer.fromstring(string.rep("z", 100000))) == 100000)
-- 96 MiB: within the size limit, over the VM heap
local huge = buffer.create(96 * 1024 * 1024)
error("96 MiB buffer was allocated: " .. buffer.len(huge))
