-- Level-sensitive observers of a plain builtin (lane 9, design 4.1): error
-- levels, argument-error names, debug.info and debug.traceback (getfenv and
-- setfenv: frameless_fenv.luau), each with the builtin called directly, through pcall and as a
-- metamethod. A plain builtin runs frameless; whatever looks at the frames
-- must see it exactly as a pushed frame.
local function all(...)
  local t = table.pack(...)
  for i = 1, t.n do t[i] = tostring(t[i]) end
  return "(" .. table.concat(t, ",") .. ")"
end

-- error() is itself a plain builtin: levels 0 to 3, called directly
local function lvl(n) error("e" .. tostring(n), n) end
local function outer(n) lvl(n) end
for n = 0, 3 do
  print(all(pcall(outer, n)))
  print(all(pcall(error, "direct" .. n, n)))
end
-- error raised by a builtin called from a builtin's caller (around)
local function around(n)
  local x = math.abs(-1)
  error("around" .. x, n)
end
for n = 0, 3 do print(all(pcall(around, n))) end

-- argument errors name the builtin (and the method form)
print(all(pcall(function() return math.floor("x") end)))
print(all(pcall(function() return string.rep() end)))
print(all(pcall(function() return ("x"):rep({}) end)))
print(all(pcall(function() return select(0, 1) end)))
print(all(pcall(function() return setmetatable(1, {}) end)))
print(all(pcall(function() return tonumber("1", 99) end)))
print(all(pcall(function() local f = math.max; return f() end)))
print(all(pcall(function() return bit32.band("a") end)))
print(all(pcall(function() return buffer.readu8(buffer.create(1), 5) end)))
print(all(pcall(function() return vector.create("x") end)))
print(all(pcall(math.floor, "x")))
print(all(pcall(string.rep)))

-- debug.info at levels 1 to 3, called directly
local function di(level) return debug.info(level, "sln") end
local function di2(level) return di(level) end
local function viapcall(level) return pcall(debug.info, level, "sln") end
for level = 1, 3 do
  print(all(di2(level)))
  print(all(viapcall(level)))
end
print(all(debug.info(1, "sln")))
-- debug.traceback, called directly and through pcall
local function tb() return debug.traceback("m") end
print(tb())
print(all(pcall(debug.traceback, "p")))
print(all(pcall(debug.traceback)))
-- from an xpcall handler after a builtin raised
print(all(xpcall(function() return math.floor("bad") end, function(m)
  return m .. "|" .. tostring(debug.info(1, "s")) .. "|" .. tostring(debug.info(2, "n")) .. "|" .. debug.traceback("h")
end)))
print(all(xpcall(function() return string.rep() end, debug.traceback)))
print(all(xpcall(function() error("lvl2", 2) end, function(m) return m .. "|" .. tostring(debug.info(2, "l")) end)))

-- as metamethods: a plain builtin as __index, __call, __len, __eq, ...
local mt = {
  __index = rawget,
  __call = select,
  __len = rawlen,
  __eq = rawequal,
  __concat = error,
  __unm = debug.info,
}
local o = setmetatable({1, 2, 3}, mt)
print(o[1], o.missing, o("#", 1, 2), #o)
print(all(pcall(function() return o .. "x" end)))
print(all(pcall(function() return -o end)))
local me = setmetatable({}, {__index = function(t, k) return debug.info(2, "sl") end})
print(all(me.x))
local mtb = setmetatable({}, {__index = function(t, k) return debug.traceback(k) end})
print(mtb.k)
local merr = setmetatable({}, {__index = function(t, k) error("idx " .. k, 2) end})
print(all(pcall(function() return merr.boom end)))
print(all(pcall(function() return setmetatable({}, {__index = math.floor}).x end)))
print(all(pcall(function() return setmetatable({}, {__add = math.max}) + 1 end)))
return "done"
