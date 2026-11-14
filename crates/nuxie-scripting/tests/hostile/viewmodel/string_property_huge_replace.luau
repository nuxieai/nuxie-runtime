--! attacks: string property set to huge strings, NUL-bearing and invalid UTF-8 strings, repeatedly replaced on one instance; expect: ok; within: 20s; viewmodels: script_create_viewmodel_instance.riv
local inst = Data.SubChild.new()
local p = inst:getString("str")
local big = string.rep("q", 8 * 1024 * 1024)
for i = 1, 6 do
  p.value = big
end
assert(#p.value == #big)
p.value = "a\0" .. big
assert(p.value == "a")
p.value = "\255\254\253" .. string.rep("\192", 1024)
p.value = ""
assert(p.value == "")
-- table.pack keeps the nil that ipairs would stop at.
local args1 = table.pack(nil, {}, print, true)
for i1 = 1, args1.n do local v = args1[i1]
  pcall(function() p.value = v end)
end
p.value = 1e308
return {}
