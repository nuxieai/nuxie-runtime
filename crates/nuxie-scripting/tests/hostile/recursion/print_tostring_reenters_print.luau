--! attacks: print of a value whose __tostring calls print on itself (host callback re-entered without bound); expect: ok; within: 20s
local depth = 0
local v = setmetatable({}, { __tostring = function(self)
  depth = depth + 1
  print(self)
  return "never"
end })
local ok, err = pcall(print, v)
assert(not ok, "print recursion returned normally")
err = tostring(err)
assert(not string.find(err, "rust panic", 1, true), err)
assert(depth > 1, "never recursed")
-- the same through a second host entry: tostring -> print -> tostring
depth = 0
local w = setmetatable({}, { __tostring = function(self) depth = depth + 1 return tostring(print(self)) end })
local ok2, err2 = pcall(tostring, w)
assert(not ok2 and not string.find(tostring(err2), "rust panic", 1, true), tostring(err2))
-- print still works
return {}
