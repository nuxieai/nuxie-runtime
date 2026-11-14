--! attacks: NaN, infinities, huge, subnormal, numeric-string and non-number values set on number and boolean properties; expect: ok; within: 10s; viewmodels: script_create_viewmodel_instance.riv
local inst = Data.SubChild.new()
local p = inst:getNumber("num")
local seen = 0
p:addListener(function() seen += 1 end)
for _, v in ipairs({0 / 0, math.huge, -math.huge, 1e308, -1e308, 2 ^ 128, 5e-324, -0, "1e400", "nan", "0x7fffffff", "  12  "}) do
  pcall(function() p.value = v end)
  local r = p.value
  assert(type(r) == "number")
end
for _, v in ipairs({{}, print, "abc", coroutine.create(function() end)}) do
  assert(not pcall(function() p.value = v end), "accepted " .. tostring(v))
end
local b = inst:getBoolean("bol")
for _, v in ipairs({0 / 0, "true", {}, nil}) do pcall(function() b.value = v end) end
-- the same NaN twice: NaN ~= NaN must not make every set notify forever
local before = seen
p.value = 0 / 0
p.value = 0 / 0
assert(seen - before <= 2)
return {}
