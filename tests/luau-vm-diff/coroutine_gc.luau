-- The collector and parked coroutines (lane 4 review findings).
local function show(...) print(...) end

-- An unreachable suspended coroutine is collected even though a closure
-- it exported still captures one of its locals; the closure keeps the
-- value (the upvalue is closed when the thread dies), and stores through
-- it after the coroutine is gone stay visible.
local weak = setmetatable({}, {__mode = "v"})
local get, set
do
    local co = coroutine.create(function()
        local x = {tag = "captured"}
        get = function() return x end
        set = function(v) x = v end
        coroutine.yield()
    end)
    coroutine.resume(co)
    weak[1] = co
end
collectgarbage()
show("thread collected", weak[1] == nil, get().tag)
set({tag = "stored"})
collectgarbage()
show("after store", get().tag)

-- A store through an open upvalue into a parked, unreachable coroutine's
-- stack between collections is kept (C++ remarkupvals).
local get2, set2
do
    local co = coroutine.create(function()
        local y = {tag = "first"}
        get2 = function() return y end
        set2 = function(v) y = v end
        coroutine.yield()
    end)
    coroutine.resume(co)
end
set2({tag = "second"})
collectgarbage()
show("remarked", get2().tag)

-- Dead registers above a parked coroutine's live top do not root reused
-- slots once it resumes: the tables a block dropped, and the tables made
-- after a collection reused their slots, are collected.
local dropped = setmetatable({}, {__mode = "v"})
local co = coroutine.create(function()
    do
        local a, b, c, d = {}, {}, {}, {}
        dropped[1], dropped[2], dropped[3], dropped[4] = a, b, c, d
    end
    coroutine.yield()
    collectgarbage()
    local n = 0
    for _ in pairs(dropped) do n = n + 1 end
    return n
end)
coroutine.resume(co)
collectgarbage()
for i = 1, 4 do dropped[i] = {} end
show("dead slots", coroutine.resume(co))
return "done"
