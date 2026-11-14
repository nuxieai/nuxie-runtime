--! attacks: string.format/table.concat/__concat/__eq/__lt/__len metamethods that re-enter format or concat on the same values; expect: ok; within: 10s
local depth = 0
local mt = {}
local v = setmetatable({ n = 1 }, mt)
mt.__tostring = function(self)
  if depth < 4 then depth += 1; local s = tostring(self) .. "|" .. string.format("%s", rawget(self, "n") or 0); depth -= 1; return s end
  return "v"
end
mt.__concat = function(a, b)
  if depth < 4 then depth += 1; local r = (type(a) == "table" and "V" or a) .. tostring(v); depth -= 1; return r end
  return "c"
end
mt.__eq = function(a, b) return tostring(a) == tostring(b) end
mt.__lt = function(a, b) return #tostring(a) < #(tostring(b)) end
mt.__len = function(self) return #tostring(self) end
-- string.format("%s") may or may not honour __tostring here; either way no crash.
pcall(string.format, "%s %s", v, v)
local s = tostring(v)
local c = v .. "x" .. v
local same = v == setmetatable({ n = 2 }, mt)
local less = v < setmetatable({}, mt)
local len = #v
-- table.concat does not use __tostring/__concat; it must refuse the table cleanly.
assert(not pcall(table.concat, { "a", v, "b" }))
-- __tostring returning a non-string through format.
local bad = setmetatable({}, { __tostring = function() return v end })
assert(not pcall(tostring, bad))
-- %q / %d with metamethod tables.
assert(not pcall(string.format, "%d", v))
assert(not pcall(string.format, "%q", v))
return {}
