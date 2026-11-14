--! attacks: buffer read/write at offsets -1, 2^31, 2^32-1, 2^32, NaN, inf, len-1 for every width, and readstring/writestring with forged counts; expect: ok; within: 20s
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
local b = buffer.create(16)
local n = buffer.len(b)
local widths = {
  { "i8", 1 }, { "u8", 1 }, { "i16", 2 }, { "u16", 2 }, { "i32", 4 }, { "u32", 4 }, { "f32", 4 }, { "f64", 8 },
}
for _, w in ipairs(widths) do
  local name, size = w[1], w[2]
  local rd, wr = buffer["read" .. name], buffer["write" .. name]
  for _, off in ipairs({ -1, -size, n - size + 1, n, 2 ^ 31, 2 ^ 31 - 1, 2 ^ 32 - 1, 2 ^ 32, 2 ^ 53, inf, -inf }) do
    refused(name .. " read " .. off, rd, b, off)
    refused(name .. " write " .. off, wr, b, off, 1)
  end
  -- the last in-bounds offset works
  wr(b, n - size, 1)
  assert(rd(b, n - size) == 1, name)
  try(name .. " read nan", rd, b, nan)
  try(name .. " write nan", wr, b, nan, 1)
  try(name .. " write value nan", wr, b, 0, nan)
  try(name .. " write value inf", wr, b, 0, inf)
  try(name .. " write value 2^64", wr, b, 0, 2 ^ 64)
end
refused("readstring count 2^31", buffer.readstring, b, 0, 2 ^ 31)
refused("readstring count 2^32+1", buffer.readstring, b, 0, 2 ^ 32 + 1)
refused("readstring off len count 1", buffer.readstring, b, n, 1)
refused("readstring count -1", buffer.readstring, b, 0, -1)
refused("readstring off 2^32-1 count 2", buffer.readstring, b, 2 ^ 32 - 1, 2)
assert(buffer.readstring(b, n, 0) == "")
refused("writestring count > #s", buffer.writestring, b, 0, "abc", 4)
refused("writestring count 2^32", buffer.writestring, b, 0, "abc", 2 ^ 32)
refused("writestring off 2^32-1", buffer.writestring, b, 2 ^ 32 - 1, "ab")
refused("writestring past end", buffer.writestring, b, n - 1, "ab")
try("writestring count -1", buffer.writestring, b, 0, "abc", -1)
try("writestring count nan", buffer.writestring, b, 0, "abc", nan)
return {}
