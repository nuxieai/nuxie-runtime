--! attacks: host frames that forward the draw's renderer loan (pcall, xpcall guard, resume, print, Gradient.linear) captured in draw and called in advance and the next draw with callbacks that use the renderer; expect: ok; within: 10s; harness: instance
local function hostFrames(into)
  for level = 1, 6 do
    local f = debug.info(level, "f")
    if f and debug.info(f, "s") == "[C]" then table.insert(into, f) end
  end
end
return function()
  return {
    draw = function(self, renderer)
      local path = Path.new()
      path:moveTo(Vector.xy(0, 0)); path:lineTo(Vector.xy(2, 2))
      local paint = Paint.new()
      local use = function() renderer:save(); renderer:drawPath(path, paint); renderer:restore(); return "drew" end
      if self.frames then
        -- Last frame's guards, called inside this draw with a callback using last frame's renderer.
        local old = self.oldUse
        for _, f in ipairs(self.frames) do pcall(f, old) end
        -- And with this frame's callback: whatever the guard does, it must not crash.
        for _, f in ipairs(self.frames) do pcall(f, use) end
      end
      self.frames = {}
      pcall(function() hostFrames(self.frames); use() end)
      xpcall(function() error("x", 0) end, function(e) hostFrames(self.frames); use(); return e end)
      coroutine.resume(coroutine.create(function() hostFrames(self.frames); use() end))
      print(setmetatable({}, { __tostring = function() hostFrames(self.frames); use(); return "p" end }))
      Gradient.linear(Vector.xy(0, 0), Vector.xy(1, 0), { setmetatable({}, { __index = function(_, k) hostFrames(self.frames); use(); return k == "position" and 0 or 1 end }) })
      assert(#self.frames >= 4, #self.frames)
      self.oldUse = use
    end,
    advance = function(self, dt)
      local failures = 0
      for _, f in ipairs(self.frames) do
        local ok = pcall(f, self.oldUse)
        if not ok then failures += 1 end
      end
      -- Calling the stale-renderer callback through the kept guards must never succeed in drawing.
      local ok, err = pcall(self.oldUse)
      assert(not ok and string.find(tostring(err), "no longer valid", 1, true), tostring(err))
      -- A promise callback resolved now that draws with the stale renderer.
      local p = Promise.resolve(1):andThen(function() return self.oldUse() end)
      local status
      p:andThen(function() status = "ok" end, function() status = "rejected" end)
      assert(status == "rejected", tostring(status))
      return true
    end,
  }
end
