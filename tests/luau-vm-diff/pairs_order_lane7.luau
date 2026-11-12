-- The lane 7 reproduction (docs/evidence/luau-safe-vm-phase2-lane7-2026-11-11.md,
-- section 9): `data_bind_blob_test`'s editor script shows the first
-- attribute `pairs` yields, so the safe VM must iterate string-keyed hash
-- parts in the vendored VM's order (docs/luau-safe-vm-pairs-order.md).
local keys = { "defaultStateMachineId", "selectedAnimations", "clip", "width",
  "height", "styleId", "name", "id" }
local t = {}
for i, k in ipairs(keys) do t[k] = tostring(i) end
local out = {}
for k in pairs(t) do out[#out + 1] = k end
print(table.concat(out, ","))
