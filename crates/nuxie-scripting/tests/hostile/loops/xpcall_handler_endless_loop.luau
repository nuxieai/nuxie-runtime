--! attacks: an xpcall error handler that loops forever, retried by an outer loop that swallows its result; expect: error(exceeded); within: 15s
while true do
  xpcall(function() error("body") end, function(e) while true do end end)
end
