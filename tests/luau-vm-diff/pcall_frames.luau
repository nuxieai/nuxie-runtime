-- `pcall` of a Lua function, which `CALL` enters in the dispatch loop
-- (docs/luau-safe-vm-register-window.md, 4.7), against every other way
-- of reaching it: the frames, the catch, yields, observers and the C-call
-- counters must be the native `pcall`'s.

local function show(...)
	local n = select("#", ...)
	local parts = table.create(n)
	for i = 1, n do
		parts[i] = tostring((select(i, ...)))
	end
	print(n, table.concat(parts, " "))
end

-- Results, adjusted to every count.
local function three() return 1, 2, 3 end
local function none() end
show(pcall(three))
show(pcall(none))
local a, b = pcall(three)
print(a, b)
local c = pcall(three)
print(c)
print((pcall(none)))
show(pcall(function(...) return ... end, "x", nil, "y"))
show(pcall(function(...) return select("#", ...) end))
local t = { pcall(three) }
print(#t, t[1], t[4])

-- An error inside, raised at several depths and with several values.
show(pcall(function() error("boom") end))
show(pcall(function() error("boom", 0) end))
print(select(2, pcall(function() error({ code = 7 }) end)).code)
show(pcall(function() error() end))
show(pcall(function() local x = nil; return x.y end))
local function deep(n) if n == 0 then error("deep", 0) end return deep(n - 1) end
show(pcall(deep, 50))
-- Upvalues captured inside close when the error unwinds.
local get
show(pcall(function()
	local v = 1
	get = function() return v end
	v = 2
	error("after capture", 0)
end))
print(get())

-- Nested pcall: the inner catches, the outer sees its results.
show(pcall(pcall, error, "native-in-native"))
show(pcall(function() return pcall(function() error("inner", 0) end) end))
show(pcall(function()
	local ok, e = pcall(error, "x", 0)
	error("outer:" .. tostring(ok) .. ":" .. tostring(e), 0)
end))
local function nest(n)
	if n == 0 then return "bottom" end
	local ok, v = pcall(nest, n - 1)
	return v
end
print(nest(40))

-- pcall of a native and of a value with `__call` (the native path).
show(pcall(math.max, 3, 9, 4))
show(pcall(tostring, nil))
show(pcall(error, "native error", 0))
local callable = setmetatable({}, { __call = function(self, x) return "called", x end })
show(pcall(callable, 5))
local bad = setmetatable({}, { __call = function() error("in __call", 0) end })
show(pcall(bad))
show(pcall(42))
show(pcall(pcall))

-- debug.info levels and a traceback through it.
local function where()
	return debug.info(1, "sl"), debug.info(2, "s"), debug.info(3, "s")
end
show(pcall(where))
local function names()
	local out = {}
	for level = 1, 3 do
		local name = debug.info(level, "n")
		out[#out + 1] = tostring(name)
	end
	return table.concat(out, ",")
end
show(pcall(names))
local tb = select(2, pcall(function() return debug.traceback("tb") end))
print((tb:gsub("0x%x+", "ADDR")))

-- A yield inside, then a return, then an error, from a coroutine.
local co = coroutine.create(function()
	local ok, v = pcall(function()
		local got = coroutine.yield("first")
		return "returned " .. tostring(got)
	end)
	coroutine.yield(tostring(ok) .. " " .. tostring(v))
	local ok2, e = pcall(function()
		coroutine.yield("second")
		error("after yield", 0)
	end)
	return tostring(ok2) .. " " .. tostring(e)
end)
show(coroutine.resume(co))
show(coroutine.resume(co, "R"))
show(coroutine.resume(co))
show(coroutine.resume(co))
print(coroutine.status(co))
local wrapped = coroutine.wrap(function()
	print(coroutine.isyieldable())
	local ok = pcall(function() print(coroutine.isyieldable()) end)
	return ok
end)
print(wrapped())

-- The C-call counters after caught errors: as many levels fit afterwards.
local function depth(n)
	local ok = pcall(depth, n + 1)
	if not ok then return n end
	return n
end
for _ = 1, 30 do pcall(error, "x") end
for _ = 1, 30 do pcall(function() error("x") end) end
local function levels()
	local count = 0
	local function go()
		count += 1
		local ok = pcall(go)
		return ok
	end
	pcall(go)
	return count
end
print(levels())
print(levels())
print(select("#", depth(0)))
print(coroutine.isyieldable())
