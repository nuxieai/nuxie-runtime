--! attacks: closest script-only form of the listener RefCell row (E1 2(a) row 1, G4 part 1): a property listener that reads, rewrites and re-listens to the property that notified it; expect: ok; within: 10s; viewmodels: script_create_viewmodel_instance.riv
local vm = Data.SubChild.new()
local num, str = vm.num, vm.str
assert(typeof(num) == "userdata" and typeof(str) == "userdata", "the fixture has SubChild.num and SubChild.str")
local heard, added = 0, 0
local function listener()
  heard += 1
  local v = num.value
  local s = str.value
  if heard < 30 then
    num.value = v + 1
    str.value = s .. "x"
    if added < 5 then added += 1; num:addListener(listener) end
  end
end
num:addListener(listener)
str:addListener(function()
  -- Cross-property write back into the first property from the second's listener.
  if heard < 30 then num.value = num.value + 0.5 end
end)
num.value = 1
-- A listener raising an error object whose __tostring writes the property.
num:addListener(function()
  error(setmetatable({}, { __tostring = function() num.value = -1; return "boxed" end }))
end)
pcall(function() num.value = 2 end)
assert(heard >= 1, "listeners ran " .. heard)
assert(heard < 1000, "listener feedback is bounded: " .. heard)
return {}
