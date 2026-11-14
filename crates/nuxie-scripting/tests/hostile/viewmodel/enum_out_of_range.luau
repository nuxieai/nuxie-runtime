--! attacks: enum property set to unknown names, indices, huge strings, and values() indexed out of range; expect: ok; within: 10s; viewmodels: scripted_enum.riv
local inst = Data.EnumVM.new()
local e = inst:getEnum("EnumProp")
local values = e:values()
local n = #values
assert(n == 4, "enum values " .. n)
for _, i in ipairs({0, -1, n + 1, 2 ^ 31, 2 ^ 53, 0 / 0, math.huge}) do
  pcall(function() return values[i] end)
end
for _, v in ipairs({"purple", "", "RED", 0, -1, 5, 2 ^ 32, 0 / 0, string.rep("r", 4 * 1024 * 1024), "red\0blue"}) do
  pcall(function() e.value = v end)
  assert(type(e.value) == "string")
end
for _, v in ipairs({{}, print, true}) do
  assert(not pcall(function() e.value = v end))
end
e.value = "red"
return {}
