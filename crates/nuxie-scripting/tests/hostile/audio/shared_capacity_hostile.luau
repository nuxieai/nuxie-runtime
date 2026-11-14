--! attacks: Audio.Ring/Region/Snapshot.create with hostile byte capacities; expect: ok; within: 10s
-- Every bad capacity is an ordinary error; the edges that are legal are created.
local bad = {-1, -0.5, 0.5, 0 / 0, math.huge, -math.huge, 2 ^ 32, 2 ^ 53, 2 ^ 1023, "16", true, {}, print}
for _, ctor in ipairs({Audio.Ring.create, Audio.Region.create, Audio.Snapshot.create}) do
  for _, size in ipairs(bad) do
    local ok, err = pcall(ctor, size)
    assert(not ok, "accepted capacity " .. tostring(size))
    assert(string.find(tostring(err), "capacity") or string.find(tostring(err), "size"), tostring(err))
  end
  assert(not pcall(ctor))
  assert(not pcall(ctor, 16, 16))
  -- One past the 256 MiB per-object limit, and 2^32 - 1 (fits the u32 check).
  assert(not pcall(ctor, 256 * 1024 * 1024 + 1))
  assert(not pcall(ctor, 2 ^ 32 - 1))
end
-- Zero: only a snapshot may be empty.
assert(not pcall(Audio.Ring.create, 0))
assert(not pcall(Audio.Region.create, 0))
local empty = Audio.Snapshot.create(0)
assert(empty:latest() == nil)
local ok = pcall(function() return empty:publish(1) end)
-- either refused or reported as not stored; it must not crash
local tiny = Audio.Ring.create(1)
assert(tiny:send(nil) == false)
assert(tiny:receive() == nil)
return {}
