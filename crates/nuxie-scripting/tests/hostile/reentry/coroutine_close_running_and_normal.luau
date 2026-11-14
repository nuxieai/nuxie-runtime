--! attacks: coroutine.close of the running coroutine, of a normal (resuming) coroutine, of a coroutine suspended inside pcall, and resuming non-suspended coroutines; expect: ok; within: 10s
local results = {}
-- close the running coroutine from inside itself
local co = coroutine.create(function()
  local ok, err = pcall(coroutine.close, coroutine.running())
  results.selfClose = { ok, err }
  coroutine.yield()
  return "done"
end)
assert(coroutine.resume(co))
assert(results.selfClose[1] == false, "closing the running coroutine must fail")
-- close a normal coroutine (it resumed the current one)
local outer
outer = coroutine.create(function()
  local inner = coroutine.create(function()
    assert(coroutine.status(outer) == "normal")
    results.closeNormal = { pcall(coroutine.close, outer) }
    results.resumeNormal = { pcall(coroutine.resume, outer) }
    results.resumeSelf = { coroutine.resume(coroutine.running()) }
  end)
  assert(coroutine.resume(inner))
  coroutine.yield("outer-yield")
  return "outer-done"
end)
local ok, v = coroutine.resume(outer)
assert(ok and v == "outer-yield", tostring(v))
assert(results.closeNormal[1] == false, "closing a normal coroutine must fail")
assert(results.resumeSelf[1] == false)
-- close a coroutine suspended inside a pcall and inside a xpcall handler chain
local insidePcall = coroutine.create(function()
  pcall(function()
    xpcall(function() coroutine.yield("deep") end, function(e) return e end)
  end)
end)
assert(coroutine.resume(insidePcall))
local closed = { coroutine.close(insidePcall) }
assert(coroutine.status(insidePcall) == "dead")
assert(not coroutine.resume(insidePcall))
-- close a coroutine that is dead with an error object having __tostring
local errored = coroutine.create(function() error(setmetatable({}, { __tostring = function() return "boxed" end })) end)
coroutine.resume(errored)
local c1, c2 = coroutine.close(errored)
assert(c1 == false)
-- wrap: resume a wrapped coroutine from inside itself
local wrapped
wrapped = coroutine.wrap(function()
  local okSelf = pcall(wrapped)
  assert(not okSelf, "a wrapped coroutine cannot resume itself")
  coroutine.yield(1)
end)
wrapped()
-- close a coroutine from inside a __gc-free finaliser path: from a sort comparator
local victim = coroutine.create(function() coroutine.yield() end)
coroutine.resume(victim)
local t = { 3, 1, 2 }
table.sort(t, function(a, b) coroutine.close(victim); return a < b end)
assert(coroutine.status(victim) == "dead")
-- finish the first coroutine
assert(coroutine.resume(co))
return {}
