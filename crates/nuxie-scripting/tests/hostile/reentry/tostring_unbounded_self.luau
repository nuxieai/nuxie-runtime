--! attacks: __tostring that calls tostring on itself without bound, through print and string.format; expect: ok; within: 15s
local t = setmetatable({}, {})
getmetatable(t).__tostring = function(self) return tostring(self) end
local ok, err = pcall(tostring, t)
assert(not ok, "unbounded __tostring recursion must fail")
local u = setmetatable({}, { __tostring = function(self) return string.format("%s", self) end })
ok, err = pcall(print, u)
assert(not ok, "unbounded format recursion must fail")
local v = setmetatable({}, { __tostring = function(self) print(self); return "x" end })
ok, err = pcall(string.format, "%s", v)
assert(not ok, "unbounded print recursion must fail")
return {}
