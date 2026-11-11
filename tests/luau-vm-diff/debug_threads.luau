-- debug.info and debug.traceback with a thread argument (C++ getthread):
-- levels count down the coroutine's own frames, the default traceback
-- level is 0 for another thread and 1 for the running one, and the
-- thread shifts every argument position by one.
local function all(...)
  local t = table.pack(...)
  for i = 1, t.n do t[i] = tostring(t[i]) end
  return "(" .. table.concat(t, ",") .. ")"
end

local function inner(x)
  coroutine.yield(x)
  return x
end
local function outer(x)
  local r = inner(x)
  coroutine.yield(r + 1)
  error("boom")
end

-- Fresh: no frames yet.
local co = coroutine.create(outer)
print(debug.traceback(co), all(debug.info(co, 0, "sl")), debug.traceback(co, "msg"))

-- Suspended inside inner, below the yield native.
coroutine.resume(co, 1)
print(debug.traceback(co))
print(debug.traceback(co, "hdr"))
print(debug.traceback(co, "hdr", 1))
print(debug.traceback(co, nil, 2))
print(debug.traceback(co, "deep", 9))
for level = 0, 4 do
  print(level, all(debug.info(co, level, "sln")))
end
print(debug.info(co, 1, "f") == inner, debug.info(co, 2, "f") == outer, debug.info(co, 0, "f") == coroutine.yield)
print(all(debug.info(co, 1, "a")), all(debug.info(co, 2, "a")))

-- Argument positions shift by one with a thread.
print(all(pcall(debug.info, co, -1, "s")))
print(all(pcall(debug.info, co, outer, "s")))
print(all(pcall(debug.info, co, "x", "s")))
print(all(pcall(debug.info, co, 1)))
print(all(pcall(debug.info, co, 1, "ss")))
print(all(pcall(debug.info, co, 1, "q")))
print(all(pcall(debug.traceback, co, "m", -1)))
print(all(pcall(debug.traceback, co, {})))

-- Second yield, then death by error: the dead thread keeps its frames.
coroutine.resume(co)
print(debug.traceback(co))
print(all(coroutine.resume(co)))
print(coroutine.status(co), debug.traceback(co))
print(all(debug.info(co, 0, "sln")), all(debug.info(co, 1, "sln")))

-- A coroutine that returned has no frames left.
local done = coroutine.create(function() return 1 end)
coroutine.resume(done)
print(coroutine.status(done), debug.traceback(done), all(debug.info(done, 0, "s")))

-- From inside a coroutine: the running thread passed explicitly (default
-- level 1), and the thread that resumed it (normal, inside resume).
local parent
local child = coroutine.create(function()
  print(debug.traceback(coroutine.running()))
  print(debug.traceback(coroutine.running(), "self", 0))
  print(all(debug.info(coroutine.running(), 0, "sn")), all(debug.info(coroutine.running(), 1, "sl")))
  print(coroutine.status(parent), debug.traceback(parent))
  print(all(debug.info(parent, 0, "sn")), all(debug.info(parent, 1, "sl")))
  print(debug.traceback())
end)
parent = coroutine.create(function()
  local ok, e = coroutine.resume(child)
  return ok, e
end)
print(all(coroutine.resume(parent)))

-- An xpcall handler in a coroutine sees the erroring frame's line.
local function fails() local t = nil; return t.x end
local trap = coroutine.wrap(function()
  return xpcall(fails, function(m) return debug.traceback(m) end)
end)
print(all(trap()))
print(all(xpcall(fails, function(m) return debug.traceback(m, 1) end)))
print(all(xpcall(fails, function(m) return debug.info(2, "sl") end)))
