--! attacks: DataValue.string with NUL-containing, 8 MiB and wrong-typed values, reassigned repeatedly; expect: ok; within: 20s
local function safe(f, ...)
  local ok, err = pcall(f, ...)
  assert(ok or not string.find(tostring(err), "rust panic", 1, true), tostring(err))
  return ok, err
end
local d = DataValue.string()
d.value = "abc\0def"
safe(function() return d.value end)
d.value = string.rep("\0", 1000)
local big = string.rep("s", 8 * 1024 * 1024)
local ok, err = pcall(function() d.value = big end)
assert(ok or string.find(tostring(err), "host memory budget", 1, true), tostring(err))
for _ = 1, 3 do
  local ok2, err2 = pcall(function() d.value = big end)
  assert(ok2 or string.find(tostring(err2), "host memory budget", 1, true), tostring(err2))
  d.value = ""
end
for _, v in ipairs({5, true, Path.new(), {}, 0 / 0}) do
  safe(function() d.value = v end)
end
safe(function() d.value = nil end)
safe(function() return DataValue.string(big) end)
safe(function() return DataValue.number().value, DataValue.number(0 / 0) end)
local n = DataValue.number()
safe(function() n.value = "x" end)
safe(function() n.value = big end)
local c = DataValue.color()
for _, v in ipairs({-1, 2 ^ 53, 0 / 0, math.huge}) do safe(function() c.value = v end) end
safe(function() c.red = 1e300 c.alpha = -1 end)
return {}
