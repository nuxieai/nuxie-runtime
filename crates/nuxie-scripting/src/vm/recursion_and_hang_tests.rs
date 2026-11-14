//! Lane E2a (`docs/luaur-client-path-safe.md` 12.2): recursion and hangs a
//! script can reach. Each test reproduces the E1 inventory's input
//! (`docs/evidence/luaur-client-path-lane-e1-inventory-2026-11-14.md`).

use super::*;
use crate::luau::CallbackCause;

/// E1 4.1's starter: a scoped `xpcall` handler, obtained through
/// `debug.info` and called after its scope ended, raises
/// `CallbackDestructed` as an error object; `E1VICTIM`'s `__tostring`
/// re-raises whatever `E1OBJ` holds.
const ERROR_CHAIN_STARTER: &str = r#"
local captured = {}
xpcall(function() error("x") end, function(e)
  for level = 1, 8 do
    local ok, f = pcall(debug.info, level, "f")
    if ok and f then table.insert(captured, f) end
  end
  return e
end)
for _, f in ipairs(captured) do
  local ok, err = pcall(f, "probe")
  if not ok and typeof(err) == "userdata" then E1OBJ = err break end
end
assert(E1OBJ ~= nil, "no structured starter")
E1VICTIM = setmetatable({}, { __tostring = function() error(E1OBJ) end })
"#;

/// One round: `print` calls `tostring`, which fails with the object; the
/// host callback's error comes back as a new object.
const ERROR_CHAIN_ROUNDS: &str = r#"
for i = 1, 500 do
  local ok, e = pcall(print, E1VICTIM)
  assert(not ok)
  E1OBJ = e
end
"#;

/// E1 4.1: a host callback that relays a script's structured error used to
/// add one `CallbackError` layer per round; `Display`, `is_structured` and
/// `Drop` recursed once per layer and overflowed the stack at a few
/// thousand. 3,000 rounds here (the old code built 3,002 layers). A
/// callback error's cause is a `CallbackCause`, so one layer is all the type
/// can hold; the test pins the cause and the text.
#[test]
fn relaying_a_structured_error_keeps_one_layer() -> Result<()> {
    let vm = ScriptVm::new();
    vm.install_rive_globals()?;
    vm.lua.load(ERROR_CHAIN_STARTER).exec()?;
    vm.lua
        .load("local ok, e = pcall(print, E1VICTIM) assert(not ok) E1OBJ = e")
        .exec()?;
    let first = vm.lua.load("error(E1OBJ)").exec().unwrap_err();
    for _ in 0..6 {
        vm.lua.load(ERROR_CHAIN_ROUNDS).exec()?;
    }
    let relayed = vm.lua.load("error(E1OBJ)").exec().unwrap_err();
    for error in [&first, &relayed] {
        assert!(
            matches!(
                error,
                Error::CallbackError {
                    cause: CallbackCause::CallbackDestructed,
                    ..
                }
            ),
            "{error}"
        );
    }
    assert_eq!(first.to_string(), relayed.to_string());
    assert_eq!(
        relayed.to_string(),
        "a destructed callback or destructed userdata method was called"
    );
    Ok(())
}
