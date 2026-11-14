--! attacks: string.rep with huge, negative, NaN and fractional counts and huge separators (ordinary refusals, memory-scale requests last); expect: error(not enough memory); within: 20s
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
refused("rep 2^31", string.rep, "x", 2 ^ 31)
refused("rep 2^53", string.rep, "x", 2 ^ 53)
refused("rep ab 2^53", string.rep, "ab", 2 ^ 53)
refused("rep huge", string.rep, "x", inf)
assert(string.rep("x", -1) == "")
assert(string.rep("x", -inf) == "")
try("rep nan", string.rep, "x", nan)
try("rep 1.5", string.rep, "x", 1.5)
refused("rep sep 2^31", string.rep, "x", 2 ^ 31, ",")
assert(#string.rep("x", 2 ^ 20, string.rep(",", 2 ^ 12)) == 2 ^ 20) -- Luau string.rep takes no separator
try("rep 0 with sep", string.rep, string.rep("y", 1000), 0, "sep")
-- a single 128 MiB string: over the 64 MiB VM heap, terminal by design
local s = string.rep("x", 2 ^ 27)
error("128 MiB string was allocated: " .. #s)
