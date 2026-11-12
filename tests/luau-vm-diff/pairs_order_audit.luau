-- The placement audit (docs/luau-safe-vm-pairs-order.md, 3.3): one case per
-- path that sizes or fills a hash part. Each prints `pairs` order, which
-- must match the vendored VM for value-typed keys. Item 4 (dead keys) is
-- pairs_order_dead_key.luau.
local function order(t)
  local o = {}
  for k, v in pairs(t) do
    local ks = type(k) == "vector" and string.format("<%g,%g,%g>", k.x, k.y, k.z) or tostring(k)
    o[#o + 1] = ks .. "=" .. (type(v) == "table" and "tbl" or tostring(v))
  end
  return table.concat(o, " ")
end
local names = { "defaultStateMachineId", "selectedAnimations", "clip", "width",
  "height", "styleId", "name", "id", "x", "y", "visible", "opacity" }

-- 1. NEWTABLE with a hash size hint, then DUPTABLE (the template's node
-- layout is copied, not rebuilt), then a store past the template.
print("1 newtable", order({ width = 1, height = 2, x = 3, y = 4, name = 5 }))
local function template() return { width = 0, height = 0, x = 0, y = 0, name = "" } end
local d = template()
print("1 duptable", order(d))
d.opacity = 1
d.visible = true
print("1 duptable+", order(d))

-- 2. table.create, table.clone, table.clear (capacity kept, lastfree
-- reset), table.move into a hash part, table.freeze.
local c = table.create(4, 0)
for i, k in ipairs(names) do c[k] = i end
print("2 create", order(c))
local src = {}
for i, k in ipairs(names) do src[k] = i end
src.clip = nil
local cl = table.clone(src)
cl.clip = 99
print("2 clone", order(cl))
table.clear(src)
for i = #names, 1, -1 do src[names[i]] = i end
print("2 clear", order(src))
local m = { a = 1 }
table.move({ 10, 20, 30, 40 }, 1, 4, 100, m)
print("2 move", order(m))
local f = table.freeze({ width = 1, clip = 2, id = 3 })
print("2 freeze", order(f))

-- 3. A full hash part: the rehash sizes, the reverse re-insert, and an
-- integer key that migrates between the array part and the hash part.
local r = {}
for i = 1, 9 do r[names[i]] = i end
print("3 rehash", order(r))
local mig = { x = 1, y = 2 }
mig[3] = "c"; mig[2] = "b"; mig[1] = "a"
print("3 into array", order(mig))
mig[4] = "d"; mig[8] = "h"; mig[100] = "z"
print("3 grow", order(mig))
local shr = {}
for i = 1, 16 do shr[i] = i end
for i = 1, 15 do shr[i] = nil end
shr.k = true
shr.j = false
print("3 shrink array", order(shr))

-- 5. Collisions: the colliding node out of its main position moves; one in
-- its main position stays and the new key takes the free node. Number keys
-- 0.5 apart in a small table collide often.
local col = {}
for i = 1, 24 do col[i * 0.5 + 1000] = i end
print("5 numbers", order(col))
local colv = {}
for i = 1, 12 do colv[vector.create(i, -i, i * 2)] = i end
print("5 vectors", order(colv))
local colb = { [true] = 1, [false] = 2, [-0] = 3, [0.25] = 4, ["true"] = 5 }
print("5 mixed", order(colb))

-- 6. A weak-valued table in `s` mode shrinks after a collection; without
-- `s` it keeps its size. The collectable values are made in a callee, and
-- its stack slots overwritten before collecting, so no stale register
-- keeps one alive.
for _, mode in ipairs({ "v", "vs" }) do
  local w = setmetatable({}, { __mode = mode })
  local keep = {}
  local function put(i) local v = {} w[names[i]] = v if i % 4 == 0 then keep[#keep + 1] = v end end
  local function scrub() local a, b, c, d, e, f, g, h, i, j, k, l = 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0 end
  for i = 1, #names do put(i) end
  scrub()
  collectgarbage()
  w.extra1 = 1
  w.extra2 = 2
  print("6 " .. mode, order(w))
end

-- 7. The host API: a table created and filled with raw_set by the host,
-- iterated by the host's pairs.
local h = hosttable("width", "height", "clip", "id", 3.5, true, vector.create(1, 2, 3), "name")
print("7 host pairs", order(h))
local hk = hostkeys(h)
local out = {}
for i, k in ipairs(hk) do out[i] = type(k) == "vector" and "vec" or tostring(k) end
print("7 host keys", table.concat(out, " "))

-- 8. Library tables as the sandbox builds them: `pairs` over them is
-- observable.
for _, lib in ipairs({ "string", "table", "math", "bit32", "coroutine", "utf8", "buffer", "vector", "os", "debug" }) do
  local t = _G[lib]
  if t then
    local keys = {}
    for k in pairs(t) do keys[#keys + 1] = k end
    print("8 " .. lib, table.concat(keys, " "))
  end
end
local g = {}
for k in pairs(_G) do g[#g + 1] = tostring(k) end
print("8 _G", table.concat(g, " "))
