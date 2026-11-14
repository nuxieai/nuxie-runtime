--! attacks: print of a value whose __tostring calls print and tostring on itself (bounded re-entry of print); expect: ok; within: 10s
local depth = 0
local maxDepth = 0
local t
t = setmetatable({}, { __tostring = function(self)
  depth += 1
  if depth > maxDepth then maxDepth = depth end
  if depth < 8 then
    print("nested", self)
  end
  depth -= 1
  return "t@" .. depth
end })
print(t, t, t)
assert(maxDepth >= 8, "print re-entered " .. maxDepth)
-- A __tostring that returns a non-string from inside print.
local bad = setmetatable({}, { __tostring = function() print("inner"); return {} end })
local ok, err = pcall(print, bad)
assert(not ok, "a non-string __tostring must fail")
-- A __tostring that raises from inside print, with an error object that itself has __tostring calling print.
local raiser = setmetatable({}, { __tostring = function()
  error(setmetatable({}, { __tostring = function() print("in error tostring"); return "boxed" end }))
end })
ok, err = pcall(print, raiser)
assert(not ok)
-- The error object crosses print, a host function: it may arrive as itself or as text, never as a crash.
assert(tostring(err) ~= nil)
return {}
