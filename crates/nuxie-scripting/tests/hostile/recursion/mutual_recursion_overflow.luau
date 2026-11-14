--! attacks: mutual non-tail recursion overflow, then a second overflow started from 10,000 frames deep; expect: ok; within: 20s
local isEven, isOdd
function isEven(n) if n == 0 then return true end return not not isOdd(n - 1) end
function isOdd(n) if n == 0 then return false end return not not isEven(n - 1) end
local ok, err = pcall(isEven, 1e9)
assert(not ok and string.find(tostring(err), "stack overflow", 1, true), tostring(err))
assert(not string.find(tostring(err), "rust panic", 1, true), tostring(err))
local function over(n) return 1 + over(n + 1) end
local hit = false
local function outer(depth)
  if depth == 0 then
    local ok2, e2 = pcall(over, 1)
    assert(not ok2 and string.find(tostring(e2), "stack overflow", 1, true), tostring(e2))
    hit = true
    return 0
  end
  return 1 + outer(depth - 1)
end
assert(outer(10000) == 10000)
assert(hit)
return {}
