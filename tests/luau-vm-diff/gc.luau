-- allocation pressure: the collector runs at safepoints while values stay live
local keep = {}
for i = 1, 20000 do
    local t = {i, tostring(i), {i * 2}}
    if i % 100 == 0 then keep[#keep + 1] = t end
end
local sum = 0
for _, t in ipairs(keep) do sum = sum + t[1] + t[3][1] end
print(#keep, sum, keep[1][2], keep[200][2])
local strings = {}
for i = 1, 5000 do strings[i % 50 + 1] = string.rep("s", i % 37) .. i end
print(#strings, strings[1], strings[50])
local function make(n)
    local list = nil
    for i = 1, n do list = {value = i, next = list} end
    return list
end
local l = make(10000)
local c = 0
while l do c = c + l.value; l = l.next end
print(c)
local closures = {}
for i = 1, 3000 do closures[i % 10 + 1] = function() return i end end
print(closures[1](), closures[10]())
return sum
