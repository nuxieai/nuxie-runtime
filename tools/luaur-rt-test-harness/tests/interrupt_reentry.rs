use std::cell::Cell;
use std::rc::Rc;

use luaur_rt::{Function, Lua, Result, VmState};

// The interrupt trampoline holds the closure's entry for the call instead of
// taking it out of the map: safepoints reached while it runs still skip it,
// and a closure it installs takes effect at once.

#[test]
fn interrupts_inside_their_own_callback_are_skipped() -> Result<()> {
    let lua = Lua::new();
    let calls = Rc::new(Cell::new(0));
    let depth = Rc::new(Cell::new(0));
    let reentered = Rc::new(Cell::new(false));
    let nested: Function = lua
        .load("return function() local n = 0 for i = 1, 50 do n = n + i end return n end")
        .eval()?;
    let (counter, active, overlap) = (calls.clone(), depth.clone(), reentered.clone());
    lua.set_interrupt(move |_| {
        counter.set(counter.get() + 1);
        active.set(active.get() + 1);
        overlap.set(overlap.get() || active.get() > 1);
        // Lua run from the interrupt reaches safepoints of its own; they do
        // not re-enter this closure.
        let result = if counter.get() == 1 {
            nested.call::<i64>(()).map(|_| VmState::Continue)
        } else {
            Ok(VmState::Continue)
        };
        active.set(active.get() - 1);
        result
    });
    lua.load("local n = 0 for i = 1, 50 do n = n + i end")
        .exec()?;
    assert!(calls.get() > 1);
    assert!(!reentered.get());
    let after = calls.get();
    lua.remove_interrupt();
    lua.load("for i = 1, 50 do end").exec()?;
    assert_eq!(calls.get(), after);
    Ok(())
}

#[test]
fn an_interrupt_may_replace_itself() -> Result<()> {
    let lua = Lua::new();
    let replaced = Rc::new(Cell::new(0));
    let inner = Lua::clone(&lua);
    let seen = replaced.clone();
    lua.set_interrupt(move |_| {
        let seen = seen.clone();
        inner.set_interrupt(move |_| {
            seen.set(seen.get() + 1);
            Ok(VmState::Continue)
        });
        Ok(VmState::Continue)
    });
    lua.load("for i = 1, 100 do end").exec()?;
    assert!(replaced.get() > 0);
    lua.remove_interrupt();
    Ok(())
}
