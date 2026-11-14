--! attacks: string.format with huge width/precision, malformed specifiers, %d/%x of NaN/inf/2^63, and %s/%q of multi-MiB strings; expect: ok; within: 20s
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
refused("width 99999", string.format, "%99999d", 1)
refused("width 100", string.format, "%100d", 1)
refused("precision 100", string.format, "%.100f", 1)
refused("flags overflow", string.format, "%-----------------------------5d", 1)
try("star", string.format, "%*d", 5, 1)
refused("trailing percent", string.format, "abc%")
refused("unknown conv", string.format, "%y", 1)
refused("missing arg", string.format, "%d %d", 1)
assert(#string.format("%99d", 1) == 99)
assert(#string.format("%.99f", 1e308) > 300)
assert(#string.format("%99.99f", -1e308) > 300)
for _, v in ipairs({ nan, inf, -inf, 2 ^ 63, -(2 ^ 63), 2 ^ 64, 2 ^ 53 + 1, -0 }) do
  try("d " .. tostring(v), string.format, "%d", v)
  try("x " .. tostring(v), string.format, "%x", v)
  try("i " .. tostring(v), string.format, "%5i", v)
  try("c " .. tostring(v), string.format, "%c", v)
  try("g " .. tostring(v), string.format, "%.99g", v)
end
local s = string.rep("q\0\"\\\n", 400000)
assert(#string.format("%s", s) == #s)
assert(#string.format("%-99s|", s) == #s + 1)
local q = string.format("%q", s)
assert(#q > #s)
try("q of number", string.format, "%q", 1 / 3)
try("q of table", string.format, "%q", {})
-- many conversions in one format
local fmt = string.rep("%s", 7000)
try("7000 conversions", string.format, fmt, table.unpack(table.create(7000, "ab")))
return {}
