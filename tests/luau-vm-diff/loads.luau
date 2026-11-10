-- LOADNIL LOADB LOADN LOADK MOVE RETURN
local a, b, c = nil, true, false
local n, big, neg, frac = 7, 40000, -12, 0.25
local s = "hello"
local m = a
local copy = s
print(a, b, c, n, big, neg, frac, s, m, copy)
print(1e300 * 1e10, -1e300 * 1e10, 0/0 ~= 0/0, -0.0, 1/(-0.0))
print(123456789012345, 2^53, 2^63, 1e15, 1e16, 0.1 + 0.2, 1/3, 100, 1e-5, 1e-7)
return a, b, c, n, s
