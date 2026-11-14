--! attacks: a trigger listener that fires its own trigger (and a second trigger that fires back) during notification; expect: ok; within: 10s; viewmodels: script_create_viewmodel_instance.riv
local b = Data.button.new()
local t = b:getTrigger("onClick")
local n = 0
t:addListener(function()
  n += 1
  if n < 100000 then pcall(function() t:fire() end) end
end)
t:fire()
assert(n < 1000, "trigger re-fired " .. n .. " times")
local s = Data.VMSettings.new()
local t1, t2 = s:getTrigger("newItem"), s:getTrigger("swapItems")
local m = 0
t1:addListener(function() m += 1 pcall(function() t2:fire() end) end)
t2:addListener(function() m += 1 pcall(function() t1:fire() end) end)
t1:fire()
assert(m < 1000, "trigger cycle ran " .. m .. " times")
return {}
