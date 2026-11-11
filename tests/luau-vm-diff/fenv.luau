-- getfenv/setfenv (a chunk naming them compiles without fastcalls).
-- Level 0 is the thread's environment; a numeric string counts as a level.
local base = getfenv(0)
local env = setmetatable({marker = "thread"}, {__index = base})
print(pcall(setfenv, "0", env))
print(getfenv(0).marker, rawequal(getfenv(0), env))
print(pcall(setfenv, 0, base))
print(getfenv(0).marker, rawequal(getfenv(0), base))
local function f() return marker end
print(pcall(setfenv, f, env), f())
print(pcall(setfenv, "x", env))
return "ok"
