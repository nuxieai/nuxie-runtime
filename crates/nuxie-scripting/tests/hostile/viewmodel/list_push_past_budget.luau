--! attacks: view-model list push of distinct new items in a loop past the host budget; expect: ok; within: 30s; harness: instance; viewmodels: data_bind_blob_test.riv
-- Each push is charged; growth must stop at an ordinary refusal rather than
-- grow host memory without bound. The list length must agree with what was pushed.
local root = nil
local pushed = 0
local refused = false
local function grow(n)
  for i = 1, n do
    if refused then return end
    local ok = pcall(function() root:getList("children"):push(Data.XMLNodeVM.new()) end)
    if not ok then refused = true return end
    pushed += 1
  end
end
return function()
  return {
    init = function(self) root = Data.Main.new() grow(3000) end,
    draw = function(self, renderer) grow(3000) end,
    advance = function(self, dt) grow(3000) return true end,
    update = function(self)
      grow(3000)
      assert(root:getList("children").length == pushed)
    end,
  }
end
