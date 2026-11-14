--! attacks: a renderer method function and its metatable kept past draw, called later with the stale renderer, a fresh renderer, and wrong self values; expect: ok; within: 10s; harness: instance
return function()
  return {
    draw = function(self, renderer)
      local save = renderer.save
      local restore = renderer.restore
      local drawPath = renderer.drawPath
      if self.save then
        -- Old method functions on the live renderer: methods are per type, so this may work; it must not crash.
        pcall(self.save, renderer)
        pcall(self.restore, renderer)
        -- Old renderer through new method functions.
        assert(not pcall(save, self.renderer))
        -- Wrong self types.
        assert(not pcall(save, Path.new()))
        assert(not pcall(save, {}))
        assert(not pcall(drawPath, renderer, renderer, renderer))
        assert(not pcall(drawPath, renderer, Paint.new(), Path.new()))
      end
      self.save, self.restore, self.renderer = save, restore, renderer
      self.mt = getmetatable(renderer)
    end,
    advance = function(self)
      assert(not pcall(self.save, self.renderer))
      if type(self.mt) == "table" then
        -- Tampering with the renderer metatable must be refused or harmless.
        pcall(function() self.mt.__index = function() return function() end end end)
        pcall(function() self.mt.__metatable = nil end)
      end
      return true
    end,
  }
end
