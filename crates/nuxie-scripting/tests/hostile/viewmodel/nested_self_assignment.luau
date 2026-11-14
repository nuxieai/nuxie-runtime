--! attacks: nested view-model self-assignment (main.chi = main), a cycle through a nested slot; expect: ok; within: 10s; viewmodels: script_create_viewmodel_instance.riv
-- The parent becomes its own child. The assignment must be refused or leave a
-- cycle that later walks terminate on; it must not recurse without bound.
local main = Data.Main.new()
local chi = main:getViewModel("chi")
pcall(function() chi.value = main end)
local _ = main:getViewModel("chi").value
main = nil
return {}
