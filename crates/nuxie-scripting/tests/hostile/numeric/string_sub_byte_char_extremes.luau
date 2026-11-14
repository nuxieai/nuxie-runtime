--! attacks: string.sub/byte/char/len/reverse/upper with indices at +-2^31, +-2^53, NaN, inf and huge byte ranges; expect: ok; within: 20s
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
local s = "hello world"
for _, i in ipairs({ 2 ^ 31, -(2 ^ 31), 2 ^ 32, 2 ^ 53, -(2 ^ 53), 2 ^ 63, inf, -inf, nan, -0, 0.5, -0.5 }) do
  for _, j in ipairs({ 2 ^ 31, -(2 ^ 31), 2 ^ 53, -1, inf, nan }) do
    local ok, r = try("sub " .. i .. "," .. j, string.sub, s, i, j)
    if ok then assert(type(r) == "string" and #r <= #s) end
  end
  try("byte " .. i, string.byte, s, i)
end
try("byte 1..2^31", string.byte, s, 1, 2 ^ 31)
try("byte range of a 100k string", string.byte, string.rep("a", 100000), 1, -1)
try("byte -2^31..2^31", string.byte, s, -(2 ^ 31), 2 ^ 31)
refused("char 256", string.char, 256)
refused("char -1", string.char, -1)
refused("char 2^32", string.char, 2 ^ 32)
refused("char inf", string.char, inf)
try("char nan", string.char, nan)
try("char 7000 args", string.char, table.unpack(table.create(7000, 65)))
local big = string.rep("ab", 1000000)
assert(#string.reverse(big) == #big)
assert(#string.upper(big) == #big)
assert(#string.sub(big, -(2 ^ 53), 2 ^ 53) == #big)
assert(string.sub(big, 2 ^ 53, -(2 ^ 53)) == "")
-- 1 Mi results from one string.byte: the value stack must refuse it
local okb, eb = pcall(function() return select("#", string.byte(string.rep("a", 2 ^ 20), 1, -1)) end)
assert(not okb and string.find(tostring(eb), "stack overflow", 1, true), tostring(eb))
return {}
