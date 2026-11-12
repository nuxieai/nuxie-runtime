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
for _, collect in ipairs({false, true}) do
  local t = {}
  for i = 12, 32, 2 do t[i] = i end
  t[k3] = 1
  t[k2] = 2
  t[k2] = nil
  if collect then collectgarbage() end
  -- `next` still accepts the deleted key as a cursor, dead or not.
  print(collect, "next", (next(t, k2)))
  t[k2] = 3
  print(collect, order(t))
  -- A different key into the emptied table, and the same key again.
  t[k3] = nil
  if collect then collectgarbage() end
  t.other = 4
  t[k3] = 5
  print(collect, order(t))
end
