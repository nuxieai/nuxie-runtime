--! attacks: buffer.convert with zero strides and huge counts in one buffer, huge components, negative strides, bad formats, overflowing offsets; expect: ok; within: 30s
local function safe(f, ...)
  local ok, err = pcall(f, ...)
  assert(ok or not string.find(tostring(err), "rust panic", 1, true), tostring(err))
  return ok, err
end
local I32 = 2147483647
local a = buffer.create(4096)
for i = 0, 4095, 4 do buffer.writef32(a, i, i / 7) end
local function refused(...)
  local args = table.pack(...)
  local ok, err = pcall(function() buffer.convert(table.unpack(args, 1, args.n)) end)
  assert(not ok, "convert accepted")
  assert(not string.find(tostring(err), "rust panic", 1, true), tostring(err))
end
-- dst, dstOff, dstFmt, src, srcOff, srcFmt, count, components, dstStride, srcStride
local formats = {"f16", "f32", "u8", "u8norm", "i8norm", "u16", "u16norm", "i16norm", "u32"}
for _, df in ipairs(formats) do
  for _, sf in ipairs(formats) do
    -- Zero strides, one buffer, 2^31-1 repeats: must settle or error, not hang.
    safe(function() buffer.convert(a, 0, df, a, 2, sf, I32, 3, 0, 0) end)
    safe(function() buffer.convert(a, 1, df, a, 0, sf, I32, 1, 0, 0) end)
    -- Overlapping packed in-place run.
    safe(function() buffer.convert(a, 3, df, a, 0, sf, 100, 2) end)
  end
end
refused(a, 0, "f64", a, 0, "f32", 1)
refused(a, 0, "f32", a, 0, "", 1)
refused(a, 0, "f32", a, 0, "f32", -1)
refused(a, 0, "f32", a, 0, "f32", 1, 0)
refused(a, 0, "f32", a, 0, "f32", 1, -5)
refused(a, 0, "f32", a, 0, "f32", 1, I32)
refused(a, 0, "f32", a, 0, "f32", 1, I32, 0, 0)
refused(a, 0, "f32", a, 0, "f32", 2, 1, -4, 4)
refused(a, 0, "f32", a, 0, "f32", 2, 1, 4, -4)
refused(a, 0, "f32", a, 0, "f32", 2, 2, 4, 8)
refused(a, I32, "f32", a, 0, "f32", 1)
refused(a, -1, "f32", a, 0, "f32", 1)
refused(a, 0, "f32", a, 0, "f32", I32)
refused(a, 0, "f32", a, 0, "f32", 2, 1, I32, 4)
refused(a, 0, "u8", a, 4000, "f32", 100, 1, 1, 4)
safe(function() buffer.convert(a, 0, "f32", a, 0, "f32", 0 / 0) end)
safe(function() buffer.convert(a, 0, "f32", "x", 0, "f32", 1) end)
safe(function() buffer.convert(a, 0, 5, a, 0, "f32", 1) end)
-- NaN/inf through every float path.
buffer.writef32(a, 0, 0 / 0) buffer.writef32(a, 4, math.huge) buffer.writef32(a, 8, -math.huge)
for _, df in ipairs(formats) do buffer.convert(a, 100, df, a, 0, "f32", 3) end
safe(function() buffer.writef16(a, 0, 0 / 0) return buffer.readf16(a, 0) end)
safe(function() return buffer.readf16(a, 4095) end)
safe(function() return buffer.readf16(a, -1) end)
safe(function() buffer.writef16(a, I32, 1) end)
return {}
