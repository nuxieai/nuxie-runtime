-- vector library and the vector metatable. Compiled calls run the fast
-- bodies under safeenv (magnitude and dot fuse their multiply-adds);
-- pcall reaches the library bodies. (vector.angle with one argument is not
-- probed: the oracle dereferences a null vector there.)
local function all(...)
  local t = table.pack(...)
  for i = 1, t.n do t[i] = tostring(t[i]) end
  return "(" .. table.concat(t, ",") .. ")"
end
local V = vector.create
local a, b = V(1, 2, 3), V(-4.5, 0.25, 8)
print(a, vector.zero, vector.one, V(1, 2), V(1e40, -1e-50, 0.1), typeof(a), type(vector.create(0, 0, 0)))
print(a.x, a.Y, a.z, b.X, all(pcall(function() return a.w end)), all(pcall(function() return a.length end)), all(pcall(function() return a[1] end)))
print(vector.magnitude(a), vector.normalize(b), vector.cross(a, b), vector.dot(a, b), vector.angle(a, b), vector.angle(a, b, V(0, 0, -1)))
print(vector.floor(b), vector.ceil(b), vector.abs(b), vector.sign(b), vector.sign(V(0, -0, 0/0)))
print(vector.clamp(V(5, -5, 0.5), V(0, 0, 0), V(1, 1, 1)), vector.max(a, b, V(9, -9, 0)), vector.min(a, b), vector.max(a), vector.min(V(0/0, 1, 1), a))
print(vector.lerp(a, b, 0.5), vector.lerp(a, b, 1), vector.lerp(a, b, 0), vector.normalize(V(0, 0, 0)))
print(all(pcall(vector.clamp, a, V(1, 1, 1), V(0, 2, 2))), all(pcall(vector.clamp, a, V(0, 3, 0), V(1, 2, 2))), all(pcall(vector.clamp, a, V(0, 0, 0/0), V(1, 1, 1))))
print(all(pcall(vector.create, 1)), all(pcall(vector.create, "1", 2, "3")), all(pcall(vector.magnitude, 1)), all(pcall(vector.dot, a)), all(pcall(vector.lerp, a, b)))
print(all(pcall(vector.max, a, 1)), all(pcall(vector.min)), getmetatable(a) ~= nil, all(pcall(setmetatable, a, {})))
-- fast (fused) and library (unfused) bodies over a sweep
local fastm, libm, fastd, libd, diffm, diffd = 0, 0, 0, 0, 0, 0
for i = 1, 200 do
  local v = V(i * 0.1, i * 0.37 - 20, 1 / i)
  local w = V(i * -0.013, 3.3, i * 0.5)
  local m1, m2 = vector.magnitude(v), select(2, pcall(vector.magnitude, v))
  local d1, d2 = vector.dot(v, w), select(2, pcall(vector.dot, v, w))
  fastm += m1; libm += m2; fastd += d1; libd += d2
  if m1 ~= m2 then diffm += 1 end
  if d1 ~= d2 then diffd += 1 end
end
print(fastm, libm, fastd, libd, diffm, diffd)
local t = {}
t[a] = "a"
print(t[V(1, 2, 3)], t[b], a == V(1, 2, 3), a ~= b, rawequal(a, V(1, 2, 3)))
return vector.magnitude(V(3, 4, 0))
