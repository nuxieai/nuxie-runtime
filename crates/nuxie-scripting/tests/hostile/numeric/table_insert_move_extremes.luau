--! attacks: table.insert/remove at extreme positions and table.move with ranges and destinations overflowing 32 and 53 bits, the saturating 2^31-slot move last; expect: error(not enough memory); within: 20s
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
local t = { 1, 2, 3 }
for _, pos in ipairs({ 0, 5, -1, 2 ^ 31, -(2 ^ 31), 2 ^ 32, 2 ^ 53, inf, -inf }) do
  try("insert pos " .. pos, table.insert, t, pos, "v")
  assert(#t < 100)
end
try("insert pos nan", table.insert, t, nan, "v")
try("insert pos -0", table.insert, t, -0, "v")
try("insert pos 1.5", table.insert, t, 1.5, "v")
refused("insert 4 args", table.insert, t, 1, 2, 3)
for _, pos in ipairs({ 2 ^ 31, -(2 ^ 31), 2 ^ 53, inf, nan, -1 }) do
  try("remove pos " .. tostring(pos), table.remove, t, pos)
end
-- a sparse table whose border is near 2^31
local sparse = {}
sparse[2 ^ 31 - 1] = 1
try("insert into sparse", table.insert, sparse, "v")
try("remove from sparse", table.remove, sparse)
local a = { 1, 2, 3 }
refused("move 1..2^31", table.move, a, 1, 2 ^ 31, 2)
refused("move -2^31..2^31", table.move, a, -(2 ^ 31), 2 ^ 31, 1)
refused("move to wrap", table.move, a, 1, 3, 2 ^ 31 - 1)
refused("move to wrap 53", table.move, a, 1, 3, 2 ^ 53)
refused("move from -inf", table.move, a, -inf, 1, 1)
try("move empty range huge dest", table.move, a, 2, 1, 2 ^ 53)
try("move nan", table.move, a, nan, 3, 1)
try("move to nan", table.move, a, 1, 3, nan)
try("move to -2^31", table.move, a, 1, 3, -(2 ^ 31))
local okm, r = try("move into other", table.move, a, 1, 3, 2 ^ 31 - 3, {})
-- move into a table with __newindex (the VM moves raw: bounded work either way)
local count = 0
local sink = setmetatable({}, { __newindex = function() count = count + 1 end })
table.move(table.create(20000, 1), 1, 20000, 1, sink)
assert(count == 0 or count == 20000)
-- 1..2^53 saturates to 1..2^31-1 and asks for a 2^31-slot destination array: terminal memory error by design
table.move(a, 1, 2 ^ 53, 1)
error("2^31-element move completed")
