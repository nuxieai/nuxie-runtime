--! attacks: drawImage/drawImageMesh/drawImageMeshInstanced with wrong types, bad blend modes and mismatched buffers; expect: ok; within: 10s; harness: instance
return function()
  return {
    draw = function(self, renderer)
      local sampler = ImageSampler("clamp", "repeat", "nearest")
      local vb, uv, tb = VertexBuffer(), VertexBuffer(), TriangleBuffer()
      vb:add(Vector.xy(0, 0), Vector.xy(1, 0), Vector.xy(0, 1))
      uv:add(Vector.xy(0, 0))
      tb:add(0, 1, 2)
      local mi = MeshInstances(2)
      local p = Path.new()
      local calls = {
        function() renderer:drawImage(p, sampler, "srcOver", 1) end,
        function() renderer:drawImage(nil, sampler, "srcOver", 0 / 0) end,
        function() renderer:drawImage(p, sampler, "bogus", 1) end,
        function() renderer:drawImage(p, p, "srcOver", 1) end,
        function() renderer:drawImageMesh(p, sampler, vb, uv, tb, "srcOver", 1) end,
        function() renderer:drawImageMesh(p, sampler, vb, vb, tb, "nope", math.huge) end,
        function() renderer:drawImageMesh(p, sampler, tb, uv, vb, "srcOver", 1) end,
        function() renderer:drawImageMeshInstanced(p, sampler, vb, uv, tb, mi) end,
        function() renderer:drawImageMeshInstanced(p, sampler, vb, vb, tb, p) end,
        function() renderer:drawImageMeshInstanced() end,
        function() ImageSampler("clamp", "clamp", "cubic") end,
        function() ImageSampler(1, 2, 3) end,
      }
      for i, f in ipairs(calls) do
        local ok, err = pcall(f)
        assert(not ok, "call " .. i .. " was accepted")
        assert(not string.find(tostring(err), "rust panic", 1, true), tostring(err))
      end
    end,
  }
end
