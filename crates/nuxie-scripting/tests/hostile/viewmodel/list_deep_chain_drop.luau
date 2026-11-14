--! attacks: a deep chain of nested view models built top-down through list push (each push charges only an empty node), then dropped at collection; expect: ok; within: 40s; harness: instance; viewmodels: data_bind_blob_test.riv
-- Built bottom-up, each push charges the whole subtree and the budget stops
-- the chain near 400 links; built top-down every push is one empty node, so
-- the chain reaches tens of thousands of links. Dropping it must not recurse.
local root, tail = nil, nil
local depth = 0
local function grow(n)
  for i = 1, n do
    local node = Data.XMLNodeVM.new()
    local ok = pcall(function() tail:getList("children"):push(node) end)
    if not ok then return end
    tail = node
    depth += 1
  end
end
return function()
  return {
    init = function(self) root = Data.XMLNodeVM.new() tail = root grow(3000) end,
    draw = function(self, renderer) grow(3000) end,
    advance = function(self, dt) grow(3000) return true end,
    update = function(self)
      grow(3000)
      -- one more bottom-up push of the whole chain: its charge walks every link
      pcall(function() Data.Main.new():getList("children"):push(root) end)
    end,
  }
end
