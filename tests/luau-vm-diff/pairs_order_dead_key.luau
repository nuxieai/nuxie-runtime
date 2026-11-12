-- Audit item 4 (docs/luau-safe-vm-pairs-order.md, 3.3): delete a key that
-- sits outside its main position, then store it again. Before a collection
-- the node is reused; after one, C++'s collector has made the key dead
-- (`removeentry`), lookups pass it by, and the key takes a free node.
local k2, k3 = "9i7SgmmBaY7xV7uz", "JMiLAEYuuSiNzjQ5yaFHB1cfupenRJ4S7qqCSgZWz_j0Vq509hWRrrLHy9x8itEpcxbvAzBMVCSl_DREbXtA7Ea6sW7qApmlFEIf1pz9vh0WqeZA0hoxFCU1"
local function order(t)
  local o = {}
  for k, v in pairs(t) do o[#o + 1] = tostring(k):sub(1, 4) .. "=" .. v end
  return table.concat(o, " ")
end
-- In a table weak in both keys and values, C++'s collector leaves empty
-- nodes alone (`traversetable` returns before its node loop), so the old
-- node is reused even after a collection.
for _, case in ipairs({ { false, nil }, { true, nil }, { true, "kv" }, { true, "v" } }) do
  local collect, mode = case[1], case[2]
  local t = mode and setmetatable({}, { __mode = mode }) or {}
  for i = 12, 32, 2 do t[i] = i end
  t[k3] = 1
  t[k2] = 2
  t[k2] = nil
  if collect then collectgarbage() end
  -- `next` still accepts the deleted key as a cursor, dead or not.
  print(collect, mode, "next", (next(t, k2)))
  t[k2] = 3
  print(collect, mode, order(t))
  -- A different key into the emptied table, and the same key again.
  t[k3] = nil
  if collect then collectgarbage() end
  t.other = 4
  t[k3] = 5
  print(collect, mode, order(t))
end
