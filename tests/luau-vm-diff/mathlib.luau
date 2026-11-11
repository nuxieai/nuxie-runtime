-- math library: lane 5 additions (frexp, ldexp, random, noise, map, lerp,
-- predicates, constants). Compiled calls run the fast body under safeenv;
-- pcall reaches the library body.
local function all(...)
  local t = table.pack(...)
  for i = 1, t.n do t[i] = tostring(t[i]) end
  return "(" .. table.concat(t, ",") .. ")"
end
print(math.nan ~= math.nan, math.e, math.phi, math.sqrt2, math.tau)
print(all(math.frexp(math.pi)), all(math.frexp(-0.0)), all(math.frexp(0)), all(math.frexp(5e-324)), all(math.frexp(math.huge)), all(math.frexp(-1e300)))
print(all(pcall(math.frexp, 2^-1060)), all(pcall(math.frexp, "8")))
-- ldexp: the fast body is exact C ldexp; the library multiplies by powi.
print(math.ldexp(1, 10), math.ldexp(0.5, 3), math.ldexp(2^-1000, 1050), math.ldexp(1, -1074), math.ldexp(3, 2000), math.ldexp(1, -2000))
print(all(pcall(math.ldexp, 2^-1000, 1050)), all(pcall(math.ldexp, 1, -1074)), all(pcall(math.ldexp, 1.5, 7.9)), all(pcall(math.ldexp, 1, "3")))
local v, e = math.frexp(123.456)
print(math.ldexp(v, e), v, e)
print(math.fround(0.1), math.fround(1e40), math.fround(-0.0), all(pcall(math.fround)))
print(math.map(5, 0, 10, 0, 100), math.map(1, 1, 1, 0, 1) ~= math.map(1, 1, 1, 0, 1), math.map(-1, 0, 1, 10, 20))
print(math.lerp(1, 3, 0.5), math.lerp(1, 3, 1), math.lerp(-1e308, 1e308, 0.25), all(pcall(math.lerp, 1, 2)))
print(math.isnan(0/0), math.isnan(1), math.isinf(-math.huge), math.isinf(1e308), math.isfinite(1), math.isfinite(0/0), all(pcall(math.isnan, "x")))
print(math.noise(0.5), math.noise(1.5, 2.25), math.noise(-3.75, 4.5, 10.125), math.noise(300.3, -700.7, 1e9), math.noise("1.5"))
print(all(pcall(math.noise)), all(pcall(math.noise, 1, "x")), all(pcall(math.noise, 1, nil, {})))
math.randomseed(42)
local r = {}
for i = 1, 5 do r[#r + 1] = math.random() end
for i = 1, 5 do r[#r + 1] = math.random(10) end
for i = 1, 5 do r[#r + 1] = math.random(-5, 5) end
r[#r + 1] = math.random(-2^31, 2^31 - 2)
print(table.concat(r, " "))
math.randomseed(-1)
print(math.random(), math.random(1, 2^31 - 1), math.random(2^40))
print(all(pcall(math.random, 0)), all(pcall(math.random, 3, 2)), all(pcall(math.random, -2^31, 2^31 - 1)), all(pcall(math.random, 1, 2, 3)), all(pcall(math.random, "x")))
print(all(pcall(math.randomseed)), math.randomseed(7))
return math.random(100)
