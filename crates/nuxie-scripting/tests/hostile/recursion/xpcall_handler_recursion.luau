--! attacks: xpcall handlers that recurse without bound, call xpcall with themselves as handler, and xpcall nested 5,000 deep; expect: ok; within: 20s; harness: instance
local function nopanic(e) assert(not string.find(tostring(e), "rust panic", 1, true), tostring(e)) end
local function h(err) return h(err) .. "!" end
local function h2(err) local _, r = xpcall(function() error("inner") end, h2) return r end
local function deep(n)
  if n == 0 then error("bottom") end
  local okd, ed = xpcall(deep, function(err) return err end, n - 1)
  if not okd then error(ed, 0) end
end
return function()
  return {
    init = function(self)
      local ok, e = xpcall(function() error("body") end, h)
      assert(not ok) nopanic(e)
    end,
    draw = function(self, renderer)
      local ok, e = xpcall(function() error("body") end, h2)
      assert(not ok) nopanic(e)
    end,
    advance = function(self, dt)
      local ok, e = pcall(deep, 5000)
      assert(not ok) nopanic(e)
      return true
    end,
  }
end
