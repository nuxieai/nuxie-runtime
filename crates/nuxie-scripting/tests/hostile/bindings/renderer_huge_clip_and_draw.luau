--! attacks: clipPath and drawPath of a ~2M-command path, and a path redrawn after growing; expect: ok; within: 30s; harness: instance
return function()
  return {
    init = function(self)
      local p = Path.new()
      p:moveTo(Vector.xy(0, 0))
      for i = 1, 64 do p:cubicTo(Vector.xy(i, -i), Vector.xy(-i, i), Vector.xy(i, i)) end
      local ok, err = pcall(function() for _ = 1, 15 do p:add(p) end end)
      if not ok then assert(string.find(tostring(err), "host memory budget", 1, true), tostring(err)) end
      self.p = p
    end,
    draw = function(self, renderer)
      local paint = Paint.new()
      paint.style = "stroke"
      paint.thickness = 1e30
      for _ = 1, 50 do renderer:clipPath(self.p) end
      renderer:drawPath(self.p, paint)
      local ok, err = pcall(function() self.p:add(self.p) end)
      if not ok then assert(string.find(tostring(err), "host memory budget", 1, true), tostring(err)) end
      renderer:drawPath(self.p, paint)
    end,
  }
end
