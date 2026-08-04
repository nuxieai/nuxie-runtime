// Adapted from mlua (https://github.com/mlua-rs/mlua), MIT License,
// © 2019 Aleksandr Orlenko / mlua authors. See tests/ATTRIBUTION.md.
//
// Ported from mlua's `tests/memory.rs`. luaur-rt implements the Luau-feasible
// GC + memory-limit surface: `Lua::used_memory`, `set_memory_limit`,
// `gc_collect` / `gc_stop` / `gc_restart` / `gc_is_running` / `gc_count` /
// `gc_step` / `gc_inc` / `gc_set_mode`, and the `state::{GcMode, GcIncParams,
// GcGenParams}` types.
//
// DEVIATION: Luau has a single **incremental** GC — it has no generational mode.
// The `GcMode::Generational` arms of mlua's `test_gc_control` are
// `#[cfg(any(feature = "lua55", feature = "lua54"))]` (i.e. not run under Luau
// anyway), so the Luau-active subset is ported verbatim. `test_gc_error`
// (`__gc` metamethod raising during collection) is `#[cfg(lua52/lua53)]` only —
// not applicable to Luau — and is omitted.

use std::sync::Arc;

use luaur_rt::state::{GcIncParams, GcMode};
use luaur_rt::{Error, Lua, Result, UserData};

#[test]
fn test_memory_limit() -> Result<()> {
    let lua = Lua::new();

    let initial_memory = lua.used_memory();
    assert!(
        initial_memory > 0,
        "used_memory reporting is wrong, lua uses memory for stdlib"
    );

    let f = lua
        .load("local t = {}; for i = 1,10000 do t[i] = i end")
        .into_function()?;
    f.call::<()>(()).expect("should trigger no memory limit");

    lua.set_memory_limit(initial_memory + 10000)?;
    match f.call::<()>(()) {
        Err(Error::MemoryError(_)) => {}
        something_else => panic!("did not trigger memory error: {:?}", something_else),
    };

    lua.set_memory_limit(0)?;
    f.call::<()>(()).expect("should trigger no memory limit");

    // Test memory limit during chunk loading
    lua.set_memory_limit(1024)?;
    match lua
        .load("local t = {}; for i = 1,10000 do t[i] = i end")
        .into_function()
    {
        Err(Error::MemoryError(_)) => {}
        _ => panic!("did not trigger memory error"),
    };

    Ok(())
}

#[test]
fn test_memory_limit_thread() -> Result<()> {
    let lua = Lua::new();

    let f = lua
        .load("local t = {}; for i = 1,10000 do t[i] = i end")
        .into_function()?;

    let thread = lua.create_thread(f)?;
    lua.set_memory_limit(lua.used_memory() + 10000)?;
    match thread.resume::<()>(()) {
        Err(Error::MemoryError(_)) => {}
        something_else => panic!("did not trigger memory error: {:?}", something_else),
    };

    Ok(())
}

#[test]
fn test_gc_control() -> Result<()> {
    let lua = Lua::new();
    let globals = lua.globals();

    // Luau is always running an incremental GC.
    assert!(lua.gc_is_running());
    lua.gc_stop();
    assert!(!lua.gc_is_running());
    lua.gc_restart();
    assert!(lua.gc_is_running());

    // `gc_set_mode(Incremental(..))` applies the params and reports the previous
    // (incremental) mode. (mlua's `Generational` arms are gated off for Luau.)
    assert!(matches!(
        lua.gc_set_mode(GcMode::Incremental({
            let p = GcIncParams::default().step_multiplier(100);
            // DEVIATION: mlua's non-luau path uses `.pause(200)`; on Luau the
            // analogous tunable is `.goal(200)`.
            p.goal(200)
        })),
        GcMode::Incremental(_)
    ));

    struct MyUserdata(#[allow(unused)] Arc<()>);
    impl UserData for MyUserdata {}

    let rc = Arc::new(());
    globals.set("userdata", lua.create_userdata(MyUserdata(rc.clone()))?)?;
    // DEVIATION: mlua's `Table::raw_remove(key)` accepts a string key; luaur-rt's
    // `raw_remove` is the array-index (`table.remove`) form, so we clear the
    // string slot with the equivalent `raw_set(key, nil)`.
    globals.raw_set("userdata", luaur_rt::Value::Nil)?;

    assert_eq!(Arc::strong_count(&rc), 2);
    lua.gc_collect()?;
    lua.gc_collect()?;
    assert_eq!(Arc::strong_count(&rc), 1);

    Ok(())
}

#[test]
fn test_gc_step_and_count() -> Result<()> {
    // Extra luaur-rt coverage (the `gc_count`/`gc_step` ops mlua exposes that
    // its `tests/memory.rs` doesn't directly exercise).
    let lua = Lua::new();

    assert!(lua.gc_count() > 0, "gc_count reports KB in use");

    // Allocate some garbage then run a few incremental steps; this should not
    // error and eventually a step completes a cycle.
    lua.load("local t = {}; for i = 1,1000 do t[i] = {} end")
        .exec()?;
    for _ in 0..100 {
        if lua.gc_step()? {
            break;
        }
    }
    lua.gc_collect()?;

    Ok(())
}
