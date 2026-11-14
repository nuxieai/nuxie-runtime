--! attacks: buffer functions (core and Rive stridedcopy/convert) given tables with __index/__tostring that re-enter buffer ops on the same buffer; expect: ok; within: 10s
local b = buffer.create(64)
local trick = setmetatable({}, {
  __index = function() buffer.fill(b, 0, 255); return 1 end,
  __tostring = function() buffer.writeu32(b, 0, 7); return "1" end,
  __len = function() return 4 end,
})
local fns = {
  function() buffer.writeu8(b, trick, 1) end,
  function() buffer.writeu8(b, 0, trick) end,
  function() buffer.readu32(b, trick) end,
  function() buffer.copy(b, 0, b, trick) end,
  function() buffer.writestring(b, 0, trick) end,
  function() buffer.fill(b, trick, 1) end,
  function() buffer.stridedcopy(b, 0, 4, b, trick, 4, 4, 4) end,
  function() buffer.stridedcopy(b, 0, 4, trick, 0, 4, 4, 4) end,
  function() buffer.convert(b, trick) end,
  function() buffer.convert(trick) end,
}
for i, f in ipairs(fns) do
  local ok = pcall(f)
  assert(not ok, "call " .. i .. " accepted a table")
end
-- Self-overlapping strided copy and copy are well-defined; must not crash.
buffer.stridedcopy(b, 4, 4, b, 0, 4, 4, 15)
buffer.copy(b, 1, b, 0, 63)
-- String numbers are coerced; a string with __index tricks cannot be built, but a huge numeric string must fail cleanly.
assert(not pcall(buffer.writeu8, b, "1e300", 1))
return {}
