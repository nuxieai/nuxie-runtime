--! attacks: font property set to hostile values (decoded garbage, wrong types) and stale font handles; expect: ok; within: 10s; viewmodels: data_bind_font_test.riv
local inst = Data.ViewModel1.new()
local f = inst:getFont("fontSource")
for _, v in ipairs({1, "x", {}, true, print, Audio.Ring.create(8), Path.new()}) do
  pcall(function() f.value = v end)
end
pcall(function() f.value = nil end)
local okd, decoded = pcall(Font.decode, buffer.fromstring(string.rep("\0", 4096)))
if okd and decoded ~= nil then pcall(function() f.value = decoded end) end
local okt = pcall(function() return Font.decode(buffer.fromstring("OTTO" .. string.rep("\255", 65536))) end)
local got = f.value
local count = 0
f:addListener(function() count += 1 f.value = f.value end)
pcall(function() f.value = got end)
assert(count < 1000, "font self-assign ran " .. count)
return {}
