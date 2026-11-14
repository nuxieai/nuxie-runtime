--! attacks: one 16 MiB VM string assigned to the string property of many instances (each keeps a host copy); expect: ok; within: 30s; harness: instance; viewmodels: script_create_viewmodel_instance.riv
-- The script holds one 16 MiB string; each assignment copies it into the
-- model. The 64 MiB host budget must refuse by the fifth copy; the case
-- stops at the sixth (96 MiB held) so the outcome, not the memory ceiling,
-- reports a missing charge.
local big = string.rep("s", 16 * 1024 * 1024)
local kept = {}
local refused = false
local function assign(n)
  for i = 1, n do
    if refused then return end
    local inst = Data.SubChild.new()
    local ok = pcall(function() inst:getString("str").value = big end)
    if not ok then refused = true return end
    table.insert(kept, inst)
    assert(#kept <= 5, "kept " .. #kept .. " instances holding 16 MiB strings each")
  end
end
return function()
  return {
    init = function(self) assign(8) end,
    draw = function(self, renderer) assign(8) end,
    advance = function(self, dt) assign(8) return true end,
    update = function(self)
      assign(8)
    end,
  }
end
