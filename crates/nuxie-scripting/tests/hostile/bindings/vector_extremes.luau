--! attacks: Vector math with NaN/inf and Vector.writeToBuffer/writeVec4 at bad offsets and into too-small buffers; expect: ok; within: 10s
local function safe(f, ...)
  local ok, err = pcall(f, ...)
  assert(ok or not string.find(tostring(err), "rust panic", 1, true), tostring(err))
  return ok, err
end
local nan, inf = 0 / 0, math.huge
local v = Vector.xy(nan, inf)
local w = Vector.xy(-inf, 3e38)
safe(function() return Vector.normalized(v), Vector.length(w), Vector.lengthSquared(w) end)
safe(function() return Vector.distance(v, w), Vector.distanceSquared(w, w), Vector.dot(v, w), Vector.cross(v, w) end)
safe(function() return Vector.lerp(v, w, nan), Vector.scaleAndAdd(v, w, inf), Vector.scaleAndSub(w, v, -inf) end)
safe(function() return Vector.normalized(Vector.xy(0, 0)) end)
safe(function() return Vector.xy(), Vector.xyz(nan, nan, nan), Vector.origin() end)
local function refusedWrite(f)
  local ok, err = pcall(f)
  assert(not ok, "write was accepted")
  assert(not string.find(tostring(err), "rust panic", 1, true), tostring(err))
end
for _, size in ipairs({0, 1, 7}) do
  refusedWrite(function() Vector.writeToBuffer(w, buffer.create(size), 0) end)
end
for _, size in ipairs({0, 7, 15}) do
  refusedWrite(function() Vector.writeVec4(w, buffer.create(size), 0, 1) end)
end
local b = buffer.create(32)
for _, off in ipairs({-1, -2147483648, 32, 31, 29, 2 ^ 31, 2 ^ 53, 2 ^ 63, -2 ^ 63, 1e300}) do
  refusedWrite(function() Vector.writeToBuffer(w, b, off) end)
  refusedWrite(function() Vector.writeVec4(w, b, off, nan) end)
end
safe(function() Vector.writeToBuffer(w, b, nan) end)
safe(function() Vector.writeToBuffer(w, b, 0.5) end)
Vector.writeToBuffer(v, b, 0)
Vector.writeVec4(v, b, 16, inf)
safe(function() Vector.writeToBuffer(w, "not a buffer", 0) end)
safe(function() Vector.writeToBuffer(5, b, 0) end)
return {}
