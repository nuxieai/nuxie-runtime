-- Cases from the 2026-11-11 codex review.
-- math.modf: a zero fraction carries the sign of an integral argument.
local function frac(x) local _, f = math.modf(x); return 1 / f end
print(frac(-1), frac(-0.0), frac(2), frac(-math.huge), frac(math.huge))
-- the library body (no fast path: pcall; getfenv is in modf-unsafe.luau) is
-- x - trunc(x); results of the fast path also land right under MULTRET
local function libfrac(x) local _, _, f = pcall(math.modf, x); return 1 / f end
print(libfrac(-1), libfrac(2), libfrac(math.huge), libfrac(-2.5))
print(math.modf(-3))
print(select("#", math.modf(7.25)), ({math.modf(-0.5)})[2])
-- the FASTCALL2/2K forms locate their CALL past the aux word
local function frac2(x, y) local _, f = math.modf(x, y); return 1 / f end
local function frac2k(x) local _, f = math.modf(x, 2); return 1 / f end
print(frac2(-1, 2), frac2(math.huge, 0), frac2k(-1), frac2k(-math.huge))
-- math.clamp refuses unordered bounds
print(pcall(math.clamp, 1, 0, 0 / 0), pcall(math.clamp, 1, 0 / 0, 2), math.clamp(0 / 0, 0, 1) ~= math.clamp(0 / 0, 0, 1))
-- a backreference to a position capture never matches
print(string.match("a", "()%1"), string.find("aa", "()%1"), ("xx"):gsub("()%1", "y"))
-- hexadecimal scaling near the ends of the double range
print(tonumber("0x1000000000000000p-1100"), tonumber("0x0p99999"), tonumber("0x1p99999"), tonumber("0x1p-1074"), tonumber("0x1p1023"), tonumber("0x1p1024"))
-- subnormal hex values round once
print(tonumber("0x17ffffffffffffffp-1134"), tonumber("0x18p-1078"), tonumber("0x1p-1075"), tonumber("0x3p-1076"), tonumber("0x1.fffffffffffff8p1023"))
return "ok"
