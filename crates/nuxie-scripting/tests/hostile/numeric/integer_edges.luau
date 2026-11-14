--! attacks: integer-like edges (2^53+1, 2^63, -0, NaN keys) across table keys, array borders, string.format, tostring/tonumber, // and % by zero; expect: ok; within: 20s
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
local t = {}
t[2 ^ 53] = "a"
t[2 ^ 53 + 1] = "b"
assert(t[2 ^ 53] == "b") -- 2^53+1 rounds to 2^53
t[-0] = "z"
assert(t[0] == "z")
refused("nan key", function() local k = {} k[nan] = 1 end)
assert(({})[nan] == nil)
refused("rawset nan", rawset, {}, nan, 1)
t = {}
t[2 ^ 31] = 1 t[2 ^ 32] = 1 t[-(2 ^ 31)] = 1 t[2 ^ 63] = 1 t[inf] = 1 t[-inf] = 1
try("len of sparse", function() return #t end)
local arr = table.create(100, 1)
arr[2 ^ 31] = 1
try("ipairs sparse", function() local n = 0 for _ in ipairs(arr) do n = n + 1 end return n end)
try("next after huge key", next, t, 2 ^ 63)
refused("next bad key", next, t, 12345)
assert(tostring(-0) == "-0" or tostring(-0) == "0")
assert(1 / -0 == -inf)
assert(tonumber("0x" .. string.rep("f", 100)) ~= nil or true)
try("tonumber huge hex", tonumber, "0x" .. string.rep("f", 100))
try("tonumber huge exp", tonumber, "1e999999999999")
try("tonumber base 36 long", tonumber, string.rep("z", 1000), 36)
refused("tonumber base 37", tonumber, "1", 37)
refused("tonumber base 1", tonumber, "1", 1)
try("tonumber base nan", tonumber, "1", nan)
assert(tostring(2 ^ 53 + 1) == tostring(2 ^ 53))
try("fmt 2^63", string.format, "%d", 2 ^ 63)
try("fmt -2^63", string.format, "%d", -(2 ^ 63))
try("fmt 2^53+1", string.format, "%.0f", 2 ^ 53 + 1)
assert(1 // 0 == inf)
assert(-1 // 0 == -inf)
assert(0 // 0 ~= 0 // 0)
assert(1 % 0 ~= 1 % 0)
try("5 % inf", function() return 5 % inf end)
try("-5 % inf", function() return -5 % inf end)
assert((-0) % 1 == 0)
try("int-like loop bounds", function() local n = 0 for i = 2 ^ 53, 2 ^ 53 + 2 do n = n + 1 if n > 10 then error("did not terminate") end end return n end)
try("loop step -0", function() for i = 1, 2, -0 do end end)
return {}
