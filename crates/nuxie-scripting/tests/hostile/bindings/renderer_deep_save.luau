--! attacks: Renderer:save 20,000 deep without restore each frame (host restores at the end); expect: ok; within: 30s; harness: instance
-- 1e6 is out of reach inside one callback's 50,000 safepoints (each save spends more than one); 20,000 per draw.
return function()
  return {
    draw = function(self, renderer)
      for _ = 1, 20000 do renderer:save() end
      local p = Path.new()
      p:moveTo(Vector.xy(0, 0)) p:lineTo(Vector.xy(10, 10))
      renderer:clipPath(p)
      renderer:drawPath(p, Paint.new())
    end,
  }
end
