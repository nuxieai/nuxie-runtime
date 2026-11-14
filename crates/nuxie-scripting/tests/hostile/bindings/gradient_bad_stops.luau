--! attacks: Gradient.linear/radial with NaN/unsorted/duplicate stop positions, NaN/inf/negative radius and centers, malformed stop tables; expect: ok; within: 20s; harness: instance
local function safe(f, ...)
  local ok, err = pcall(f, ...)
  assert(ok or not string.find(tostring(err), "rust panic", 1, true), tostring(err))
  return ok, err
end
local nan, inf = 0 / 0, math.huge
local red = Color.rgba(255, 0, 0, 255)
return function()
  return {
    draw = function(self, renderer)
      local stopSets = {
        {},
        {{position = nan, color = red}},
        {{position = 1, color = red}, {position = 0, color = red}, {position = nan, color = red}, {position = inf, color = red}, {position = -inf, color = red}},
        {{position = 0.5, color = red}, {position = 0.5, color = red}},
        {{position = "x", color = red}},
        {{position = 0}},
        {{color = red}},
        {5, "x", {position = 0, color = red}},
        table.create(50000, {position = 0.25, color = red}),
      }
      local p = Path.new()
      p:moveTo(Vector.xy(0, 0)) p:lineTo(Vector.xy(100, 0)) p:lineTo(Vector.xy(0, 100)) p:close()
      for _, stops in ipairs(stopSets) do
        for _, make in ipairs({
          function() return Gradient.linear(Vector.xy(nan, inf), Vector.xy(-inf, nan), stops) end,
          function() return Gradient.linear(Vector.xy(0, 0), Vector.xy(0, 0), stops) end,
          function() return Gradient.radial(Vector.xy(0, 0), nan, stops) end,
          function() return Gradient.radial(Vector.xy(nan, 0), inf, stops) end,
          function() return Gradient.radial(Vector.xy(0, 0), -1e38, stops) end,
          function() return Gradient.radial(Vector.xy(0, 0), 0, stops) end,
        }) do
          local ok, g = safe(make)
          if ok then
            safe(function()
              local paint = Paint.new()
              paint.gradient = g
              paint.gradientTransform = Mat2D.values(nan, 0, 0, inf, 0, 0)
              renderer:drawPath(p, paint)
              renderer:drawPath(p, Paint.with({gradient = g}))
            end)
          end
        end
      end
      safe(function() return Gradient.linear(Vector.xy(0, 0), Vector.xy(1, 0)) end)
      safe(function() return Gradient.radial(Vector.xy(0, 0), "big", {}) end)
      safe(function() return Gradient.linear(1, 2, 3) end)
    end,
  }
end
