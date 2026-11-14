--! attacks: E1 4.1 error chain: pcall(print, victim) whose __tostring re-raises the previous round's error, relayed until the budget stops it; expect: error(exceeded); within: 30s
-- starter: what the guarded xpcall handler gives when kept through debug.info and called late
local known = { [pcall] = true, [xpcall] = true, [error] = true, [debug.info] = true }
local captured = {}
xpcall(function() error("x") end, function(e)
  for level = 1, 8 do
    local ok, f = pcall(debug.info, level, "f")
    if ok and type(f) == "function" and not known[f] and debug.info(f, "s") == "[C]" then table.insert(captured, f) end
  end
  return e
end)
local obj = { starter = true }
for _, f in ipairs(captured) do
  local ok, err = pcall(f, "probe")
  if not ok then obj = err if typeof(err) == "userdata" then break end end
end
-- one relay of a table error object, then a long relay of whatever comes back
local victim = setmetatable({}, { __tostring = function() error(obj, 0) end })
for i = 1, 1000000 do
  local ok, e = pcall(print, victim)
  assert(not ok)
  assert(not string.find(tostring(e), "rust panic", 1, true), "rust panic")
  obj = e
end
error("relay was never stopped")
