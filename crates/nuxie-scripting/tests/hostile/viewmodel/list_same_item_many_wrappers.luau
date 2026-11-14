--! attacks: list push of one item through a fresh (soon collected) list wrapper each time, so a per-wrapper growth charge can be refunded while the list keeps growing; expect: ok; within: 30s; harness: instance; viewmodels: data_bind_blob_test.riv
local root, node = nil, nil
local pushed = 0
local function grow(n)
  for i = 1, n do
    local ok = pcall(function() root:getList("children"):push(node) end)
    if not ok then return end
    pushed += 1
  end
end
return function()
  return {
    init = function(self) root, node = Data.Main.new(), Data.XMLNodeVM.new() grow(3000) end,
    draw = function(self, renderer) grow(3000) end,
    advance = function(self, dt) grow(3000) return true end,
    update = function(self)
      grow(3000)
      assert(root:getList("children").length == pushed)
    end,
  }
end
