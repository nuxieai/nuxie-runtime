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

fn registration(name: &'static str, source: &str) -> (Vec<u8>, &'static str) {
    (crate::luau::compile(source.as_bytes()).unwrap(), name)
}

fn register_sources(
    sources: &[(&'static str, &str)],
) -> Vec<(bool, Option<String>)> {
    let compiled: Vec<_> = sources
        .iter()
        .map(|(name, source)| registration(name, source))
        .collect();
    let scripts: Vec<_> = compiled
        .iter()
        .map(|(bytecode, name)| ScriptAssetRegistration {
            name,
            bytecode,
            is_protocol: false,
            missing_dependencies: Vec::new(),
        })
        .collect();
    let vm = ScriptVm::new();
    native_registration::register(&vm, &scripts)
        .into_iter()
        .map(|result| (result.completed, result.error.map(|error| error.to_string())))
        .collect()
}

/// E1 4.2's minimal input: file order [Y, X, P, C] where Y requires C,
/// X requires Y and P requires X. No cycle; all four register.
#[test]
fn an_acyclic_require_chain_registers_without_a_cycle_error() {
    let results = register_sources(&[
        ("Y", "require('C') return {}"),
        ("X", "require('Y') return {}"),
        ("P", "require('X') return {}"),
        ("C", "return {}"),
    ]);
    for (index, (completed, error)) in results.iter().enumerate() {
        assert!(completed, "script {index}: {error:?}");
    }
}

/// E1 4.2's smaller form: [Y fails, X requires Y, P requires X]. Each
/// script keeps its own error instead of a false cycle error.
#[test]
fn a_failed_dependency_keeps_each_scripts_own_error() {
    let results = register_sources(&[
        ("Y", "error('Y failed')"),
        ("X", "require('Y') return {}"),
        ("P", "require('X') return {}"),
    ]);
    for (completed, error) in &results {
        assert!(!completed);
        let error = error.as_deref().unwrap_or_default();
        assert!(!error.contains("cyclic"), "{error}");
    }
    assert!(results[0].1.as_deref().unwrap().contains("Y failed"));
    assert!(
        results[1]
            .1
            .as_deref()
            .unwrap()
            .contains("require could not find a script named Y")
    );
    assert!(
        results[2]
            .1
            .as_deref()
            .unwrap()
            .contains("require could not find a script named X")
    );
}

/// A real cycle keeps today's error text.
#[test]
fn a_real_require_cycle_keeps_the_cycle_error() {
    let results = register_sources(&[
        ("A", "require('B') return {}"),
        ("B", "require('A') return {}"),
    ]);
    for (completed, error) in &results {
        assert!(!completed);
        assert!(
            error.as_deref().unwrap().contains("cyclic script module dependency"),
            "{error:?}"
        );
    }
}
