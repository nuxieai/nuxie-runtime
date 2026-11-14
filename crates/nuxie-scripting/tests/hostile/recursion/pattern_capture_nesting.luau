--! attacks: Lua patterns with 5,000 nested captures, 100,000 chained items, and deep %b/%f nesting in find/match/gsub; expect: ok; within: 20s
local function refused(f, ...)
  local ok, err = pcall(f, ...)
  assert(not ok, "accepted")
  err = tostring(err)
  assert(not string.find(err, "rust panic", 1, true), err)
  return err
end
refused(string.find, "a", string.rep("(", 5000) .. "a" .. string.rep(")", 5000))
refused(string.gsub, "a", string.rep("(", 300) .. "a" .. string.rep(")", 300), "%1")
local ok, e = pcall(string.match, string.rep("a", 100000), string.rep("a?", 100000))
assert(not string.find(tostring(e), "rust panic", 1, true), tostring(e))
-- balanced-match over 200,000 nested brackets
local br = string.rep("(", 100000) .. string.rep(")", 100000)
ok, e = pcall(string.find, br, "%b()")
assert(not string.find(tostring(e), "rust panic", 1, true), tostring(e))
-- back references to captures that do not exist / are unclosed
refused(string.find, "aa", "(a)%2")
refused(string.find, "aa", "(a%1)")
refused(string.find, "a", "%")
refused(string.find, "a", "[a")
return {}
