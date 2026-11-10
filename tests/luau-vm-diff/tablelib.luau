-- table library
local t = {}
table.insert(t, "a")
table.insert(t, "c")
table.insert(t, 2, "b")
table.insert(t, 1, "z")
print(#t, table.concat(t, ","))
print(table.remove(t), table.remove(t, 1), #t, table.concat(t, ","))
print(table.remove({}), table.remove({}, 0), #t)
print(pcall(table.insert, t, 10, "x"))
print(pcall(table.insert, t, 1, 2, 3))
print(table.concat({1, 2, 3}), table.concat({1, 2, 3}, ", ", 2), table.concat({1, 2, 3}, "", 3, 2))
print(pcall(table.concat, {1, {}, 3}))
print(pcall(table.concat, {1, 2}, ",", 1, 5))
local p = table.pack(1, nil, 3)
print(p.n, p[1], p[2], p[3])
print(table.unpack({1, 2, 3}), table.unpack({1, 2, 3}, 2), table.unpack({1, 2, 3}, 2, 3), table.unpack({}, 1, 0))
print(select("#", table.unpack({1, nil, 3}, 1, 3)))
local big = {}
for i = 1, 100 do table.insert(big, i) end
for i = 1, 50 do table.remove(big, 1) end
print(#big, big[1], big[50])
print(pcall(table.remove, setmetatable({}, {__metatable = 1})))
return #t
