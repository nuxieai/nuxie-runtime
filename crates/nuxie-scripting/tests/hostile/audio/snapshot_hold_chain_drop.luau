--! attacks: snapshot publish hold chain of ~65,536 links dropped at collection (E1 4.4); expect: ok; within: 90s; harness: instance
-- Each snapshot publishes the previous one, so dropping the head drops the
-- whole chain. The chain is built across several callbacks (each callback has
-- its own safepoint allowance), and dropped when the VM is collected.
local head = nil
local links = 0
local refusal = nil
local function grow(n)
  for i = 1, n do
    local ok, s = pcall(Audio.Snapshot.create, 16)
    if not ok then refusal = s return end
    if head ~= nil then s:publish(head) end
    head = s
    links += 1
  end
end
return function()
  return {
    init = function(self) grow(12000) end,
    draw = function(self, renderer) grow(12000) end,
    advance = function(self, dt) grow(12000) return true end,
    update = function(self)
      grow(12000)
    end,
  }
end
