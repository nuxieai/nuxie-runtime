--! attacks: view-model list push of 21,000 distinct new items across seven callbacks, as far as their instruction budgets reach (not as far as the 64 MiB host budget); expect: ok; within: 30s; harness: instance; viewmodels: data_bind_blob_test.riv
-- Each push is charged. Empty nodes are small, so the per-callback
-- instruction budget ends each callback long before 64 MiB: this case checks
-- that a long push run stays consistent. No case drives a list push itself
-- past the budget (see the lane H4 evidence); blob_property_huge does it
-- through a view-model value.
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
