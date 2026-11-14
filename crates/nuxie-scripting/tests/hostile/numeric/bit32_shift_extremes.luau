--! attacks: bit32 shifts and rotates by >=32, negative, 2^31, NaN and inf amounts, extract/replace with out-of-range fields, and NaN/inf/2^53 operands; expect: ok; within: 20s
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
local function u32(name, f, ...)
  local ok, v = try(name, f, ...)
  if ok then assert(type(v) == "number" and v >= 0 and v < 2 ^ 32 and v == math.floor(v), name .. " = " .. tostring(v)) end
end
for _, n in ipairs({ 31, 32, 33, 64, 2 ^ 31, 2 ^ 53, -1, -32, -33, -(2 ^ 31), -(2 ^ 53), nan, inf, -inf }) do
  for _, f in ipairs({ "lshift", "rshift", "arshift", "lrotate", "rrotate" }) do
    u32(f .. " " .. n, bit32[f], 0x80000001, n)
  end
end
assert(bit32.lshift(1, 32) == 0 and bit32.rshift(1, 32) == 0)
assert(bit32.arshift(0x80000000, 100) == 0xFFFFFFFF)
for _, v in ipairs({ nan, inf, -inf, 2 ^ 53, 2 ^ 64, -(2 ^ 63), -0, 0.5 }) do
  u32("band " .. tostring(v), bit32.band, v)
  u32("bnot " .. tostring(v), bit32.bnot, v)
  u32("countlz " .. tostring(v), bit32.countlz, v)
  u32("countrz " .. tostring(v), bit32.countrz, v)
  u32("byteswap " .. tostring(v), bit32.byteswap, v)
end
refused("extract 31 2", bit32.extract, 1, 31, 2)
refused("extract -1", bit32.extract, 1, -1)
refused("extract w 0", bit32.extract, 1, 0, 0)
refused("extract w 33", bit32.extract, 1, 0, 33)
refused("extract 2^31", bit32.extract, 1, 2 ^ 31)
refused("replace 32", bit32.replace, 1, 1, 32)
refused("replace w -1", bit32.replace, 1, 1, 0, -1)
try("extract nan", bit32.extract, 1, nan)
try("replace nan width", bit32.replace, 1, 1, 0, nan)
u32("band 7000 args", bit32.band, table.unpack(table.create(7000, 0xFFFFFFFF)))
return {}
