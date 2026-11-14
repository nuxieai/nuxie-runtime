--! attacks: a renderer stashed during draw and used in advance, update, and the next frame's draw after the first draw returned; expect: ok; within: 10s; harness: instance
local function mustBeInvalid(renderer, where)
  local path = Path.new()
  path:moveTo(Vector.xy(0, 0))
  path:lineTo(Vector.xy(5, 5))
  local paint = Paint.new()
  local calls = {
    function() renderer:save() end,
    function() renderer:restore() end,
    function() renderer:drawPath(path, paint) end,
    function() renderer:clipPath(path) end,
    function() renderer:transform(Mat2D.identity and Mat2D.identity() or Mat2D.new()) end,
    function() renderer:modulateOpacity(0.5) end,
    function() renderer:modulateColor(0xff00ff00) end,
    function() renderer:setColorModulation(0xff00ff00) end,
  }
  for i, call in ipairs(calls) do
    local ok, err = pcall(call)
    assert(not ok, where .. ": stale renderer call " .. i .. " succeeded")
    assert(string.find(tostring(err), "no longer valid", 1, true) or string.find(tostring(err), "unbalanced", 1, true), where .. ": " .. tostring(err))
  end
end
return function()
  return {
    frames = 0,
    draw = function(self, renderer)
      self.frames += 1
      if self.kept then
        assert(self.kept ~= renderer or true)
        mustBeInvalid(self.kept, "second draw")
      end
      renderer:save()
      self.kept = renderer
      -- left saved on purpose: the frame ends with an unbalanced save
    end,
    advance = function(self, dt)
      mustBeInvalid(self.kept, "advance")
      return true
    end,
    update = function(self)
      mustBeInvalid(self.kept, "update")
    end,
  }
end
