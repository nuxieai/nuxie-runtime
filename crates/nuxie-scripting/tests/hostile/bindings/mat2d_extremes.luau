--! attacks: Mat2D with NaN/inf entries, singular invert, multiply and constructors with extreme values; expect: ok; within: 10s
local function safe(f, ...)
  local ok, err = pcall(f, ...)
  assert(ok or not string.find(tostring(err), "rust panic", 1, true), tostring(err))
  return ok, err
end
local nan, inf = 0 / 0, math.huge
local singular = Mat2D.values(0, 0, 0, 0, 0, 0)
local bad = Mat2D.values(nan, inf, -inf, nan, inf, nan)
local out = Mat2D.identity()
assert(Mat2D.invert(out, singular) == false, "singular matrix inverted")
safe(function() return Mat2D.invert(out, bad) end)
safe(function() return Mat2D.invert(out, Mat2D.values(1e-45, 0, 0, 1e-45, 0, 0)) end)
safe(function() return singular:invert() end)
safe(function() return bad:invert() end)
safe(function() return bad * bad, bad * Vector.xy(nan, inf), singular * Vector.xy(1, 1) end)
safe(function() return bad * 5 end)
safe(function() return bad * "x" end)
safe(function() return bad == bad, bad:isIdentity() end)
for _, x in ipairs({nan, inf, -inf, 3e38}) do
  safe(function() return Mat2D.withRotation(x) end)
  safe(function() return Mat2D.withScale(x, x) end)
  safe(function() return Mat2D.withTranslation(x, x) end)
  safe(function() return Mat2D.withScaleAndTranslation(x, x, x, x) end)
end
safe(function() return Mat2D.values() end)
safe(function() return Mat2D.invert(Vector.xy(1, 1), bad) end)
safe(function() return Mat2D.invert(bad, Path.new()) end)
return {}
