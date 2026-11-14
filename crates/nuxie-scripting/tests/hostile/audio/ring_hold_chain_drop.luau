--! attacks: ring send hold chain (each ring holds the previous inside an object inside an array), then drop; expect: ok; within: 40s; harness: instance
local head = nil
local function grow(n)
  for i = 1, n do
    local ok, r = pcall(Audio.Ring.create, 64)
    if not ok then return end
    if head ~= nil then r:send({{prev = head}}) end
    head = r
  end
end
return function()
  return {
    init = function(self)
      -- a ring that holds itself, and two snapshots holding each other
      local selfring = Audio.Ring.create(64)
      selfring:send(selfring)
      local s1, s2 = Audio.Snapshot.create(16), Audio.Snapshot.create(16)
      s1:publish(s2)
      s2:publish(s1)
      grow(8000)
    end,
    advance = function(self, dt) grow(8000) return true end,
    update = function(self)
      grow(8000)
    end,
  }
end
