--! attacks: coroutine.yield from inside table.sort comparators, gsub callbacks and __index/__lt/__tostring run by library calls; expect: ok; within: 10s
local function run(f)
  local co = coroutine.create(f)
  local ok, err = coroutine.resume(co)
  -- Either the yield is refused (an error), or it suspends; resume again until dead.
  local guard = 0
  while ok and coroutine.status(co) == "suspended" and guard < 100 do
    guard += 1
    ok, err = coroutine.resume(co)
  end
  return ok, err, coroutine.status(co)
end
local results = {}
results.sort = { run(function()
  local t = { 5, 3, 1, 4, 2 }
  table.sort(t, function(a, b) coroutine.yield(); return a < b end)
end) }
results.gsub = { run(function()
  return (string.gsub("abcabc", "b", function(c) coroutine.yield(c); return "B" end))
end) }
results.index = { run(function()
  local proxy = setmetatable({}, { __index = function(_, k) coroutine.yield(k); return 1 end })
  return rawget(proxy, "x"), proxy.x
end) }
results.lt = { run(function()
  local mt = { __lt = function(a, b) coroutine.yield(); return a.v < b.v end }
  local t = {}
  for i = 1, 10 do t[i] = setmetatable({ v = 11 - i }, mt) end
  table.sort(t)
end) }
results.tostring = { run(function()
  return tostring(setmetatable({}, { __tostring = function() coroutine.yield(); return "s" end }))
end) }
results.concat = { run(function()
  return table.concat({ "a", setmetatable({}, { __tostring = function() coroutine.yield() end }) })
end) }
results.print = { run(function()
  print(setmetatable({}, { __tostring = function() coroutine.yield(); return "p" end }))
end) }
results.pcall = { run(function()
  -- yield inside a pcall: allowed in Luau, the pcall must still catch a later error.
  local ok = pcall(function() coroutine.yield(); error("after yield") end)
  assert(not ok)
end) }
assert(results.sort[1] == false, "yield across table.sort must be refused")
assert(results.gsub[1] == false, "yield across gsub must be refused")
assert(results.pcall[1] == true, tostring(results.pcall[2]))
for name, r in pairs(results) do
  assert(r[3] == "dead" or r[3] == "suspended", name .. " left " .. tostring(r[3]))
end
return {}
