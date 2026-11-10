-- CALL RETURN NAMECALL PREPVARARGS GETVARARGS (and CALLFB on v11)
local function multi() return 1, 2, 3 end
local function count(...) return select("#", ...) end
print(count(multi()), count(multi(), 10), count((multi())), count(nil, nil), count())
local function pass(...) return ... end
print(pass(1, nil, 3))
local function tail(n) if n == 0 then return "done" end return tail(n - 1) end
print(tail(1000))
local function fact(n) if n <= 1 then return 1 end return n * fact(n - 1) end
print(fact(10), fact(20))
local function va(a, ...)
    local x, y = ...
    local all = {...}
    return a, x, y, #all, select(2, ...)
end
print(va(1, 2, 3, 4))
print(va())
local obj = {n = 10}
function obj:add(k) self.n = self.n + k; return self end
print(obj:add(5):add(1).n)
print(("abc"):upper(), ("x"):rep(3, ","), ("hello"):sub(2, -2))
local s = "word"
print(s:len(), s:byte(1, -1))
local ok, err = pcall(function() local t = {} t:nomethod() end)
print(ok, err)
ok, err = pcall(function() local x = nil; x() end)
print(ok, err)
ok, err = pcall(function() undefinedfn() end)
print(ok, err)
ok, err = pcall(function() local t = {}; t.f() end)
print(ok, err)
local callable = setmetatable({}, {__call = function(self, a, b) return a + b end})
print(callable(3, 4))
local function rec(n) return n > 0 and rec(n - 1) or "bottom" end
print(rec(150))
ok, err = pcall(function() local function inf(n) return 1 + inf(n) end return inf(1) end)
print(ok, err)
print(select(-1, "a", "b", "c"), select(2, "a", "b"))
ok, err = pcall(select, 0, 1)
print(ok, err)
return multi()
