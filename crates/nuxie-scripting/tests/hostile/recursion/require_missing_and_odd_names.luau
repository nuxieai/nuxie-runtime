--! attacks: require of missing, empty, huge, non-string and self-referential names; expect: ok; within: 15s
local function refused(f, ...)
  local ok, err = pcall(f, ...)
  assert(not ok, "accepted")
  err = tostring(err)
  assert(not string.find(err, "rust panic", 1, true), err)
  return err
end
assert(refused(require, "does_not_exist"))
assert(refused(require, ""))
assert(refused(require, nil))
assert(refused(require, 1e308))
assert(refused(require, {}))
assert(refused(require, string.rep("m", 1000000)))
assert(refused(require, "recursion/require_missing_and_odd_names"))
assert(refused(require, "require_missing_and_odd_names"))
assert(refused(require, "../../../../etc/passwd"))
assert(refused(require, "a\0b"))
-- many missing requires in a row (any per-name cache must stay bounded)
for i = 1, 2000 do pcall(require, "missing" .. i) end
return {}
