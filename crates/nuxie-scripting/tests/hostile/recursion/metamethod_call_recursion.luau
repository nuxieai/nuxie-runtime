--! attacks: __call that calls its own object, and a __call table chain; expect: ok; within: 15s
local function refused(f, ...)
  local ok, err = pcall(f, ...)
  assert(not ok, "accepted")
  err = tostring(err)
  assert(not string.find(err, "rust panic", 1, true), err)
  return err
end
local obj = setmetatable({}, {})
getmetatable(obj).__call = function(self, n) return 1 + self(n + 1) end
refused(obj, 1)
-- __call is itself a callable table, 10,000 deep
local f = function() return "bottom" end
for i = 1, 10000 do f = setmetatable({}, { __call = f }) end
local ok, v = pcall(f)
assert(not string.find(tostring(v), "rust panic", 1, true), tostring(v))
-- a callable table whose __call is itself
local selfcall = {}
setmetatable(selfcall, { __call = selfcall })
refused(selfcall)
return {}
