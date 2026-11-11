-- Per-thread globals (lane 4 review finding). Kept out of coroutines.luau:
-- that file with this case appended crashes the vendored luaur compiler
-- (EXC_BAD_ACCESS in DenseHashTable::find), a compiler bug outside this VM.
local function show(...) print(...) end

-- setfenv(0, t) changes the running thread's globals only (C++ L->gt);
-- a new coroutine starts with its creator's.
do
  local outer = getfenv(0)
  local env = setmetatable({ tag = "child" }, { __index = outer })
  local co = coroutine.create(function()
    setfenv(0, env)
    coroutine.yield(getfenv(0).tag)
    local inner = coroutine.wrap(function() return getfenv(0).tag end)
    return inner()
  end)
  show("fenv", coroutine.resume(co))
  show("fenv outer", getfenv(0) == outer, rawget(getfenv(0), "tag"))
  show("fenv inner", coroutine.resume(co))
  show("fenv after", getfenv(0) == outer)
end
return "done"
