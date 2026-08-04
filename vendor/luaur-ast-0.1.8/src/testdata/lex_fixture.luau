local x = 1 + 2.5e-3 // 4
local s = "hi\n\"there\"" .. 'q'
local r = [==[ raw
multi ]==]
local t = `a{x}b{ y }c`
local u = `simple`
function f(a, b): number
    return a <= b and a ~= 2 or not b
end
-- line comment
--[[ block
comment ]]
local q = 0xFF + .5
@native
local z = foo.bar:baz(1, 2)
local map = {[1] = 2; a = 3, ["k"] = 4}
x += 1
x //= 2
s ..= "z"
