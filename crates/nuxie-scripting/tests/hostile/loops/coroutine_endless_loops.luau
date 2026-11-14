--! attacks: a coroutine whose body loops forever without yielding, resumed in a loop that swallows the result; expect: error(exceeded); within: 15s
local co = coroutine.create(function() while true do end end)
while true do
  coroutine.resume(co)
  co = coroutine.create(function() while true do end end)
end
