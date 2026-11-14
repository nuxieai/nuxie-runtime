--! attacks: a nested instance assigned into its own nested slot (child.subch = child, wrong type too); expect: ok; within: 10s; viewmodels: script_create_viewmodel_instance.riv
local c = Data.child.new()
pcall(function() c:getViewModel("subch").value = c end)
c = nil
return {}
