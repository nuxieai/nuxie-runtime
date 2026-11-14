--! attacks: Paint fields and Paint.with given NaN/inf/negative values, wrong types and bad enum names, then drawn; expect: ok; within: 10s; harness: instance
local function safe(f, ...)
  local ok, err = pcall(f, ...)
  assert(ok or not string.find(tostring(err), "rust panic", 1, true), tostring(err))
  return ok, err
end
local nan, inf = 0 / 0, math.huge
return function()
  return {
    draw = function(self, renderer)
      local p = Path.new()
      p:moveTo(Vector.xy(0, 0)) p:lineTo(Vector.xy(10, 10)) p:lineTo(Vector.xy(0, 10)) p:close()
      for _, v in ipairs({nan, inf, -inf, -1, 0, 1e38}) do
        local paint = Paint.new()
        safe(function() paint.thickness = v end)
        safe(function() paint.feather = v end)
        safe(function() paint.color = v end)
        safe(function() paint.style = "stroke" end)
        renderer:drawPath(p, paint)
        safe(function() renderer:drawPath(p, Paint.with({thickness = v, feather = v, style = "stroke", join = "round", cap = "square"})) end)
      end
      local paint = Paint.new()
      for _, field in ipairs({"style", "join", "cap", "blendMode"}) do
        safe(function() paint[field] = "bogus" end)
        safe(function() paint[field] = 7 end)
        safe(function() paint[field] = nil end)
      end
      safe(function() paint.gradient = Path.new() end)
      safe(function() paint.gradient = 5 end)
      safe(function() paint.gradientTransform = "x" end)
      safe(function() paint.thickness = "thick" end)
      safe(function() return Paint.with(5) end)
      safe(function() return Paint.with({thickness = "x", color = {}}) end)
      safe(function() return Paint.with({gradient = Paint.new()}) end)
      renderer:drawPath(p, paint)
      local copy = Paint.with(paint)
      safe(function() renderer:drawPath(p, copy) end)
    end,
  }
end
