--! attacks: Mat4 singular invert, NaN/inf entries, degenerate projections, bad element keys and writeToBuffer offsets; expect: ok; within: 10s
local function safe(f, ...)
  local ok, err = pcall(f, ...)
  assert(ok or not string.find(tostring(err), "rust panic", 1, true), tostring(err))
  return ok, err
end
local nan, inf = 0 / 0, math.huge
local zero = Mat4.values(0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0)
local bad = Mat4.values(nan, inf, -inf, nan, inf, nan, 1, 2, 3, 4, nan, inf, -inf, 0, 0, nan)
safe(function() return zero:invert(), zero:invertAffine(), Mat4.invert(zero), Mat4.invertAffine(zero) end)
safe(function() return bad:invert(), bad:transpose(), bad:transformPoint(nan, inf, -inf) end)
safe(function() return bad:transformVec4(nan, nan, nan, nan) end)
safe(function() return Mat4.multiply(bad, zero), Mat4.multiplyAffine(bad, bad) end)
safe(function() return Mat4.perspective(0, 0, 0, 0) end)
safe(function() return Mat4.perspective(nan, inf, -inf, nan) end)
safe(function() return Mat4.perspectiveReverseZ(0, 0, 0) end)
safe(function() return Mat4.ortho(1, 1, 1, 1, 1, 1) end)
safe(function() return Mat4.lookAt(Vector.xyz(0, 0, 0), Vector.xyz(0, 0, 0), Vector.xyz(0, 0, 0)) end)
safe(function() return Mat4.fromRotationX(inf), Mat4.fromScale(nan, nan, nan), Mat4.fromTranslation(inf, inf, inf) end)
for _, k in ipairs({"m00", "m55", "m10", "m01", "m17", "0", "+0", " 0", "-1", "m", "", "m1", "m111"}) do
  safe(function() return bad[k] end)
  safe(function() bad[k] = 1 end)
end
for _, k in ipairs({0, -1, 17, 2 ^ 31, 2 ^ 53, nan, inf, 1.5}) do
  safe(function() return bad[k] end)
  safe(function() bad[k] = 1 end)
end
local function refusedWrite(b, off)
  local ok, err = pcall(function() bad:writeToBuffer(b, off) end)
  assert(not ok, "Mat4 write at " .. tostring(off) .. " was accepted")
  assert(not string.find(tostring(err), "rust panic", 1, true), tostring(err))
end
refusedWrite(buffer.create(0), 0)
refusedWrite(buffer.create(63), 0)
local b = buffer.create(128)
for _, off in ipairs({-1, 65, 128, 2 ^ 31, 2 ^ 53, -2 ^ 63, 2 ^ 63, 1e300}) do refusedWrite(b, off) end
safe(function() bad:writeToBuffer(b, nan) end)
bad:writeToBuffer(b, 64)
safe(function() Mat4.values(1, 2, 3) end)
safe(function() Mat4.values("a") end)
return {}
