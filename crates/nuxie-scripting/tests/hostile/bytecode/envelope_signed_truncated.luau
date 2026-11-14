--! attacks: malformed bytecode, a ScriptAsset payload whose signed flag is set but whose 64-byte signature is cut short (the chunk is the tail); expect: ok; within: 60s; harness: bytecode; bytecode: payload-prefix 80
local t = { 1, 2.5, "three", true, nil, { nested = { deep = 4 } } }
local function add(a, b) return a + b end
local function closure(n)
  local acc = 0
  return function(x) acc += x * n; return acc end
end
local f = closure(3)
for i = 1, 10 do f(i) end
local s = string.format("%d:%s", #t, tostring(t[3]))
local v = vector.create(1, 2, 3)
local ok = pcall(function() error({ code = 7 }) end)
return { value = add(f(1), #s) + v.x, ok = ok }
