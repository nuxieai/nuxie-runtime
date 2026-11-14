--! attacks: an endless loop of cheap protected host entries that each fail (pcall(error), pcall(tostring), coroutine.resume of a dead coroutine); expect: error(exceeded); within: 15s
local dead = coroutine.create(function() end)
coroutine.resume(dead)
while true do
  pcall(error, "x")
  pcall(tostring, setmetatable({}, { __tostring = function() error("y") end }))
  coroutine.resume(dead)
  xpcall(error, function(e) return e end, "z")
end
