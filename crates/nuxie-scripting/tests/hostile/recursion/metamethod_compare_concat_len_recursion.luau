--! attacks: __eq, __lt, __le, __concat, __len and __unm metamethods recursing through their own operator; expect: ok; within: 20s
local function refused(f, ...)
  local ok, err = pcall(f, ...)
  assert(not ok, "accepted")
  err = tostring(err)
  assert(not string.find(err, "rust panic", 1, true), err)
  return err
end
local mt = {}
mt.__eq = function(a, b) return a == setmetatable({}, mt) end
mt.__lt = function(a, b) return a < b end
mt.__le = function(a, b) return b <= a end
mt.__concat = function(a, b) return a .. b end
mt.__len = function(a) return #a + 1 end
mt.__unm = function(a) return -a end
mt.__add = function(a, b) return a + b end
local x, y = setmetatable({}, mt), setmetatable({}, mt)
refused(function() return x == y end)
refused(function() return x < y end)
refused(function() return x <= y end)
refused(function() return x .. y end)
refused(function() return #x end)
refused(function() return -x end)
refused(function() return x + 1 end)
return {}
