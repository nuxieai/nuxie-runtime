-- math.modf without the fast path. A chunk that names getfenv compiles
-- without fastcalls, and calling getfenv clears safeenv at run time, so
-- every call here takes the library body: x - trunc(x).
local function frac(x) local _, f = math.modf(x); return 1 / f end
print(frac(-1), frac(2), frac(math.huge), frac(-2.5))
local _ = getfenv()
print(frac(-1), frac(-0.0), frac(-math.huge))
print(math.modf(-3))
return "ok"
