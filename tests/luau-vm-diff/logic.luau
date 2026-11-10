-- AND OR ANDK ORK NOT JUMPIF JUMPIFNOT
local function f(a, b)
    return a and b, a or b, a and 1, a or 2, not a, not not a
end
print(f(nil, 1))
print(f(false, "x"))
print(f(0, false))
print(f("", nil))
local x = nil
local y = x or "default"
local z = x and x.field
print(y, z)
local count = 0
local function side() count = count + 1; return true end
local _ = false and side()
local _ = true or side()
local _ = true and side()
print(count)
local v = 3
if v then print("truthy") else print("falsy") end
if not v then print("no") end
while v do v = v > 0 and v - 1 or nil end
print(v)
return (1 and 2) or 3
