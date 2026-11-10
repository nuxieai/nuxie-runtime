-- NEWTABLE DUPTABLE SETLIST GETTABLE SETTABLE GETTABLEKS SETTABLEKS GETTABLEN SETTABLEN LENGTH
local t = {}
t.a = 1
t["b"] = 2
local key = "c"
t[key] = 3
t[1] = "one"
t[2] = "two"
local idx = 3
t[idx] = "three"
print(t.a, t.b, t[key], t[1], t[2], t[idx], #t)
local d = {x = 1, y = 2, z = 3}
d.y = 20
print(d.x, d.y, d.z)
local function three() return 7, 8, 9 end
local list = {1, 2, 3, three()}
print(#list, list[4], list[6])
local list2 = {three(), 10}
print(#list2, list2[1], list2[2])
local function va(...) return {...} end
print(#va(1, nil, 3), #va(), #va("a", "b"))
local big = {}
for i = 1, 300 do big[i] = i end
print(#big, big[300], big[256], big[257])
big[301] = 301
print(#big)
print(#"abc", #"")
local holes = {1, 2, nil, 4}
print(holes[4])
local n = {}
n[1.0] = "int"; n[2.5] = "frac"; n[-0.0] = "zero"
print(n[1], n[2.5], n[0])
local nested = {a = {b = {c = "deep"}}}
print(nested.a.b.c)
t[1], t[2] = t[2], t[1]
print(t[1], t[2])
local constructed = {[1] = "a", [2] = "b", [10] = "j", k = "v"}
print(constructed[1], constructed[10], constructed.k)
local ok, err = pcall(function() local x = {} x[nil] = 1 end)
print(ok, err)
ok, err = pcall(function() local x = {} x[0/0] = 1 end)
print(ok, err)
return #t
