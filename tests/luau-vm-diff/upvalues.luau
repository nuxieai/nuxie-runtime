-- GETUPVAL SETUPVAL CLOSEUPVALS NEWCLOSURE DUPCLOSURE CAPTURE
local function counter()
    local n = 0
    return function() n = n + 1; return n end, function() return n end
end
local inc, get = counter()
inc(); inc()
print(get())

-- CLOSEUPVALS: closures capturing a fresh loop local each iteration
local fns = {}
for i = 1, 3 do
    local j = i * 2
    fns[i] = function() return j end
end
print(fns[1](), fns[2](), fns[3]())

local k = 0
local saved = {}
while k < 3 do
    k = k + 1
    local captured = k
    saved[k] = function() captured = captured + 100; return captured end
    if k == 2 then
        -- block exit closes `captured`
    end
end
print(saved[1](), saved[1](), saved[3]())

-- DUPCLOSURE: no mutable captures
local function const() return 42 end
local function make() return function() return const() end end
print(make()(), make() == make())

-- captures by value of an unchanged local, by reference of a mutated one
local fixed = 5
local mutable = 1
local function both() return fixed + mutable end
mutable = 10
print(both())

-- nested upvalues
local function outer()
    local x = 1
    return function()
        return function() x = x + 1; return x end
    end
end
local deep = outer()()
print(deep(), deep())
return get()
