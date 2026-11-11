-- GETGLOBAL SETGLOBAL GETIMPORT
counter = 1
counter = counter + 1
local function bump() counter = counter * 10 end
bump()
print(counter, rawget(_G, "counter"), type(math.floor), math.floor(2.5), string.format("%d", 3))
missing_global = nil
print(missing_global, undefined_thing)
g1, g2 = 3, 4
print(g1 + g2)
return counter
