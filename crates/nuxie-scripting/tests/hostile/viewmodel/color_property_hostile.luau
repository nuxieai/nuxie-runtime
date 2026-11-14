--! attacks: color properties set to NaN, negative, > 32-bit and non-number values; expect: ok; within: 10s; viewmodels: global_variables_test.riv
local inst = Data.Colors.new()
local c = inst:getColor("textColor")
for _, v in ipairs({0 / 0, -1, 2 ^ 32, 2 ^ 53, math.huge, -math.huge, 0.5, "0xffffffff", {}, true}) do
  pcall(function() c.value = v end)
  assert(type(c.value) == "number")
end
return {}
