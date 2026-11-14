--! attacks: string.gsub callback re-entering gsub without bound, a gmatch iterator re-entered from a gsub callback, and a gsub table replacement whose __index recurses into gsub; expect: ok; within: 20s
local function nopanic(e) assert(not string.find(tostring(e), "rust panic", 1, true), tostring(e)) end
local depth = 0
local function rep(m)
  depth = depth + 1
  return (string.gsub("abc", ".", rep))
end
local ok, e = pcall(string.gsub, "xyz", ".", rep)
assert(not ok, "unbounded gsub recursion returned normally")
nopanic(e)
assert(depth > 10, "recursion stopped at " .. depth)
-- the same gmatch iterator advanced from inside a gsub callback while the for loop drives it
local it = string.gmatch("a b c d e f g h", "%a")
local seen = 0
for w in it do
  seen = seen + 1
  string.gsub("xy", ".", function() local nxt = it() return nxt or "" end)
end
assert(seen >= 1)
-- gsub with a table replacement whose __index recurses into gsub
local rt
rt = setmetatable({}, { __index = function(_, k) return (string.gsub(k .. k, ".", rt)) end })
local ok3, e3 = pcall(string.gsub, "q", ".", rt)
assert(not ok3, "unbounded __index/gsub recursion returned normally")
nopanic(e3)
return {}
