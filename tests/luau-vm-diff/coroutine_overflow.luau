-- A coroutine whose results do not fit its resumer (lane 4 review
-- finding): the resume raises "too many results to resume", the results
-- stay on the coroutine, and it reports suspended (C++ lua_costatus). The
-- next resume calls the last of those results. The values come from Lua
-- recursion, not table.unpack, whose own limit is lane 2's.
local function show(...) print(...) end

local big = {}
for i = 1, 7998 do big[i] = i end
big[7999] = function(...) return "called", select("#", ...) end
local function cat(t, i, n)
  if i > n then return end
  return t[i], cat(t, i + 1, n)
end
local co = coroutine.create(function() return cat(big, 1, 7999) end)
show("overflow", pcall(coroutine.resume, co))
show("status", coroutine.status(co))
show("again", coroutine.resume(co, "x", "y"))
show("status", coroutine.status(co))
show("dead", coroutine.resume(co))

-- A yield that does not fit leaves the coroutine suspended at the yield.
local yco = coroutine.create(function()
  local got = coroutine.yield(cat(big, 1, 7999))
  return "resumed", got
end)
show("yield overflow", pcall(coroutine.resume, yco))
show("status", coroutine.status(yco))
show("yield again", coroutine.resume(yco, "z"))
return "done"
