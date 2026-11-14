--! attacks: Audio.start with unregistered names and hostile option tables (no processor module can be registered from a script); expect: ok; within: 10s; harness: instance
local function refused(...)
  local ok, err = pcall(Audio.start, ...)
  assert(not ok, "Audio.start accepted")
  return tostring(err)
end
return function()
  return {
    init = function(self)
      local deep = {}
      for i = 1, 1000 do deep = {deep} end
      local cyc = {}
      cyc.self = cyc
      refused()
      refused(1)
      refused(nil, {})
      refused("x", {}, 3)
      local e = refused("tone", {})
      assert(string.find(e, "not registered"), e)
      refused("tone", deep)
      refused("tone", cyc)
      refused(string.rep("n", 1024 * 1024), {})
      refused("\255", {})
      refused("tone", setmetatable({}, {__index = function() error("x") end}))
      for i = 1, 200 do refused("p" .. i, {i}) end
    end,
  }
end
