--! attacks: buffer.readbits/writebits with bit counts 0, 32, 33, -1, 2^31, NaN, bit offsets past the end, 2^35 and 2^53, and values past 32 bits; expect: ok; within: 20s
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
local b = buffer.create(8)
local bits = 64
for _, c in ipairs({ 33, 64, -1, 2 ^ 31, 2 ^ 32, inf, -inf }) do
  refused("readbits count " .. c, buffer.readbits, b, 0, c)
  refused("writebits count " .. c, buffer.writebits, b, 0, c, 1)
end
for _, off in ipairs({ -1, bits - 1 + 0, bits, 2 ^ 31, 2 ^ 32 - 1, 2 ^ 35, 2 ^ 53, inf }) do
  refused("readbits off " .. off .. " count 2", buffer.readbits, b, off, 2)
  refused("writebits off " .. off .. " count 2", buffer.writebits, b, off, 2, 3)
end
assert(buffer.readbits(b, 0, 0) == 0)
try("readbits off 64 count 0", buffer.readbits, b, bits, 0)
buffer.writebits(b, bits - 32, 32, 0xFFFFFFFF)
assert(buffer.readbits(b, bits - 32, 32) == 0xFFFFFFFF)
buffer.writebits(b, 3, 32, 2 ^ 53)
buffer.writebits(b, 5, 31, -1)
try("writebits value nan", buffer.writebits, b, 0, 8, nan)
try("writebits value inf", buffer.writebits, b, 0, 8, inf)
try("readbits off nan", buffer.readbits, b, nan, 1)
try("readbits count nan", buffer.readbits, b, 0, nan)
try("writebits count nan", buffer.writebits, b, 0, nan, 1)
-- unaligned 32-bit windows at every bit offset
for off = 0, bits - 32 do buffer.writebits(b, off, 32, off) assert(buffer.readbits(b, off, 32) == off) end
return {}
