-- FASTCALL FASTCALL1 FASTCALL2 FASTCALL2K FASTCALL3
local x, y, z = -3.5, 2, 9
print(math.abs(x), math.floor(x), math.ceil(x), math.sqrt(z))
print(math.max(x, y), math.min(x, y), math.max(x, 1), math.fmod(z, 4))
print(math.clamp(x, 0, 1), math.clamp(z, 0, 5))
print(math.max(1, 2, 3, 4, 5), math.min(5, 4, 3, 2, 1, 0))
print(type(x), typeof(y), type("s"), type(nil), type({}), type(print))
print(rawlen({1, 2}), rawequal(x, x), rawget({a = 1}, "a"))
local t = {}
rawset(t, "k", "v")
print(t.k, select("#", 1, 2, 3), tostring(12), tonumber("0x1F"))
print(string.byte("A"), string.char(72, 105), string.len("four"), string.sub("hello", 2, 3))
print(math.sign(-2), math.round(2.5), math.round(-2.5))
local function f(...) return math.max(...) end
print(f(4, 9, 2))
local v = {1, 2, 3}
print(unpack and "unpack" or "no-unpack", table.unpack(v))
print(getmetatable("").__index == string, setmetatable({}, nil) ~= nil)
local ok, err = pcall(math.floor, "x")
print(ok, err)
ok, err = pcall(function() return math.max() end)
print(ok, err)
return math.abs(-1)
