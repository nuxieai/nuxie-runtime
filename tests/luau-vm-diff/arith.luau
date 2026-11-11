-- ADD SUB MUL DIV MOD POW IDIV ADDK SUBK MULK DIVK MODK POWK IDIVK SUBRK DIVRK MINUS
local function ops(a, b)
    print(a + b, a - b, a * b, a / b, a % b, a ^ b, a // b)
    print(a + 1, a - 1, a * 3, a / 4, a % 3, a ^ 2, a ^ 0.5, a ^ 3, a ^ 4, a // 3)
    print(10 - a, 1 / a, 2 - b, 3 / b, 2 // b, 7 % b)
    print(-a, -(-a), -b + 1)
    return a + b
end
ops(7, 2)
ops(-7.5, 0.5)
ops(0, -0.0)
local a, b = 7, 2
print(-7 % 3, 7 % -3, -7 % -3, 5.5 % 2, -5.5 % 2, 5 % 0, 5 % math.huge, -5 % math.huge)
print(-7 // 2, 7 // -2, 7 // 0, -7 // 0, 0/0 ~= 0/0)
print(2 ^ -1, (-8) ^ (1/3) ~= (-8) ^ (1/3), 0 ^ 0)
print("10" + 1, "3" * "4", 10 .. "", "0x10" + 0, " 5 " * 2, "1e2" / 4)
print(math.huge - math.huge ~= 0, 1e308 * 10, -1e308 * 10)
local x = 0.1
for i = 1, 10 do x = x + 0.1 end
print(x, x * 3, x / 7)
local ok, err = pcall(function() return {} + 1 end)
print(ok, err)
ok, err = pcall(function() local s = "abc"; return s * 2 end)
print(ok, err)
ok, err = pcall(function() local n; return -n end)
print(ok, err)
ok, err = pcall(function() return 1 - nil end)
print(ok, err)
ok, err = pcall(function() local t = {} return t.x.y end)
print(ok, err)
return a + b
