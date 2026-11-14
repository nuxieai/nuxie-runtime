--! attacks: host userdata (Path, Paint, Gradient, Text, Mat2D, Promise, coroutines holding them) kept only in weak tables across collections, then used; expect: ok; within: 20s
local weakV = setmetatable({}, { __mode = "v" })
local weakK = setmetatable({}, { __mode = "k" })
local weakKV = setmetatable({}, { __mode = "kv" })
local function fill()
  for i = 1, 40 do
    local p = Path.new()
    p:moveTo(Vector.xy(0, 0))
    p:lineTo(Vector.xy(i, i))
    local paint = Paint.with({ color = 0xff000000 + i, thickness = i })
    local g = Gradient.linear(Vector.xy(0, 0), Vector.xy(i, 0), { { position = 0, color = 1 }, { position = 1, color = 2 } })
    paint.gradient = g
    weakV[i] = p
    weakV[100 + i] = paint
    weakV[200 + i] = g
    weakV[300 + i] = Text.new()
    weakV[400 + i] = p:contours()
    weakK[p] = paint
    weakK[paint] = g
    weakKV[g] = p
    -- A coroutine that is the only owner of a path.
    weakV[500 + i] = coroutine.create(function() local held = Path.new(); held:moveTo(Vector.xy(1, 1)); coroutine.yield(); return #held end)
    coroutine.resume(weakV[500 + i])
  end
end
fill()
-- Churn garbage in several bursts so incremental collection steps run (no collectgarbage in the sandbox).
local function churn()
  for _ = 1, 8 do
    local junk = {}
    for j = 1, 400 do junk[j] = { j, tostring(j) } end
  end
end
local used = 0
for round = 1, 5 do
  churn()
  for key, value in pairs(weakV) do
    used += 1
    local kind = typeof(value)
    if kind == "Path" then pcall(function() value:lineTo(Vector.xy(round, round)); return #value end)
    elseif kind == "Paint" then pcall(function() value.thickness = round; return value.gradient end)
    elseif kind == "thread" then pcall(coroutine.resume, value)
    else pcall(function() return tostring(value) end) end
  end
  for key, value in pairs(weakK) do pcall(function() return #key, value.color end) end
  for key, value in pairs(weakKV) do pcall(function() return #value end) end
  if round == 3 then fill() end
end
assert(used > 0)
-- Leave the weak tables populated for the final collection and VM drop.
return { weakV = weakV, weakK = weakK, weakKV = weakKV }
