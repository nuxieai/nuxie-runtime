--! attacks: about 20,000 property wrappers each holding a listener, then one write notifies them all; expect: ok; within: 30s; harness: instance; viewmodels: script_create_viewmodel_instance.riv
local inst = nil
local kept = {}
local calls = 0
local function make(n)
  for i = 1, n do
    local ok, w = pcall(function() return inst:getNumber("num") end)
    if not ok then return end
    pcall(function() w:addListener(function() calls += 1 end) end)
    table.insert(kept, w)
  end
end
return function()
  return {
    init = function(self) inst = Data.SubChild.new() make(3000) end,
    draw = function(self, renderer) make(3000) end,
    advance = function(self, dt) make(3000) return true end,
    update = function(self)
      pcall(function() inst:getNumber("num").value = #kept end)
      assert(calls <= #kept, "calls " .. calls)
    end,
  }
end
