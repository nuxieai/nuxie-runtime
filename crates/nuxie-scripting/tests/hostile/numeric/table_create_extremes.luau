--! attacks: table.create and table.pack/unpack sizing with huge, negative, NaN and 2^32 counts; expect: error(not enough memory); within: 20s
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
refused("create 2^31", table.create, 2 ^ 31)
refused("create 2^32", table.create, 2 ^ 32)
refused("create 2^53", table.create, 2 ^ 53, 0)
refused("create -1", table.create, -1)
refused("create -2^31", table.create, -(2 ^ 31))
refused("create inf", table.create, inf)
try("create nan", table.create, nan)
try("create 0", table.create, 0, "x")
local t = table.create(1000, "v")
assert(#t == 1000)
-- 2^23 slots with a value: 128 MiB of slots, over the VM heap
local big = table.create(2 ^ 23, true)
error("2^23-slot table was allocated: " .. #big)
