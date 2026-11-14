--! attacks: Region views: escaped view after the lock, nested locking, erroring/yielding callbacks, hostile timeouts and offsets; expect: ok; within: 15s
local r = Audio.Region.create(64)
-- hostile timeouts
for _, t in ipairs({-1, 0 / 0, math.huge, 1e300, -math.huge}) do
  assert(not pcall(function() return r:tryExclusive(function() end, t) end), "timeout " .. tostring(t))
end
assert(not pcall(function() return r:tryExclusive(nil, 0) end))
-- escaped view is expired
local escaped
assert(r:tryExclusive(function(view) escaped = view end, 0) == true)
local ok, err = pcall(function() return escaped:getU32(0) end)
assert(not ok and string.find(tostring(err), "expired"), tostring(err))
assert(not pcall(function() escaped:write(0, buffer.create(4)) end))
-- nested lock (same region and another region) is refused, and the outer lock survives
local r2 = Audio.Region.create(64)
assert(r:tryExclusive(function(view)
  local okn, errn = pcall(function() return r:tryExclusive(function() end, 0) end)
  assert(not okn and string.find(tostring(errn), "nested"), tostring(errn))
  assert(not pcall(function() return r2:tryExclusive(function() end, 0) end))
  view:setU32(0, 0xDEADBEEF)
end, 0))
assert(r:atomicLoad(0) == 0xDEADBEEF)
-- offsets
assert(r:tryExclusive(function(view)
  for _, off in ipairs({61, 64, 2 ^ 31, 2 ^ 32, 2 ^ 53, -1, 0 / 0}) do
    assert(not pcall(function() return view:getU32(off) end), "getU32 " .. tostring(off))
    assert(not pcall(function() return view:setU32(off, 1) end), "setU32 " .. tostring(off))
  end
  assert(not pcall(function() return view:read(1, buffer.create(64)) end))
  assert(not pcall(function() return view:write(2 ^ 40, buffer.create(1)) end))
end, 0))
for _, off in ipairs({61, 64, 2 ^ 32, -1, 0 / 0, 1}) do
  assert(not pcall(function() return r:atomicAdd(off, 1) end), "atomicAdd " .. tostring(off))
end
-- erroring callback poisons, and the lock is released
assert(not pcall(function() return r:tryExclusive(function() error("boom") end, 0) end))
assert(r.poisoned == true)
pcall(function() return r:tryExclusive(function() end, 0) end)
-- yielding inside the callback
local co = coroutine.create(function()
  return r2:tryExclusive(function() coroutine.yield(1) end, 0)
end)
local okc = coroutine.resume(co)
-- either the yield is refused or the coroutine is suspended; then the lock must be usable
pcall(function() return r2:tryExclusive(function() end, 0) end)
co = nil
return {}
