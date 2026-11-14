--! attacks: Renderer:restore with no save, and more restores than saves; expect: ok; within: 10s; harness: instance
return function()
  return {
    draw = function(self, renderer)
      local ok, err = pcall(function() renderer:restore() end)
      assert(not ok and string.find(tostring(err), "unbalanced", 1, true), tostring(err))
      renderer:save()
      renderer:restore()
      for _ = 1, 1000 do
        ok, err = pcall(function() renderer:restore() end)
        assert(not ok, "restore past the saves was accepted")
      end
      renderer:save() renderer:save() -- left for the host
    end,
  }
end
