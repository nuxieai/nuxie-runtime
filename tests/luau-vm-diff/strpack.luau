-- string.pack / packsize / unpack: sizes, alignment, endianness, limits.
local function all(...)
  local t = table.pack(...)
  for i = 1, t.n do
    local v = t[i]
    t[i] = type(v) == "string" and string.format("%q", v) or tostring(v)
  end
  return "(" .. table.concat(t, ",") .. ")"
end
local function p(...) return all(pcall(string.pack, ...)) end
local function u(...) return all(pcall(string.unpack, ...)) end
local function s(...) return all(pcall(string.packsize, ...)) end
print(p("i4", 100), p(">i4", -2), p("<h>H=b", 1, 2, 3), p("i16", -1), p("I16", 7), p("j J T", -5, 5, 6), p("l L", -1, 2^53))
print(p("f d n", 1.5, -0.1, 1/0), p(">f", 0/0 ~= 0/0 and 2.25 or 0), p("c5", "ab"), p("c0", ""), p("s1", "hey"), p("s", "x"), p("z", "zed"), p("xXi4", 1))
print(p("!4 b i4", 1, 2), p("!8 b d", 1, 2), p("! b l", 1, 2), p("!2 b i8", 1, 2), p("b Xi4 b", 1, 2), p("i3 i5 i7", 1, -1, 2^40), p("I3", 2^24 - 1))
print(p("i1", 128), p("i1", -129), p("I1", 256), p("I1", -1), p("i2", 2^15), p("c2", "abc"), p("s1", string.rep("a", 256)), p("z", "a\0b"))
print(p("i17"), p("i0"), p("q"), p("c"), p("c1073741825", ""), p("X"), p("Xc1"), p("X<"), p("!3 i4", 1), p("!4 i3", 1), p("i4"), p("i4", "x"), p("\xFF"))
print(p("i4\0i4", 1, 2), p("c99999999999", ""), p("s4", 12345), p("i", "10"), p("b", 1.9), p("b", -1.9), p("l", 2^63), p("l", -2^63), p("L", 2^64))
print(s("i4 i8"), s("!8 b d"), s("c10 x"), s("s"), s("z"), s("c1073741824 c1"), s("c1073741824"), s(""), s("Xi4"), s("bXi16"), s(1))
local packed = string.pack("<i4 >I2 f d s1 z c3 !4 i8", -7, 513, 0.5, 1e300, "ab", "zz", "xyz", 2^40)
print(u("<i4 >I2 f d s1 z c3 !4 i8", packed), u("i4", packed, 5), u("i4", packed, -4), u("i4", packed, #packed + 1), u("i4", packed, #packed + 2), u("i4", packed, -100))
print(u("i16", string.rep("\255", 16)), u("i16", string.rep("\0", 8) .. "\1" .. string.rep("\0", 7)), u("I9", string.rep("\255", 9)), u("i9", "\0\0\0\0\0\0\0\128\255"), u(">i3", "\255\255\254"), u("I3", "\1\2\3"))
print(u("s1", "\5ab"), u("z", "abc"), u("z", "abc\0"), u("c4", "ab"), u("x", ""), u("Xi4 b", "\1"), u("!4 b Xi4 i4", "\1\0\0\0\2\0\0\0"), u("i4", "\1\2\3"), u("<f", "\0\0\128\127"), u(">d", "\127\240\0\0\0\0\0\0"))
print(u("i4", 12345678), u(1, "1"), u("b", "\1", 0), u("b", "\1", "x"), u("", "abc", 2), u("  ", "a"))
print(all(pcall(string.unpack, string.rep("b", 7999), string.rep("\1", 7999))):sub(1, 60), all(pcall(string.unpack, string.rep("b", 8000), string.rep("\1", 8000))):sub(-60))
