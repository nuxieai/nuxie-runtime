-- Paint definitions (crates/nuxie-scripting/src/vm/lua_paint.rs
-- `apply_definition`) are walked with `pairs`, one setter per key, so the
-- recorded op order is the hash order (the script_paths and
-- script_path_opacity silvers; docs/luau-safe-vm-pairs-order.md).
local defs = {
  { style = "stroke", color = 0xff00ff00, thickness = 4, cap = "round", join = "round" },
  { style = "fill", color = 0xff0000ff },
  { color = 0x80ffffff, thickness = 2, cap = "butt", join = "miter", style = "stroke", feather = 3 },
  { blendMode = "multiply", style = "fill", color = 0xffffffff, gradient = "linear" },
}
for _, t in ipairs(defs) do
  local keys = {}
  for k in pairs(t) do keys[#keys + 1] = k end
  print(table.concat(keys, ","))
end
