-- getfenv and setfenv around plain builtins (lane 9, design 4.1), apart
-- from frameless_observers.luau: in one chunk with it the vendored compiler
-- crashes (a dense-hash-table read in luaur_common; recorded in the lane 9
-- evidence).
local function all(...)
  local t = table.pack(...)
  for i = 1, t.n do t[i] = tostring(t[i]) end
  return "(" .. table.concat(t, ",") .. ")"
end
local function fenv1() return getfenv(1) == getfenv(0) end
local okf, envf = pcall(getfenv, 1)
print(fenv1(), getfenv(1) == getfenv(), okf, envf == getfenv(0))
local function g2() local ok, e = pcall(getfenv, 2); return ok, e == getfenv(0) end
print(g2(), all(pcall(getfenv, 50)))
local function setter()
  local t = setmetatable({marker = "new"}, {__index = getfenv(0)})
  setfenv(1, t)
  return marker, math.abs(-3), type(1)
end
print(setter())
local function setter2()
  local function inner()
    setfenv(2, setmetatable({who = "caller"}, {__index = getfenv(0)}))
  end
  inner()
  return who, math.floor(1.5)
end
print(setter2())
print(all(pcall(setfenv, 1, {})))
local ok_sf = pcall(function()
  local env = setmetatable({}, {__index = getfenv(0)})
  setfenv(1, env)
  return math.abs(-2)
end)
print(ok_sf)

local mtf = setmetatable({}, {__index = function(t, k) return getfenv(2) == getfenv(0) end})
print(mtf.k)
print(all(pcall(function() return setmetatable({}, {__index = getfenv}).x end)))
return "done"
