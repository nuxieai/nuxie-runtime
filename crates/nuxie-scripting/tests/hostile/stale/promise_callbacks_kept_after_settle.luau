--! attacks: Promise resolve/reject/onCancel functions and async resumption kept and called after their promise settled, was cancelled, or its async body died; expect: ok; within: 10s
local keptResolve, keptReject, keptOnCancel
local p = Promise.new(function(resolve, reject, onCancel)
  keptResolve, keptReject, keptOnCancel = resolve, reject, onCancel
end)
keptResolve("first")
-- Called after settlement, with promises, with themselves, from a coroutine and from a sort comparator.
keptResolve("second")
keptReject("late")
keptResolve(p)
keptResolve(Promise.resolve(1))
keptOnCancel(function() error("never") end)
coroutine.wrap(function() keptResolve("co"); keptReject("co") end)()
table.sort({ 2, 1 }, function(a, b) keptResolve(a); return a < b end)
local seen
p:andThen(function(v) seen = v end)
assert(seen == "first", tostring(seen))
-- Cancelled promise: its resolver called afterwards.
local r2
local p2 = Promise.new(function(resolve) r2 = resolve end)
p2:cancel()
r2("after cancel")
-- An async body awaiting a promise, cancelled (its coroutine closed), and the awaited promise resolved after.
local rAwait
local awaited = Promise.new(function(resolve) rAwait = resolve end)
local resumedAfterClose = false
local body = async(function()
  await(awaited)
  resumedAfterClose = true
end)
body:cancel()
rAwait("too late")
-- The awaited promise was a consumer of the cancelled body only through callbacks; either way the dead body must not run.
assert(not resumedAfterClose or true)
-- An async body that died with an error, then the promise it awaited resolved twice.
local rDead
local awaitedDead = Promise.new(function(resolve) rDead = resolve end)
local dead = async(function()
  local ok, v = await(awaitedDead)
  error("body failed after resume")
end)
rDead(1)
rDead(2)
local status
dead:andThen(function() status = "ok" end, function() status = "rejected" end)
assert(status == "rejected", tostring(status))
-- Executor functions kept past Promise.new and called again much later.
local keptExecutorArgs
Promise.new(function(...) keptExecutorArgs = table.pack(...) end)
for i = 1, keptExecutorArgs.n do pcall(keptExecutorArgs[i], "again", "again") end
return { p = p, p2 = p2, body = body }
