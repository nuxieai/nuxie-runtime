--! attacks: long and cyclic __index table chains; expect: ok; within: 15s
local function refused(f, ...)
  local ok, err = pcall(f, ...)
  assert(not ok, "accepted")
  err = tostring(err)
  assert(not string.find(err, "rust panic", 1, true), err)
  return err
end
-- a self loop
local t = {}
setmetatable(t, { __index = t })
local err = refused(function() return t.missing end)
assert(string.find(err, "loop", 1, true) or string.find(err, "chain", 1, true), err)
-- a two-table cycle
local a, b = {}, {}
setmetatable(a, { __index = b }) setmetatable(b, { __index = a })
refused(function() return a.missing end)
-- a 30,000-long acyclic chain
local head = {}
local cur = head
for i = 1, 30000 do local nxt = {} setmetatable(cur, { __index = nxt }) cur = nxt end
cur.found = true
local ok, v = pcall(function() return head.found end)
assert(not string.find(tostring(v), "rust panic", 1, true), tostring(v))
assert(ok == false or v == true, tostring(v))
-- __newindex table chain cycle
local c = {}
setmetatable(c, { __newindex = c })
local okc, ec = pcall(function() c.x = 1 end)
assert(okc or not string.find(tostring(ec), "rust panic", 1, true), tostring(ec))
return {}
