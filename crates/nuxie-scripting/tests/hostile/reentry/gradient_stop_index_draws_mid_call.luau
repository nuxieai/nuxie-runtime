--! attacks: renderer calls made from a gradient stop __index while Gradient.linear runs inside draw, and a nested Text/Path draw from that __index; expect: ok; within: 10s; harness: instance
return function()
  return {
    draw = function(self, renderer)
      local path = Path.new()
      path:moveTo(Vector.xy(0, 0))
      path:lineTo(Vector.xy(10, 10))
      local paint = Paint.new()
      local reentered = 0
      local stops = {
        setmetatable({}, { __index = function(_, k)
          reentered += 1
          renderer:save()
          renderer:drawPath(path, paint)
          renderer:clipPath(path)
          local okInner = pcall(function()
            Gradient.linear(Vector.xy(0, 0), Vector.xy(1, 0), {
              setmetatable({}, { __index = function(_, kk)
                renderer:drawPath(path, paint)
                renderer:restore()
                renderer:save()
                return kk == "position" and 0 or 0xff0000ff
              end }),
            })
          end)
          renderer:restore()
          return k == "position" and 0 or 0xffff0000
        end }),
      }
      local g = Gradient.linear(Vector.xy(0, 0), Vector.xy(10, 0), stops)
      paint.gradient = g
      renderer:drawPath(path, paint)
      assert(reentered == 2, reentered)
      -- Over-restore after the re-entrant calls must be an ordinary error.
      local ok = true
      for _ = 1, 10 do ok = ok and pcall(renderer.restore, renderer) end
      assert(not ok, "unbalanced restore must fail")
    end,
  }
end
