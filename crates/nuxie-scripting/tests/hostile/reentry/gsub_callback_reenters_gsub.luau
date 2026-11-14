--! attacks: string.gsub/gmatch/find callbacks and replacement tables that re-enter gsub on the same string and pattern; expect: ok; within: 10s
local s = string.rep("ab", 6)
local depth, calls = 0, 0
local function cb(m)
  calls += 1
  if depth < 2 then
    depth += 1
    local inner = string.gsub(s, "a", cb)
    for w in string.gmatch(inner, "b+") do calls += #w > 0 and 0 or 1 end
    depth -= 1
  end
  return "A"
end
local out = string.gsub(s, "a", cb, 4)
assert(#out == #s, #out)
-- Replacement table whose __index calls gsub on the same subject.
local repl = setmetatable({}, { __index = function(_, k)
  if depth < 2 then depth += 1; local r = string.gsub(s, "(b)", getmetatable(_).__self); depth -= 1 end
  return "Z"
end })
getmetatable(repl).__self = repl
out = string.gsub(s, "(a)", repl, 3)
-- Callback returning values that need conversion with metamethods.
local ok = pcall(string.gsub, s, "a", function() return setmetatable({}, { __tostring = function() return string.gsub(s, "a", "x") end }) end)
assert(not ok, "a table replacement is invalid")
-- gmatch iterator advanced from inside its own callback chain.
local it = string.gmatch(s, "(a)(b)")
local seen = 0
string.gsub("xxxx", "x", function()
  local a = it()
  if a then seen += 1 end
  return ""
end)
assert(seen == 4, seen)
-- Callback errors with an object whose __tostring calls gsub.
ok = pcall(string.gsub, s, "a", function() error(setmetatable({}, { __tostring = function() return (string.gsub(s, "a", "q")) end })) end)
assert(not ok)
return {}
