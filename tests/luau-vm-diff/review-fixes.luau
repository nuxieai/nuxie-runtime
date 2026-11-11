-- Cases from the 2026-11-11 codex review.
-- math.modf: a zero fraction carries the sign of an integral argument.
local function frac(x) local _, f = math.modf(x); return 1 / f end
print(frac(-1), frac(-0.0), frac(2), frac(-math.huge), frac(math.huge))
-- a backreference to a position capture never matches
print(string.match("a", "()%1"), string.find("aa", "()%1"), ("xx"):gsub("()%1", "y"))
-- hexadecimal scaling near the ends of the double range
print(tonumber("0x1000000000000000p-1100"), tonumber("0x0p99999"), tonumber("0x1p99999"), tonumber("0x1p-1074"), tonumber("0x1p1023"), tonumber("0x1p1024"))
return "ok"
