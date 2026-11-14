--! attacks: an endless coroutine.wrap generator consumed by a generic for loop; expect: error(exceeded); within: 15s
local gen = coroutine.wrap(function() local i = 0 while true do i = i + 1 coroutine.yield(i) end end)
for v in gen do end
