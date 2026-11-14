//! Lane E2a (`docs/luaur-client-path-safe.md` 12.2): recursion and hangs a
//! script can reach. Each test reproduces the E1 inventory's input
//! (`docs/evidence/luaur-client-path-lane-e1-inventory-2026-11-14.md`).

use super::*;
use crate::luau::CallbackCause;

/// E1 4.1's starter was a scoped `xpcall` handler, obtained through
/// `debug.info` and called after its scope ended; that no longer yields an
/// error object (lane D3, `xpcall_handler_kept_past_its_xpcall_*`). Here the
/// structured error comes from a host function the test installs
/// (`structuredError`, which fails with `CallbackDestructed`), so the chain
/// test keeps its input. `E1VICTIM`'s `__tostring` re-raises whatever
/// `E1OBJ` holds.
const ERROR_CHAIN_STARTER: &str = r#"
local ok, err = pcall(structuredError)
assert(not ok and typeof(err) == "userdata", "no structured starter")
E1OBJ = err
E1VICTIM = setmetatable({}, { __tostring = function() error(E1OBJ) end })
"#;

/// The script half of E1 4.1's old starter: keep every function on the
/// handler's stack and call each after `xpcall` returns.
const XPCALL_HANDLER_KEPT: &str = r#"
local captured = {}
xpcall(function() error("x") end, function(e)
  for level = 1, 8 do
    local ok, f = pcall(debug.info, level, "f")
    -- Host frames only: a Luau frame here is this chunk or its body.
    if ok and f and debug.info(f, "s") == "[C]" then table.insert(captured, f) end
  end
  return e
end)
local errors = {}
for _, f in ipairs(captured) do
  local ok, err = pcall(f, "probe")
  if not ok then table.insert(errors, err) end
end
return errors
"#;

fn install_structured_error(vm: &ScriptVm) -> Result<()> {
    let structured = vm
        .lua
        .create_function(|_, ()| -> Result<()> { Err(Error::CallbackDestructed) })?;
    vm.lua.globals().set("structuredError", structured)
}

/// Lane D3: the guarded `xpcall` handler is one host function made at
/// install, not a scoped one, so a script that keeps it past its `xpcall`
/// gets an ordinary error string, never a destructed-callback error object.
#[test]
fn xpcall_handler_kept_past_its_xpcall_raises_an_ordinary_error() -> Result<()> {
    let vm = ScriptVm::new();
    vm.install_rive_globals()?;
    let errors: Table = vm.lua.load(XPCALL_HANDLER_KEPT).eval()?;
    let errors: Vec<Value> = errors.sequence_values().collect::<Result<_>>()?;
    assert!(
        errors.iter().all(|error| matches!(error, Value::String(_))),
        "every kept function fails with a string"
    );
    let texts: Vec<String> = errors
        .iter()
        .filter_map(|error| match error {
            Value::String(text) => text.to_str().ok(),
            _ => None,
        })
        .collect();
    assert!(
        texts
            .iter()
            .any(|text| text.contains("xpcall error handler called outside its xpcall")),
        "{texts:?}"
    );
    Ok(())
}

/// The guard still runs the script's handler, nested xpcalls each reach
/// their own, and the handler's frames are unchanged: level 2 is a host
/// function, as the per-call handler was.
#[test]
fn guarded_xpcall_handlers_nest_and_stay_host_frames() -> Result<()> {
    let vm = ScriptVm::new();
    vm.install_rive_globals()?;
    let (outer, what): (String, String) = vm
        .lua
        .load(
            r#"
            local what
            local _, outer = xpcall(function()
              local _, inner = xpcall(function() error("inner", 0) end, function(e)
                what = debug.info(2, "s")
                return "handled " .. e
              end)
              error(inner, 0)
            end, function(e) return "outer saw " .. e end)
            return outer, what
            "#,
        )
        .eval()?;
    assert_eq!(outer, "outer saw handled inner");
    assert_eq!(what, "[C]");
    Ok(())
}

/// A guard kept from one xpcall belongs to that xpcall: called during a
/// later, unrelated xpcall it fails rather than reaching that xpcall's
/// handler, and an outer guard re-entered during a nested xpcall reaches the
/// outer handler, not the inner one.
#[test]
fn a_kept_xpcall_guard_reaches_only_its_own_handler() -> Result<()> {
    let vm = ScriptVm::new();
    vm.install_rive_globals()?;
    let (later, nested): (String, String) = vm
        .lua
        .load(
            r#"
            local kept
            xpcall(function() error("first", 0) end, function(e)
              kept = debug.info(2, "f")
              return e
            end)
            local later
            xpcall(function()
              local ok, result = pcall(kept, "probe")
              later = tostring(ok) .. ":" .. tostring(result)
              error("later", 0)
            end, function(e) return "unrelated:" .. e end)

            local nested
            local depth = 0
            xpcall(function() error("outer-body", 0) end, function(e)
              depth += 1
              if depth > 1 then return "outer:" .. e end
              local outerGuard = debug.info(2, "f")
              xpcall(function() error("inner-body", 0) end, function(inner)
                local ok, result = pcall(outerGuard, "reentered")
                nested = tostring(ok) .. ":" .. tostring(result)
                return inner
              end)
              return e
            end)
            return later, nested
            "#,
        )
        .eval()?;
    assert!(
        later.starts_with("false:")
            && later.contains("xpcall error handler called outside its xpcall"),
        "{later}"
    );
    assert_eq!(nested, "true:outer:reentered");
    Ok(())
}

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
    install_structured_error(&vm)?;
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

/// E1 section 3: a script that re-enters its own VM through a host entry in
/// a loop. The nested entry used to reset the cycle's safepoints, so the
/// loop never reached the cap.
#[test]
fn reentering_the_vm_in_a_loop_reaches_the_safepoint_cap() -> Result<()> {
    let vm = Rc::new(ScriptVm::new());
    ScriptVm::install_rive_globals(&vm)?;
    assert!(!script_running(&vm.lua), "no script runs at the host's top level");
    let entries = Rc::new(Cell::new(0usize));
    let reenter = {
        let vm = Rc::downgrade(&vm);
        let entries = Rc::clone(&entries);
        vm.upgrade()
            .unwrap()
            .lua
            .create_function(move |_, ()| {
                let count = entries.get() + 1;
                entries.set(count);
                if count > 2 * SCRIPT_SAFEPOINTS_PER_CYCLE {
                    return Err(Error::runtime("re-entry never reached the safepoint cap"));
                }
                let vm = vm.upgrade().ok_or_else(|| Error::runtime("no VM"))?;
                vm.call_global::<()>("inner", ())
            })?
    };
    vm.lua.globals().set("reenter", reenter)?;
    vm.lua
        .load(
            "function inner() end \
             function outer() while true do reenter() end end \
             function short() for i = 1, 1000 do end end",
        )
        .exec()?;
    let error = vm.call_global::<()>("outer", ()).unwrap_err().to_string();
    assert!(
        error.contains(&format!("{SCRIPT_SAFEPOINTS_PER_CYCLE} script safepoints")),
        "{error} after {} entries",
        entries.get()
    );
    assert!(entries.get() < SCRIPT_SAFEPOINTS_PER_CYCLE);

    // The nested entries' guards restored the depth on their way out, also
    // on the error path: the next outermost entry starts a fresh cycle.
    vm.call_global::<()>("short", ())?;
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

/// Runs `source` on a fresh VM on its own thread and fails if it does not
/// finish within `seconds` (the old code looped up to `i32::MAX` times; the
/// thread is left behind and ends with the test process).
fn finishes_within(seconds: u64, source: &'static str) -> std::result::Result<(), String> {
    let (sender, receiver) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let vm = ScriptVm::new();
        let outcome = vm
            .install_rive_globals()
            .and_then(|()| vm.lua.load(source).exec())
            .map_err(|error| error.to_string());
        let _ = sender.send(outcome);
    });
    receiver
        .recv_timeout(std::time::Duration::from_secs(seconds))
        .map_err(|_| format!("did not finish within {seconds} s"))?
}

/// E1 section 3: `buffer.stridedcopy` with zero strides (so a zero element
/// size) and `count = i32::MAX`, source and destination the same buffer.
#[test]
fn a_zero_stride_strided_copy_is_bounded() {
    finishes_within(
        10,
        "local b = buffer.create(4096) buffer.stridedcopy(b, 0, 0, b, 0, 0, 0, 2147483647)",
    )
    .unwrap();
}

/// `buffer.convert` with both strides zero and `count = i32::MAX`, apart and
/// aliased.
#[test]
fn a_zero_stride_convert_is_bounded() {
    finishes_within(
        10,
        r#"
local a, b = buffer.create(64), buffer.create(64)
for i = 0, 63 do buffer.writeu8(a, i, i * 3) end
buffer.convert(b, 0, "u16", a, 0, "u8", 2147483647, 4, 0, 0)
assert(buffer.readu16(b, 6) == 9)
buffer.convert(a, 1, "u8", a, 0, "u8", 2147483647, 4, 0, 0)
"#,
    )
    .unwrap();
}
