-- FASTCALL under safeenv after a library function is replaced
-- (docs/luau-safe-vm-call-path.md, 4.2): the compiler resolved the import
-- when the chunk loaded and the fast body runs, so the replacement is not
-- called; through a variable it is.
local function f(x) return math.abs(x) end
local function g(x) return math.max(x, 1, 2, 3) end
local function h(t) table.insert(t, "v"); return #t end
local function k(x) return type(x) end
print(f(-2), g(-2), h({}), k(1))
math.abs = function() return "other abs" end
math.max = function() return "other max" end
table.insert = function() error("other insert") end
print(f(-2), g(-2), h({}), k(1))
print(pcall(math.abs, -2))
local abs = math.abs
print(abs(-2))
print(pcall(function() return math.abs("x") end))
print(pcall(function() return math.max("x", 1) end))
