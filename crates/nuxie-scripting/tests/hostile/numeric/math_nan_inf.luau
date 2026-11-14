--! attacks: math library with NaN/inf/-0/2^53 arguments (floor, ceil, fmod, log, ldexp, frexp, clamp, sign, round, noise, min/max) and math.random with huge, reversed, NaN ranges; expect: ok; within: 20s
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
local vals = { nan, inf, -inf, -0, 2 ^ 53 + 1, -(2 ^ 63), 2 ^ 1023 * 1.9, 5e-324 }
for _, v in ipairs(vals) do
  for _, f in ipairs({ "floor", "ceil", "abs", "sqrt", "log", "log10", "exp", "sin", "cos", "tan", "asin", "atan", "sinh", "round", "sign", "frexp", "modf", "deg", "rad" }) do
    try(f .. " " .. tostring(v), math[f], v)
  end
  for _, w in ipairs(vals) do
    try("fmod", math.fmod, v, w)
    try("pow", math.pow or function(a, b) return a ^ b end, v, w)
    try("atan2", math.atan2, v, w)
    try("min", math.min, v, w)
    try("max", math.max, v, w)
    try("log base", math.log, v, w)
  end
  try("ldexp", math.ldexp, 1, v)
  try("ldexp v", math.ldexp, v, 2 ^ 31)
  try("noise", math.noise, v, v, v)
  try("clamp v 0 1", math.clamp, v, 0, 1)
  try("clamp 0 v 1", math.clamp, 0, v, 1)
  try("clamp 0 0 v", math.clamp, 0, 0, v)
  try("lerp", math.lerp or math.max, v, v, v)
  try("map", math.map or math.max, v, 0, 1, 0, 1)
end
refused("clamp max<min", math.clamp, 0, 1, 0)
try("fmod 1 0", math.fmod, 1, 0)
local function inrange(name, lo, hi, ...)
  local ok, v = try(name, math.random, ...)
  if ok then assert(v >= lo and v <= hi and v == math.floor(v), name .. " = " .. tostring(v)) end
end
inrange("random 2^31", 1, 2 ^ 31, 2 ^ 31)
inrange("random 0 2^31", 0, 2 ^ 31, 0, 2 ^ 31)
inrange("random -2^31 2^31", -(2 ^ 31), 2 ^ 31, -(2 ^ 31), 2 ^ 31)
inrange("random 2^53", 1, 2 ^ 53, 2 ^ 53)
inrange("random -2^53 2^53", -(2 ^ 53), 2 ^ 53, -(2 ^ 53), 2 ^ 53)
refused("random 0", math.random, 0)
refused("random 5 1", math.random, 5, 1)
try("random nan", math.random, nan)
try("random 1 nan", math.random, 1, nan)
try("random inf", math.random, inf)
try("random -inf inf", math.random, -inf, inf)
try("randomseed nan", math.randomseed, nan)
try("randomseed inf", math.randomseed, inf)
local x = math.random()
assert(x >= 0 and x < 1)
return {}
