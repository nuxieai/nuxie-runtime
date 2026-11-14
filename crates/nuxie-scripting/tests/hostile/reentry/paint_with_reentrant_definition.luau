--! attacks: Paint.with definitions and paint field writes whose metamethods re-enter the paint being built or written; expect: ok; within: 10s
local target = Paint.new()
local def = setmetatable({ color = 0xff00ff00, thickness = 2, style = "stroke" }, {
  __index = function(t, k) target.color = 0xff0000ff; return Paint.with({ thickness = 9 }) end,
  __iter = function(t)
    target.thickness = 99
    return next, t
  end,
})
local ok, p = pcall(Paint.with, def)
assert(ok, tostring(p))
-- A definition value that is a userdata of the wrong kind, and one that is a table with __tostring calling Paint.with.
ok = pcall(Paint.with, { color = Path.new() })
assert(not ok)
ok = pcall(Paint.with, { thickness = setmetatable({}, { __tostring = function() Paint.with({ color = 1 }); return "3" end }) })
assert(not ok)
-- A gradient value that is a plain table with __index: accepted or refused, never a crash.
pcall(Paint.with, { gradient = setmetatable({}, { __index = function() return Paint.new() end }) })
-- Field writes on a paint from a metamethod that writes the same paint.
local paint = Paint.new()
local writes = 0
local sneaky = setmetatable({}, { __tostring = function() writes += 1; paint.color = 0xff112233; return "x" end })
ok = pcall(function() paint.color = sneaky end)
assert(not ok)
ok = pcall(function() paint.style = sneaky end)
assert(not ok)
-- copy() of the paint from inside a __index triggered while the definition is read.
local copies = {}
local def2 = setmetatable({}, { __index = function() table.insert(copies, paint:copy()); return nil end })
def2.thickness = 1
ok = pcall(Paint.with, def2)
assert(ok)
-- The definition table is the paint's gradient's stops at the same time.
local shared = { { position = 0, color = 0xff000000 }, color = 0xffffffff }
local g = Gradient.linear(Vector.xy(0, 0), Vector.xy(1, 0), shared)
shared.gradient = g
ok = pcall(Paint.with, shared)
assert(ok)
return {}
