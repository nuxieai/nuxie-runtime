--! attacks: an endless resume/yield ping-pong, each resume a fresh protected host entry, from an advance callback; expect: error(exceeded); within: 15s; harness: instance
return function()
  return {
    advance = function(self, dt)
      local co = coroutine.create(function() while true do coroutine.yield() end end)
      while true do coroutine.resume(co) end
    end,
  }
end
