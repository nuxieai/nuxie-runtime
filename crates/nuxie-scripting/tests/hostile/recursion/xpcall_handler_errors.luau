--! attacks: xpcall handlers that error, error with an object, and are captured through debug.info and called after xpcall returned; expect: ok; within: 15s
local function nopanic(e) assert(not string.find(tostring(e), "rust panic", 1, true), tostring(e)) end
local ok, e = xpcall(function() error("body") end, function(err) error("handler") end)
assert(not ok) nopanic(e)
ok, e = xpcall(function() error("body") end, function(err) error({ tag = "obj" }) end)
assert(not ok) nopanic(e)
ok, e = xpcall(function() error({}) end, function(err) return err, err, err end)
assert(not ok) nopanic(e)
-- non-function handlers
ok, e = pcall(xpcall, function() error("body") end, setmetatable({}, { __call = function() return "called" end }))
nopanic(e)
ok, e = pcall(xpcall, function() error("body") end, nil)
nopanic(e)
-- the guarded handler, kept past its xpcall
local known = { [pcall] = true, [xpcall] = true, [error] = true, [debug.info] = true }
local captured = {}
xpcall(function() error("x") end, function(err)
  for level = 1, 8 do
    local okf, f = pcall(debug.info, level, "f")
    if okf and type(f) == "function" and not known[f] and debug.info(f, "s") == "[C]" then table.insert(captured, f) end
  end
  return err
end)
assert(#captured > 0, "no host frame captured")
for _, f in ipairs(captured) do
  for _, arg in ipairs({ "probe", {}, f }) do
    local okc, ec = pcall(f, arg)
    nopanic(ec)
  end
  -- called from inside another xpcall's handler
  xpcall(function() error("y") end, function(err) local okc, ec = pcall(f, err) nopanic(ec) return err end)
end
return {}
