-- Re-entry from the host into the interpreter, up to the C-call limit:
-- __index functions, __call, pcall and tostring nesting. Each must end in
-- the same error text, never in a host stack overflow.
local depth = 0
local deep = setmetatable({}, {__index = function(t, k)
    depth = depth + 1
    return t[k]
end})
local ok, err = pcall(function() return deep.x end)
print(ok, err, depth)

depth = 0
local function nest()
    depth = depth + 1
    return pcall(nest)
end
-- `nest` captures itself by value: NEWCLOSURE must store the closure in
-- its register before the captures run.
local results = {nest()}
print(#results, results[1], results[#results], depth)

local callee = setmetatable({}, {__call = function(self, n)
    if n == 0 then return "bottom" end
    return self(n - 1)
end})
print(pcall(callee, 150))
print(pcall(callee, 10000))

local named = {}
setmetatable(named, {__tostring = function(self) return tostring(self) end})
ok, err = pcall(tostring, named)
print(ok, err)

local function lessthan(a, b) return a < b end
local cmp = setmetatable({}, {__lt = function(a, b) return lessthan(a, b) end})
ok, err = pcall(lessthan, cmp, cmp)
print(ok, err)
return "done"
