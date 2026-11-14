--! attacks: long but bounded VM string work (string.rep 8 MiB, gsub over 1 MiB with string and table replacements, string.format/concat loops); expect: either; within: 20s
local s = string.rep("ab", 4 * 1024 * 1024)
assert(#s == 8 * 1024 * 1024)
local r, n = string.gsub(string.rep("x", 1048576), "x", "yy")
assert(n == 1048576 and #r == 2 * 1048576)
local r2 = string.gsub(string.rep("ab", 262144), "%a", { a = "1", b = "22" })
assert(#r2 == 3 * 262144)
local parts = table.create(20000, "")
for i = 1, 20000 do parts[i] = string.format("%08d", i) end
assert(#table.concat(parts, ",") == 20000 * 9 - 1)
assert(string.find(string.rep("a", 1000000) .. "b", "b", 1, true) == 1000001)
return {}
