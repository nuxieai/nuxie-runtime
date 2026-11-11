-- bit32: every function, the edge counts and the argument errors.
local function all(...)
  local t = table.pack(...)
  for i = 1, t.n do t[i] = tostring(t[i]) end
  return "(" .. table.concat(t, ",") .. ")"
end
local b = bit32
print(b.band(), b.band(0xff00, 0x0ff0, 0x00ff), b.bor(), b.bor(1, 2, 4), b.bxor(), b.bxor(5, 3, 1), b.bnot(0), b.bnot(-1))
print(b.band(-1), b.band(2^32), b.band(2^32 + 5), b.band(1.9), b.band(-1.5), b.band(0/0), b.band(1e300), b.band("12"))
print(b.btest(1, 2), b.btest(3, 2), b.btest(), b.btest(-1, 0x80000000))
for _, s in ipairs({-40, -33, -32, -31, -1, 0, 1, 31, 32, 33, 40, -2^31}) do
  print(s, b.lshift(0x80000001, s), b.rshift(0x80000001, s), b.arshift(0x80000001, s), b.arshift(0x7fffffff, s),
    b.lrotate(0x80000001, s), b.rrotate(0x80000001, s))
end
print(b.extract(0xf0f0f0f0, 4), b.extract(0xf0f0f0f0, 4, 4), b.extract(0xffffffff, 0, 32), b.extract(-1, 31))
print(b.replace(0, 1, 31), b.replace(0xffffffff, 0, 8, 8), b.replace(0, 0xff, 0, 32), b.replace(5, 2, 1, 2))
print(b.countlz(0), b.countlz(1), b.countlz(-1), b.countrz(0), b.countrz(0x80000000), b.countrz(12))
print(b.byteswap(0x12345678), b.byteswap(-1), b.byteswap(1))
-- library bodies through pcall (the compiled calls above run fast bodies)
print(all(pcall(b.lshift, 1, 31)), all(pcall(b.extract, 0xabcd, 8, 4)), all(pcall(b.replace, 0, 7, 29, 3)), all(pcall(b.countlz, 255)))
print(all(pcall(b.extract, 1, -1)), all(pcall(b.extract, 1, 0, 0)), all(pcall(b.extract, 1, 30, 3)), all(pcall(b.replace, 1, 1, 32)))
print(all(pcall(b.band, 1, "x")), all(pcall(b.bnot)), all(pcall(b.lshift, 1)), all(pcall(b.lrotate, {}, 1)), all(pcall(b.extract, 1, 2^40)))
local t = {}
for i = 1, 20 do t[i] = b.bxor(b.lrotate(i * 2654435761, i), b.rshift(i * 40503, 3)) end
print(table.concat(t, " "))
return b.band(0xdeadbeef, 0xffff)
