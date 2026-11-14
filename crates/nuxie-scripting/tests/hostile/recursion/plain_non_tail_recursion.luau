--! attacks: unbounded non-tail Luau recursion (plain, wide frames, vararg frames) caught by pcall in each instance callback; expect: ok; within: 20s; harness: instance
local function refused(f, ...)
  local ok, err = pcall(f, ...)
  assert(not ok, "unbounded recursion returned normally")
  err = tostring(err)
  assert(not string.find(err, "rust panic", 1, true), err)
  assert(string.find(err, "stack overflow", 1, true), err)
end
local function down(n) return 1 + down(n + 1) end
local function wide(n)
  local a, b, c, d, e, f, g, h, i, j, k, l, m, o, p, q = n, n, n, n, n, n, n, n, n, n, n, n, n, n, n, n
  return wide(n + 1) + a + b + c + d + e + f + g + h + i + j + k + l + m + o + p + q
end
local function va(...) return select("#", va(1, ...)) end
local function fine(n) if n == 0 then return 0 end return 1 + fine(n - 1) end
return function()
  return {
    init = function(self) refused(down, 1) end,
    draw = function(self, renderer) refused(wide, 1) end,
    advance = function(self, dt) refused(va, table.unpack(table.create(100, 0))) return true end,
    update = function(self) assert(fine(1000) == 1000) end,
  }
end
