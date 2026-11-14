--! attacks: await() from inside table.sort comparators, gsub callbacks, metamethods and gradient stop __index run by host calls inside async(); expect: ok; within: 10s
local pendingResolve
local pending = Promise.new(function(resolve) pendingResolve = resolve end)
local outcomes = {}
local function record(name, promise)
  promise:andThen(function(v) outcomes[name] = "ok" end, function(e) outcomes[name] = "rejected" end)
end
record("sort", async(function()
  local t = { 3, 2, 1 }
  table.sort(t, function(a, b) await(pending); return a < b end)
end))
record("gsub", async(function()
  return (string.gsub("aaa", "a", function() await(pending); return "b" end))
end))
record("index", async(function()
  local proxy = setmetatable({}, { __index = function() await(pending); return 1 end })
  return proxy.k
end))
record("gradient", async(function()
  return Gradient.linear(Vector.xy(0, 0), Vector.xy(1, 0), {
    setmetatable({}, { __index = function(_, k) await(pending); return k == "position" and 0 or 0xff000000 end }),
  })
end))
record("pcall", async(function()
  local ok, a, b = pcall(await, pending)
  return a
end))
-- An async body that awaits its own result promise, then cancelled from outside.
local selfAwait
selfAwait = async(function()
  await(Promise.resolve(0))
  await(selfAwait)
end)
-- An async body that resolves the promise other bodies await, from inside itself.
record("resolver", async(function()
  await(Promise.resolve(0))
  pendingResolve("released")
  return "resolver"
end))
selfAwait:cancel()
assert(outcomes.sort == "rejected", tostring(outcomes.sort))
assert(outcomes.gsub == "rejected", tostring(outcomes.gsub))
assert(outcomes.pcall == "ok", tostring(outcomes.pcall))
return {}
