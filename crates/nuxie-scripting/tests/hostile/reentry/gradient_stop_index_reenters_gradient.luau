--! attacks: gradient stop __index that calls Gradient.linear/radial again on the same stops table, mutates it, and sets the result on a paint mid-build; expect: ok; within: 10s
local depth, calls = 0, 0
local paint = Paint.new()
local stops = {}
local function stop(pos)
  return setmetatable({}, { __index = function(_, k)
    calls += 1
    if depth < 3 and calls < 200 then
      depth += 1
      local inner = Gradient.linear(Vector.xy(0, 0), Vector.xy(1, 1), stops)
      paint.gradient = inner
      local radial = Gradient.radial(Vector.xy(0, 0), 3, stops)
      paint.gradient = radial
      depth -= 1
    end
    -- Mutate the table being read: remove the next stop and replace an earlier one.
    stops[3] = nil
    stops[1] = false
    if k == "position" then return pos end
    return 0xff00ff00
  end })
end
for i = 1, 6 do stops[i] = stop(i / 6) end
local ok, g = pcall(Gradient.linear, Vector.xy(0, 0), Vector.xy(10, 0), stops)
assert(ok, tostring(g))
paint.gradient = g
assert(paint.gradient ~= nil)
-- A stop whose __index raises an error object whose __tostring builds a gradient.
local raising = { setmetatable({}, { __index = function()
  error(setmetatable({}, { __tostring = function()
    Gradient.linear(Vector.xy(0, 0), Vector.xy(1, 0), { { position = 0, color = 1 } })
    return "boxed"
  end }))
end }) }
local ok2, err = pcall(Gradient.radial, Vector.xy(0, 0), 1, raising)
assert(not ok2)
tostring(err)
-- A stop whose __index returns a color proxy table.
local ok3 = pcall(Gradient.linear, Vector.xy(0, 0), Vector.xy(1, 0), {
  setmetatable({}, { __index = function(_, k) if k == "position" then return 0 end; return setmetatable({}, { __tostring = function() return "1" end }) end }),
})
assert(not ok3, "a table color is invalid")
return {}
