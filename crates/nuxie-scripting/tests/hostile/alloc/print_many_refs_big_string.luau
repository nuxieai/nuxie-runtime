--! attacks: print with 7,000 references to one 8 MiB string (E1 section 5: ~56 GB line); expect: ok; within: 30s
local big = string.rep("p", 8 * 1024 * 1024)
local args = table.create(7000, big)
print(table.unpack(args))
-- __tostring returning the big string, many times in one line.
local t = setmetatable({}, {__tostring = function() return big end})
local refs = table.create(2000, t)
print(table.unpack(refs))
-- Repeated large prints are each cut and freed.
for _ = 1, 20 do print(big, big, big) end
return {}
