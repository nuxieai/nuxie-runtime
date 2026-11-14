--! attacks: buffer.stridedcopy with zero/negative strides and sizes, huge counts, i32-overflowing offsets and overlapping self-copies; expect: ok; within: 20s
local function safe(f, ...)
  local ok, err = pcall(f, ...)
  assert(ok or not string.find(tostring(err), "rust panic", 1, true), tostring(err))
  return ok, err
end
local I32 = 2147483647
local a = buffer.create(4096)
local big = buffer.create(1024 * 1024)
for i = 0, 4095, 4 do buffer.writeu32(a, i, i) end
local function refused(...)
  local args = table.pack(...)
  local ok, err = pcall(function() buffer.stridedcopy(table.unpack(args, 1, args.n)) end)
  assert(not ok, "stridedcopy accepted")
  assert(not string.find(tostring(err), "rust panic", 1, true), tostring(err))
end
-- dst, dstOff, dstStride, src, srcOff, srcStride, elementSize, count
safe(function() buffer.stridedcopy(a, 0, 0, a, 0, 0, 0, I32) end) -- zero stride, zero size, 2^31-1 empty copies
refused(a, 0, 0, a, 0, 0, 4, I32)
refused(a, 0, 4, a, 0, 4, -1, 1)
refused(a, 0, 4, a, 0, 4, 4, -1)
refused(a, 0, 4, a, 0, 4, 4, I32)
refused(a, I32, 4, a, 0, 4, 4, 2)
refused(a, 0, I32, a, 0, I32, 4, 3)
refused(a, -1, 4, a, 0, 4, 4, 1)
refused(a, 0, 4, a, -2147483648, 4, 4, 1)
refused(a, 0, -4, a, 0, -4, 4, 1)
refused(a, 0, 4, a, 0, 4, I32, 1)
refused(a, 4000, 4, a, 0, 4, 4, 100)
safe(function() buffer.stridedcopy(a, 0, 4, a, 0, 4, 4, 2 ^ 31) end)
safe(function() buffer.stridedcopy(a, 0, 4, a, 0, 4, 4, 0 / 0) end)
safe(function() buffer.stridedcopy(a, 0.5, 4, a, 0, 4, 4, 1) end)
-- Overlapping self copies, both directions, at the largest legal count.
buffer.stridedcopy(a, 4, 8, a, 0, 8, 4, 511)
buffer.stridedcopy(a, 0, 8, a, 4, 8, 8, 511)
buffer.stridedcopy(a, 0, 4, a, 0, 4, 4, 1024)
-- A self copy of a 1 MiB buffer (host makes a local copy), and stride 1 element 1 over it.
buffer.stridedcopy(big, 1, 1, big, 0, 1, 1, 1024 * 1024 - 1)
buffer.stridedcopy(big, 0, 16, a, 0, 4, 4, 1024)
safe(function() buffer.stridedcopy(a, 0, 4, "x", 0, 4, 4, 1) end)
return {}
