--! attacks: deeply nested pcall: recursive pcall of self and one pcall(pcall, pcall, ...) 7,000-long argument chain; expect: ok; within: 20s
local function nopanic(e) assert(not string.find(tostring(e), "rust panic", 1, true), tostring(e)) end
-- each level a fresh protected host entry
local reached = 0
local function nest(n)
  reached = n
  local ok, e = pcall(nest, n + 1)
  if not ok then nopanic(e) end
  return ok, e
end
local ok, e = pcall(nest, 1)
nopanic(e)
assert(reached > 10, "nesting stopped at " .. reached)
-- pcall(pcall, pcall, ..., pcall, error, "x"): one call, 7,000 nested pcall frames
local args = table.create(7000, pcall)
table.insert(args, error)
table.insert(args, "bottom")
local results = table.pack(pcall(table.unpack(args)))
for i = 1, results.n do nopanic(results[i]) end
-- xpcall nesting the same way
local xreached = 0
local function xnest(n)
  xreached = n
  return xpcall(xnest, function(err) return err end, n + 1)
end
local okx, ex = pcall(xnest, 1)
nopanic(ex)
assert(xreached > 10)
return {}
