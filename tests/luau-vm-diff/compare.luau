-- JUMPIFEQ JUMPIFLE JUMPIFLT JUMPIFNOTEQ JUMPIFNOTLE JUMPIFNOTLT JUMPXEQKNIL JUMPXEQKB JUMPXEQKN JUMPXEQKS
local function cmp(a, b)
    local r = {}
    if a == b then r[#r + 1] = "eq" end
    if a ~= b then r[#r + 1] = "ne" end
    if a < b then r[#r + 1] = "lt" end
    if a <= b then r[#r + 1] = "le" end
    if a > b then r[#r + 1] = "gt" end
    if a >= b then r[#r + 1] = "ge" end
    if not (a < b) then r[#r + 1] = "nlt" end
    if not (a <= b) then r[#r + 1] = "nle" end
    return table.concat(r, ",")
end
print(cmp(1, 2), cmp(2, 1), cmp(2, 2))
print(cmp("a", "b"), cmp("b", "a"), cmp("abc", "abd"), cmp("", "a"), cmp("a\0b", "a\0c"))
print(cmp(0/0, 0/0), cmp(-0.0, 0.0), cmp(math.huge, -math.huge))
local function consts(v)
    local r = {}
    if v == nil then r[#r + 1] = "nil" end
    if v ~= nil then r[#r + 1] = "notnil" end
    if v == true then r[#r + 1] = "true" end
    if v == false then r[#r + 1] = "false" end
    if v ~= false then r[#r + 1] = "notfalse" end
    if v == 3 then r[#r + 1] = "three" end
    if v ~= 3 then r[#r + 1] = "notthree" end
    if v == "s" then r[#r + 1] = "s" end
    if v ~= "s" then r[#r + 1] = "nots" end
    return table.concat(r, ",")
end
print(consts(nil), consts(true), consts(false), consts(3), consts("s"), consts({}))
print(1 == "1", "1" == 1, {} == {}, nil == false)
local t = {}
print(t == t, rawequal(t, t), rawequal(t, {}))
local ok, err = pcall(function() return 1 < "2" end)
print(ok, err)
ok, err = pcall(function() return {} < {} end)
print(ok, err)
ok, err = pcall(function() return nil <= 1 end)
print(ok, err)
local x = 5
print(x == 5, x < 10, x <= 5, x > 5, 3 < x)
return cmp(1, 1)
