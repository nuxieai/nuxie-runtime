--! attacks: Promise callbacks that resolve, chain or cancel into themselves, mutual resolution cycles, andThen added during notification; expect: ok; within: 10s
-- A promise resolved with itself.
local resolveSelf
local p = Promise.new(function(resolve) resolveSelf = resolve end)
resolveSelf(p)
-- A child returned from its own callback.
local src = Promise.new(function() end)
local selfChild
selfChild = src:andThen(function() return selfChild end)
local r
src = Promise.resolve(1)
selfChild = src:andThen(function() return selfChild end)
-- Two pending promises resolved with each other, then cancelled.
local ra, rb
local a = Promise.new(function(resolve) ra = resolve end)
local b = Promise.new(function(resolve) rb = resolve end)
ra(b)
rb(a)
a:cancel()
b:cancel()
-- A three-cycle through andThen children, then settled.
local rx
local x = Promise.new(function(resolve) rx = resolve end)
local y = x:andThen(function(v) return x end)
local z = y:andThen(function(v) return y end)
rx(z)
-- andThen registered on a settled source from inside its own callback (bounded).
local count = 0
local settled = Promise.resolve("v")
local function again()
  count += 1
  if count < 50 then settled:andThen(again) end
end
settled:andThen(again)
assert(count >= 1 and count <= 50, count)
-- finally callbacks that cancel the chain they belong to.
local rf
local f = Promise.new(function(resolve) rf = resolve end)
local child
child = f:finally(function() child:cancel(); f:cancel() end)
rf("go")
-- onCancel handler that cancels again and resolves.
local rc
local c = Promise.new(function(resolve, reject, onCancel)
  rc = resolve
  onCancel(function() c:cancel(); rc("late") end)
end)
c:cancel()
-- Promise.all whose members include the result promise of a member's callback.
local m1 = Promise.resolve(1)
local all
local m2 = m1:andThen(function() return all end)
all = Promise.all({ m1, m2 })
all:cancel()
return {}
