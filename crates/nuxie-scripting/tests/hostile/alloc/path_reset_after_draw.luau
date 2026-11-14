--! attacks: Path:reset after a draw built its render path, then regrow, many paths kept; expect: ok; within: 30s; harness: instance
-- E2b "Not verified": reset keeping the charge once a render path exists.
return function()
  return {
    paths = {},
    draw = function(self, renderer)
      local paint = Paint.new()
      for i = 1, 40 do
        local p = Path.new()
        p:moveTo(Vector.xy(0, 0))
        for k = 1, 64 do p:lineTo(Vector.xy(k, k)) end
        local ok, err = pcall(function()
          for _ = 1, 14 do p:add(p) end -- about 1M commands
          renderer:drawPath(p, paint)
          p:reset()
          for _ = 1, 64 do p:lineTo(Vector.xy(1, 2)) end
          for _ = 1, 14 do p:add(p) end
          renderer:drawPath(p, paint)
        end)
        if not ok then
          assert(string.find(tostring(err), "host memory budget", 1, true), tostring(err))
          return
        end
        table.insert(self.paths, p)
      end
      error("40 drawn ~1M-command paths never hit the host memory budget")
    end,
  }
end
