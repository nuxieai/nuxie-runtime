-- Vector arithmetic through the register arms' tail (docs/evidence/
-- luau-safe-vm-phase2-lane10-2026-11-12.md, 9.2): two vectors, a vector
-- and a number either way round, every register and constant form, and
-- what must still reach the general step (string coercion, `%` and `^`,
-- a vector and a number under `+`/`-`, the vector metatable, errors).
local function all(...)
  local t = table.pack(...)
  for i = 1, t.n do t[i] = tostring(t[i]) end
  return "(" .. table.concat(t, ",") .. ")"
end
local V = vector.create
local a, b = V(1, -2, 3.5), V(-4.5, 0.25, 0)
local n, z = 2, 0

-- Register forms.
print(a + b, a - b, a * b, a / b, a // b, -a, -b)
print(a * n, n * a, a / n, n / a, a // n, n // a)
print(a / z, a // z, z / a, V(0, -0, 1) * -1)
-- Constant forms (MULK, DIVK, IDIVK, SUBRK, DIVRK and the rest).
print(a * 2, 2 * a, a / 4, 4 / a, a // 3, 3 // a)
print(a * -0.5, a / 0, -0.5 * b)
-- NaN and infinities, component by component.
local nanv = V(0/0, 1/0, -1/0)
print(nanv + a, nanv * 0, a / V(0, 0, 0), -nanv)
-- Float32 rounding: the number is narrowed before the operation.
print(V(1, 1, 1) * 0.1, V(3, 3, 3) / 3.3, V(1e30, 1, 1) * 1e10)

-- The general step: coercion, unsupported operators, errors.
print(all(pcall(function() return a * "2" end)))
print(all(pcall(function() return "2" * a end)))
print(all(pcall(function() return a / "x" end)))
print(all(pcall(function() return a + 1 end)))
print(all(pcall(function() return 1 - a end)))
print(all(pcall(function() return a % b end)))
print(all(pcall(function() return a ^ 2 end)))
print(all(pcall(function() return a % 2 end)))
print(all(pcall(function() return a + nil end)))
print(all(pcall(function() return {} * a end)))
-- A table with __mul on either side of a vector.
local m = setmetatable({}, { __mul = function(x, y) return "mul:" .. typeof(x) .. ":" .. typeof(y) end,
                             __add = function(x, y) return "add" end })
print(m * a, a * m, a + m, m + a)

-- In a loop, so the arms run hot.
local acc = V(0, 0, 0)
for i = 1, 100 do
  acc += a * i
  acc -= b / i
  acc = acc * 0.5 + -acc // 7
end
print(acc)
