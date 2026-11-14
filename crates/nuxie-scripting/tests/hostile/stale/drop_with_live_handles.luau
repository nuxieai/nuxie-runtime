--! attacks: VM teardown with a stale renderer, suspended coroutines holding renderer and text handles, pending promises and kept xpcall guards still reachable; expect: ok; within: 15s; harness: instance
return function()
  return {
    draw = function(self, renderer)
      self.kept = self.kept or {}
      local k = self.kept
      k[#k + 1] = renderer
      local path = Path.new()
      path:moveTo(Vector.xy(0, 0)); path:lineTo(Vector.xy(4, 4))
      k[#k + 1] = coroutine.create(function()
        renderer:save()
        coroutine.yield()
        renderer:drawPath(path, Paint.new())
      end)
      coroutine.resume(k[#k])
      xpcall(function() error("e", 0) end, function(e)
        k[#k + 1] = debug.info(2, "f")
        return e
      end)
      k[#k + 1] = Promise.new(function(resolve) k[#k + 1] = resolve end)
      k[#k + 1] = async(function() await(k[#k - 1]) ; renderer:save() end)
      local weak = setmetatable({ renderer, path }, { __mode = "v" })
      k[#k + 1] = weak
      -- A self-referencing cycle through a userdata key in a weak-key table.
      local wk = setmetatable({}, { __mode = "k" })
      wk[path] = { path, renderer, wk }
      k[#k + 1] = wk
    end,
    advance = function(self, dt)
      -- Resolve a pending promise whose async body then calls the stale renderer.
      for _, v in ipairs(self.kept) do
        if type(v) == "function" then pcall(v, "go") end
      end
      return true
    end,
  }
end
