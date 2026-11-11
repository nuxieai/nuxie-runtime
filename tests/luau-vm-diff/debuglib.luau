-- debug.info and debug.traceback on the running thread. Levels past the
-- main chunk are not probed: the oracle harness runs it from a native
-- wrapper (`luaur-rt-exec-raw`) this harness does not have.
local function all(...)
  local t = table.pack(...)
  for i = 1, t.n do t[i] = tostring(t[i]) end
  return "(" .. table.concat(t, ",") .. ")"
end
local function qux(...)
  return all(pcall(debug.info, ...))
end
print(qux(), qux(1), qux(-1), qux(1, "?"), qux(1, "nn"), qux(1, "u"), qux({}, "n"), qux("x", "n"), qux(1, {}), qux(100, "?"))
function baz(...)
  return debug.info(...)
end
print(baz(0, "n"), baz(1, "n"), all(baz(2, "n")), all(baz(50, "n")), baz(0, "s"), baz(1, "s"), baz(0, "l"), baz(1, "l"))
print(baz(0, "f") == debug.info, baz(1, "f") == baz, all(baz(0, "a")), all(baz(1, "a")), baz(math.sqrt, "n"), baz(math.sqrt, "f") == math.sqrt, baz("1", "n"), baz(1.9, "n"))
local t = { foo = function() return 1 end }
print(baz(t.foo, "n"), all(baz(function() end, "n")), all(baz(baz, "slna")), all(pcall(baz, 1, "nlsf?")))
local function quuz(f)
  return all(debug.info(f, "a"))
end
print(quuz(math.cos), quuz(function() end), quuz(function(...) end), quuz(function(a, b) end), quuz(function(a, b, ...) end))
local function testlinedefined()
  local line = debug.info(1, "l")
  local linedefined = debug.info(testlinedefined, "l")
  return line, linedefined, debug.info(print or math.abs, "l")
end
print(testlinedefined())
print(all(baz(1, "")), all(baz(1, "s\0n")), all(pcall(baz, 1, "fff")))
-- traceback
function foo(...)
  return debug.traceback(...)
end
local function deep(n, ...)
  if n == 0 then return foo(...) end
  local r = deep(n - 1, ...)
  return r
end
print(foo())
print(foo("hello"))
print(foo("hello", 2))
print(foo("hello", 0))
print(foo("hello", 100))
print(foo(12, 1))
print(foo(nil, 1))
print(all(pcall(foo, {})), all(pcall(foo, "x", -1)), all(pcall(foo, "x", "y")))
print(foo("bad \xFF utf8"), foo("nul\0cut"))
print(deep(25, "deep"))
print(xpcall(function() error("boom") end, debug.traceback))
print(select(2, xpcall(function() local x = nil; return x.y end, debug.traceback)))
print(pcall(function() return debug.traceback("in pcall") end))
print(type(debug.traceback), type(debug.info), #debug.traceback())
