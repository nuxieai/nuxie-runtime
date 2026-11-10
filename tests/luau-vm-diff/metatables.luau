-- __index __newindex __call __eq __lt __le __unm __len __arith __tostring __metatable __type
local log = {}
local base = {greet = function() return "hi" end}
local proxy = setmetatable({}, {__index = base})
print(proxy.greet(), proxy.missing)
local computed = setmetatable({}, {__index = function(t, k) return k .. "!" end})
print(computed.abc, computed[1])
local chain = setmetatable({}, {__index = setmetatable({}, {__index = {deep = "found"}})})
print(chain.deep)
local store = {}
local guarded = setmetatable({}, {__newindex = function(t, k, v) rawset(store, k, v * 2) end})
guarded.x = 5
print(rawget(guarded, "x"), store.x)
local redirect = setmetatable({}, {__newindex = store})
redirect.y = 7
print(rawget(redirect, "y"), store.y)

local V = {}
V.__index = V
V.__add = function(a, b) return setmetatable({v = a.v + b.v}, V) end
V.__sub = function(a, b) return setmetatable({v = a.v - b.v}, V) end
V.__mul = function(a, b)
    if type(a) == "number" then return setmetatable({v = a * b.v}, V) end
    if type(b) == "number" then return setmetatable({v = a.v * b}, V) end
    return setmetatable({v = a.v * b.v}, V)
end
V.__div = function(a, b) return setmetatable({v = a.v / b.v}, V) end
V.__mod = function(a, b) return setmetatable({v = a.v % b.v}, V) end
V.__pow = function(a, b) return setmetatable({v = a.v ^ b.v}, V) end
V.__idiv = function(a, b) return setmetatable({v = a.v // b.v}, V) end
V.__unm = function(a) return setmetatable({v = -a.v}, V) end
V.__len = function(a) return a.v end
V.__eq = function(a, b) return a.v == b.v end
V.__lt = function(a, b) return a.v < b.v end
V.__le = function(a, b) return a.v <= b.v end
V.__tostring = function(a) return "V(" .. a.v .. ")" end
V.__call = function(self, k) return self.v + k end
local function new(v) return setmetatable({v = v}, V) end
local p, q = new(6), new(4)
print(tostring(p + q), tostring(p - q), tostring(p * q), tostring(p / q), tostring(p % q))
print(tostring(p ^ q), tostring(p // q), tostring(-p), #p, tostring(2 * p), tostring(p * 3))
print(p == q, p == new(6), p ~= new(6), p < q, p <= q, p > q, p >= q, q < p)
print(p(10), tostring(new(1.5)))
local locked = setmetatable({}, {__metatable = "locked"})
print(getmetatable(locked))
local ok, err = pcall(setmetatable, locked, {})
print(ok, err)
print(getmetatable("abc").__index == string)
local typed = setmetatable({}, {__type = "Custom"})
print(type(typed), typeof(typed))
local named = setmetatable({}, {__tostring = function() return "named" end})
print(tostring(named), named)
ok, err = pcall(tostring, setmetatable({}, {__tostring = function() return 1 end}))
print(ok, err)
local eqcount = 0
local E = {__eq = function(a, b) eqcount = eqcount + 1; return true end}
local e1, e2 = setmetatable({}, E), setmetatable({}, E)
print(e1 == e2, e1 == e1, eqcount)
local tm_index_err = setmetatable({}, {__index = function() error("from index") end})
ok, err = pcall(function() return tm_index_err.x end)
print(ok, err)
ok, err = pcall(function() local x = setmetatable({}, {__index = 5}); return x.y end)
print(ok, err)
return #p
