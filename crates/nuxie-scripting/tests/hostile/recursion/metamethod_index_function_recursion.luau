--! attacks: __index and __newindex functions that recurse into themselves through the metamethod; expect: ok; within: 15s
local function refused(f, ...)
  local ok, err = pcall(f, ...)
  assert(not ok, "accepted")
  err = tostring(err)
  assert(not string.find(err, "rust panic", 1, true), err)
  return err
end
local t = setmetatable({}, { __index = function(self, k) return self[k .. "x"] end })
refused(function() return t.a end)
local u = setmetatable({}, { __newindex = function(self, k, v) self[k .. "x"] = v end })
refused(function() u.a = 1 end)
-- the metamethod recurses through rawget/getmetatable lookups too
local w
w = setmetatable({}, { __index = function(self, k) return getmetatable(self).__index(self, k) end })
refused(function() return w.a end)
return {}
