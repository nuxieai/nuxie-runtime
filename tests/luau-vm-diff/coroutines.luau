-- The coroutine library (lane 4): create, resume, yield, status, wrap,
-- running, isyieldable, close, with upstream's statuses and error text.
local function show(...)
    local t = table.pack(...)
    local parts = {}
    for i = 1, t.n do
        local v = t[i]
        parts[i] = type(v) == "thread" and "<thread>" or tostring(v)
    end
    print(table.concat(parts, " "))
end

-- Arguments in, yields out, results at the end.
local co = coroutine.create(function(a, b)
    show("start", a, b, coroutine.isyieldable())
    local c, d = coroutine.yield(a + b)
    show("resumed", c, d)
    local e = coroutine.yield()
    return e, "end"
end)
show(coroutine.status(co))
show(coroutine.resume(co, 1, 2))
show(coroutine.status(co))
show(coroutine.resume(co, "x", "y"))
show(coroutine.resume(co, 99))
show(coroutine.status(co))
show(coroutine.resume(co))

-- running, status of a normal coroutine, resuming a running one.
local outer
outer = coroutine.create(function()
    show("running is outer", coroutine.running() == outer)
    local inner = coroutine.create(function()
        show("outer is", coroutine.status(outer))
        show(coroutine.resume(outer))
        show(coroutine.resume(coroutine.running()))
        return "inner done"
    end)
    show(coroutine.resume(inner))
    show(coroutine.status(inner))
end)
show(coroutine.resume(outer))
show(coroutine.isyieldable())
show(pcall(coroutine.yield, 1))

-- Errors kill a coroutine; its resumer gets the value.
local bad = coroutine.create(function(x) error({code = x}) end)
local ok, err = coroutine.resume(bad, 7)
show(ok, type(err), err.code, coroutine.status(bad))
local bad2 = coroutine.create(function() local t = nil; return t.x end)
show(coroutine.resume(bad2))
show(coroutine.resume(bad2))
show(pcall(coroutine.resume, 42))
show(pcall(coroutine.status, {}))
show(pcall(coroutine.create, 1))

-- wrap: results directly, errors rethrown with the caller's position.
local gen = coroutine.wrap(function(n)
    for i = 1, n do coroutine.yield(i, i * i) end
    return "last"
end)
show(gen(3))
show(gen())
show(gen())
show(gen())
show(pcall(gen))
local thrower = coroutine.wrap(function() error("inside") end)
show(pcall(thrower))
local thrower2 = coroutine.wrap(function() error(setmetatable({}, {__tostring = function() return "obj" end})) end)
local ok2, e2 = pcall(thrower2)
show(ok2, tostring(e2))

-- A wrap iterator in a generic for (the call is a frame and may yield).
local sum = 0
for i, sq in coroutine.wrap(function() for i = 1, 5 do coroutine.yield(i, i * i) end end) do
    sum = sum + i + sq
end
show("for sum", sum)

-- Yields across pcall and xpcall, errors after a yield.
local function colog(f)
    local c = coroutine.create(f)
    local log = {}
    while coroutine.status(c) == "suspended" do
        local r = table.pack(coroutine.resume(c))
        table.insert(log, r[1] and (coroutine.status(c) == "suspended" and "yield" or "return") or "error")
        for i = 2, r.n do table.insert(log, tostring(r[i])) end
    end
    show(table.unpack(log))
end
colog(function() return pcall(function() coroutine.yield(1) return 42 end) end)
colog(function() return pcall(function() coroutine.yield() error("after") end) end)
colog(function() return xpcall(function() coroutine.yield(2) error("x") end, function(m) return "handled " .. m end) end)
colog(function() return pcall(pcall, function() coroutine.yield(3) return "deep" end) end)
colog(function()
    local ok, e = pcall(function() coroutine.yield(4) local t = nil; return t.y end)
    coroutine.yield(ok, e)
    return "after pcall"
end)

-- Yield across a metamethod or a library callback is an error.
local mt = setmetatable({}, {__index = function(t, k) return coroutine.yield(k) end})
show(coroutine.resume(coroutine.create(function() return mt.key end)))
show(coroutine.resume(coroutine.create(function()
    return table.sort({3, 2, 1}, function(a, b) coroutine.yield() return a < b end)
end)))
show(coroutine.resume(coroutine.create(function()
    return string.gsub("ab", ".", function(c) coroutine.yield(c) end)
end)))
show(coroutine.resume(coroutine.create(function()
    return tostring(setmetatable({}, {__tostring = function() coroutine.yield() return "t" end}))
end)))
-- The xpcall handler cannot yield either.
show(coroutine.resume(coroutine.create(function()
    return xpcall(function() error("e") end, function(m) coroutine.yield() return m end)
end)))

-- close: suspended and dead coroutines, with and without an error.
local f
local open = coroutine.create(function()
    local a = 42
    f = function() return a end
    coroutine.yield()
    a = 20
end)
coroutine.resume(open)
show(coroutine.close(open))
show(coroutine.status(open), f())
show(coroutine.close(open))
local failed = coroutine.create(function() error("boom", 0) end)
coroutine.resume(failed)
show(coroutine.close(failed))
show(coroutine.close(failed))
show(pcall(coroutine.close, coroutine.running and 1))
coroutine.wrap(function() show(pcall(coroutine.close, coroutine.running())) end)()

-- Upvalues shared between threads, and a coroutine's own stack growing.
local counter = 0
local inc = coroutine.wrap(function()
    while true do
        counter = counter + 1
        coroutine.yield(counter)
    end
end)
inc(); inc()
show("counter", counter, inc())
local deep = coroutine.wrap(function()
    local function r(n) if n == 0 then return coroutine.yield("bottom") end return r(n - 1) end
    return r(500)
end)
show(deep())
show(deep("back"))

-- Values yielded and passed back in bulk.
local bulk = coroutine.create(function(...) return select("#", coroutine.yield(...)) end)
show(select("#", coroutine.resume(bulk, table.unpack({}, 1, 200))))
show(coroutine.resume(bulk, table.unpack({}, 1, 150)))
return "done"
