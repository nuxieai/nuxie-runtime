--! attacks: __tostring that recurses through tostring and through string.format %s; expect: ok; within: 15s
local function refused(f, ...)
  local ok, err = pcall(f, ...)
  assert(not ok, "accepted")
  err = tostring(err)
  assert(not string.find(err, "rust panic", 1, true), err)
  return err
end
local v = setmetatable({}, { __tostring = function(self) return "<" .. tostring(self) .. ">" end })
refused(tostring, v)
local w = setmetatable({}, { __tostring = function(self) return string.format("%s", self) end })
refused(tostring, w)
-- __tostring that returns a non-string
local bad = setmetatable({}, { __tostring = function() return {} end })
refused(tostring, bad)
-- __name / __tostring returning an enormous but bounded string
local big = setmetatable({}, { __tostring = function() return string.rep("y", 100000) end })
assert(#tostring(big) == 100000)
return {}
