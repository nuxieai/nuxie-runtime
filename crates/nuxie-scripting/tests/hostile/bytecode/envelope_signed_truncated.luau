--! attacks: malformed bytecode, a ScriptAsset payload whose signed flag is set but whose 64-byte signature is cut short at 63 bytes; expect: ok; within: 60s; harness: bytecode; bytecode: payload-hex 80 0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f202122232425262728292a2b2c2d2e2f303132333435363738393a3b3c3d3e3f
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
