--! attacks: print with __tostring that errors, returns non-strings, recurses into print, and nil holes; expect: ok; within: 10s
local function safe(f, ...)
  local ok, err = pcall(f, ...)
  assert(ok or not string.find(tostring(err), "rust panic", 1, true), tostring(err))
  return ok, err
end
local erring = setmetatable({}, {__tostring = function() error("boom") end})
local nonstring = setmetatable({}, {__tostring = function() return {} end})
local numberish = setmetatable({}, {__tostring = function() return 5 end})
local depth = 0
local recursive
recursive = setmetatable({}, {__tostring = function()
  depth += 1
  if depth < 150 then print(recursive, string.rep("r", 100000)) end
  return string.rep("x", 1024 * 1024)
end})
safe(function() print(erring) end)
safe(function() print("a", erring, "b") end)
safe(function() print(nonstring) end)
safe(function() print(numberish) end)
safe(function() print(recursive) end)
print(nil, nil, nil, 1, nil)
print(Path.new(), Paint.new(), Vector.xy(0 / 0, 1), Mat2D.identity(), buffer.create(10))
print(table.unpack(table.create(7000, nil), 1, 7000))
print()
return {}
