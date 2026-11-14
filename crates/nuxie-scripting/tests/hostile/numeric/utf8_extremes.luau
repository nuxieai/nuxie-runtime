--! attacks: utf8.char/codepoint/len/offset/codes with codepoints past 0x10FFFF, negative, NaN, 2^31 and positions at the extremes, and invalid byte sequences; expect: ok; within: 20s
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
for _, cp in ipairs({ 0x110000, 0x7FFFFFFF, 0x80000000, 2 ^ 32, 2 ^ 53, -1, -(2 ^ 31), inf }) do
  refused("char " .. cp, utf8.char, cp)
end
try("char nan", utf8.char, nan)
try("char 0x10FFFF", utf8.char, 0x10FFFF)
try("char surrogate", utf8.char, 0xD800)
try("char 7000 args", utf8.char, table.unpack(table.create(7000, 0x10FFFF)))
local s = "h\u{E9}llo \u{10FFFF}"
refused("codepoint 1..2^31", utf8.codepoint, s, 1, 2 ^ 31)
refused("codepoint -2^31", utf8.codepoint, s, -(2 ^ 31))
try("codepoint nan", utf8.codepoint, s, nan)
refused("codepoint whole 100k string", utf8.codepoint, string.rep("a", 100000), 1, -1)
try("len -2^31", utf8.len, s, -(2 ^ 31))
try("len 2^31", utf8.len, s, 2 ^ 31)
try("len j 2^53", utf8.len, s, 1, 2 ^ 53)
try("offset 2^31", utf8.offset, s, 2 ^ 31)
try("offset -2^31", utf8.offset, s, -(2 ^ 31))
try("offset n i 2^31", utf8.offset, s, 1, 2 ^ 31)
try("offset nan", utf8.offset, s, nan)
for _, bad in ipairs({ "\255", "\192\128", "\237\160\128", "\244\144\128\128", "\248\136\128\128\128", string.rep("\128", 1000) }) do
  try("len bad", utf8.len, bad)
  try("codepoint bad", utf8.codepoint, bad)
  try("codes bad", function() for _, c in utf8.codes(bad) do end end)
end
local n = 0
for _, c in utf8.codes(string.rep("\u{10FFFF}", 10000)) do n = n + 1 end
assert(n == 10000)
return {}
