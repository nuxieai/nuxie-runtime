--! attacks: numeric for loops to math.huge, with a 2^-60 step, and a NaN bound; expect: error(exceeded); within: 15s
for i = 1, 0 / 0 do error("NaN bound ran the body") end
for i = 1, -math.huge do error("-huge bound ran the body") end
local okz, ez = pcall(function() for i = 1, 10, 0 do end end)
assert(not string.find(tostring(ez), "rust panic", 1, true), tostring(ez))
for i = 1, math.huge, 2 ^ -60 do end
