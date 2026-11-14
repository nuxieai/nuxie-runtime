--! attacks: catastrophic pattern backtracking (a*a*...a*b over 3,000 a's) in string.find, gsub and gmatch; expect: error(exceeded); within: 30s
local s = string.rep("a", 3000)
local pat = string.rep("a*", 12) .. "b"
local ok, e = pcall(string.find, s, pat)
-- reached only if the matcher returned: try the other entry points
assert(not string.find(tostring(e), "rust panic", 1, true), tostring(e))
pcall(string.gsub, s, pat, "")
for _ in string.gmatch(s, pat) do end
error("backtracking finished without hitting the budget: exceeded nothing")
