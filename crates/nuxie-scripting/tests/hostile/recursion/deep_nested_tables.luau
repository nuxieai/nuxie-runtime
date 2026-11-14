--! attacks: ~200,000-deep nested tables plus deep metatable, closure and coroutine chains, fed to table.concat/clone/tostring/sort and then left for the final collection; expect: ok; harness: instance; within: 30s
local function nopanic(e) assert(not string.find(tostring(e), "rust panic", 1, true), tostring(e)) end
local function deepen(self, n)
  local cur = self.tail
  for i = 1, n do local nxt = {} cur[1] = nxt cur = nxt end
  self.tail = cur
end
return function()
  return {
    init = function(self)
      self.root = {}
      self.tail = self.root
      deepen(self, 40000)
    end,
    draw = function(self, renderer) deepen(self, 40000) end,
    advance = function(self, dt)
      if not self.metas then
        local m = {}
        for i = 1, 15000 do m = setmetatable({}, m) end
        local f = function() return 0 end
        for i = 1, 15000 do local g = f f = function() return g end end
        self.metas, self.fns = m, f
      else
        local co = coroutine.create(function() end)
        for i = 1, 8000 do local prev = co co = coroutine.create(function() return prev end) end
        self.co = co
      end
      return true
    end,
    update = function(self)
      deepen(self, 30000)
      local ok, e = pcall(table.concat, self.root)
      assert(not ok) nopanic(e)
      local c = table.clone(self.root)
      assert(c[1] == self.root[1])
      assert(type(tostring(self.root)) == "string")
      ok, e = pcall(table.sort, { self.root, self.root[1] })
      nopanic(e)
    end,
  }
end
