--! attacks: Renderer drawPath/clipPath/transform/modulate with NaN/inf geometry, matrices and values; expect: ok; within: 20s; harness: instance
local nan, inf = 0 / 0, math.huge
return function()
  return {
    draw = function(self, renderer)
      local p = Path.new()
      p:moveTo(Vector.xy(nan, inf))
      p:cubicTo(Vector.xy(-inf, nan), Vector.xy(3e38, -3e38), Vector.xy(nan, nan))
      p:close()
      local paint = Paint.new()
      paint.thickness = nan
      paint.feather = inf
      renderer:save()
      renderer:transform(Mat2D.values(nan, inf, -inf, nan, inf, nan))
      renderer:clipPath(p)
      renderer:drawPath(p, paint)
      renderer:modulateOpacity(nan)
      renderer:modulateOpacity(-inf)
      renderer:transform(Mat2D.values(0, 0, 0, 0, 0, 0))
      renderer:drawPath(Path.new(), paint)
      renderer:restore()
      local function safe(f)
        local ok, err = pcall(f)
        assert(ok or not string.find(tostring(err), "rust panic", 1, true), tostring(err))
        return ok
      end
      safe(function() renderer:modulateColor(-1) end)
      safe(function() renderer:modulateColor(nan) end)
      safe(function() renderer:setColorModulation(2 ^ 40) end)
      assert(not safe(function() renderer:transform(Vector.xy(1, 1)) end))
      assert(not safe(function() renderer:drawPath(paint, p) end))
      assert(not safe(function() renderer:clipPath(nil) end))
    end,
  }
end
