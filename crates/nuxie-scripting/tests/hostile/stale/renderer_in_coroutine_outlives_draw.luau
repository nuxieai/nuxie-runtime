--! attacks: renderer methods called from coroutines (created in draw) resumed during draw, after draw returned, and in a later frame's draw; expect: ok; within: 10s; harness: instance
return function()
  return {
    draw = function(self, renderer)
      local path = Path.new()
      path:moveTo(Vector.xy(0, 0))
      path:lineTo(Vector.xy(3, 3))
      local paint = Paint.new()
      local co = coroutine.create(function()
        while true do
          renderer:save()
          renderer:drawPath(path, paint)
          renderer:restore()
          coroutine.yield("drew")
        end
      end)
      -- Resumed inside the draw that lent the renderer: must work.
      local ok, v = coroutine.resume(co)
      assert(ok and v == "drew", tostring(v))
      -- A coroutine.wrap and a pcall'd resume inside draw.
      local w = coroutine.wrap(function() renderer:save(); coroutine.yield(); renderer:restore() end)
      w(); w()
      assert(pcall(coroutine.resume, co))
      if self.oldCo then
        -- A coroutine made in the previous frame's draw, holding the previous renderer, resumed now.
        local okOld, errOld = coroutine.resume(self.oldCo)
        assert(not okOld and string.find(tostring(errOld), "no longer valid", 1, true), tostring(errOld))
        -- A coroutine made in this draw that uses the previous renderer.
        local prev = self.prevRenderer
        local okPrev, errPrev = coroutine.resume(coroutine.create(function() prev:save() end))
        assert(not okPrev, "previous frame's renderer accepted a call")
      end
      self.oldCo = co
      self.prevRenderer = renderer
      -- A coroutine suspended in the middle of a renderer call sequence, left behind.
      self.half = coroutine.create(function() renderer:save(); coroutine.yield(); renderer:restore() end)
      coroutine.resume(self.half)
    end,
    advance = function(self, dt)
      local ok, err = coroutine.resume(self.oldCo)
      assert(not ok and string.find(tostring(err), "no longer valid", 1, true), tostring(err))
      local okHalf, errHalf = coroutine.resume(self.half)
      assert(not okHalf, "a half-finished renderer coroutine resumed outside draw must fail")
      -- The dead coroutine is dead.
      assert(not coroutine.resume(self.oldCo))
      self.oldCo = coroutine.create(function() self.prevRenderer:save() end)
      return true
    end,
  }
end
