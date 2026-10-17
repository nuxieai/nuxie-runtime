use std::cell::Cell;
use std::rc::Rc;

use luaur_rt::{Function, InterruptBudget, Lua, Result, VmState};

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

// A counted closure behaves the same with and without the budget that
// settles its free safepoints: the same safepoint count, the same failure.
fn counted(lua: &Lua, used: &Rc<Cell<usize>>, limit: usize, with_budget: bool) -> Rc<Cell<usize>> {
    let calls = Rc::new(Cell::new(0));
    let (count, closure_calls) = (used.clone(), calls.clone());
    let callback = move |_: &Lua| {
        closure_calls.set(closure_calls.get() + 1);
        let used = count.get();
        if used >= limit {
            return Err(luaur_rt::Error::runtime("safepoint ceiling"));
        }
        count.set(used + 1);
        Ok(VmState::Continue)
    };
    if with_budget {
        lua.set_interrupt_with_budget(InterruptBudget::new(used.clone(), limit), callback);
    } else {
        lua.set_interrupt(callback);
    }
    calls
}

#[test]
fn a_budget_settles_free_safepoints_like_its_closure() -> Result<()> {
    let source = "local n = 0 for i = 1, 200 do n = n + i end return n";
    for limit in [10_000, 25] {
        let plain = Lua::new();
        let plain_used = Rc::new(Cell::new(0));
        let plain_calls = counted(&plain, &plain_used, limit, false);
        let plain_result = plain.load(source).eval::<i64>().map_err(|e| e.to_string());

        let budgeted = Lua::new();
        let budget_used = Rc::new(Cell::new(0));
        let budget_calls = counted(&budgeted, &budget_used, limit, true);
        let budget_result = budgeted
            .load(source)
            .eval::<i64>()
            .map_err(|e| e.to_string());

        assert_eq!(plain_result, budget_result, "limit {limit}");
        assert_eq!(plain_used.get(), budget_used.get(), "limit {limit}");
        assert!(plain_used.get() > 0);
        // The closure runs only once the budget is spent.
        let expected_calls = if plain_result.is_ok() { 0 } else { 1 };
        assert_eq!(budget_calls.get(), expected_calls, "limit {limit}");
        assert_eq!(plain_calls.get(), plain_used.get() + expected_calls);
    }
    Ok(())
}

#[test]
fn a_countdown_also_bounds_free_safepoints() -> Result<()> {
    let lua = Lua::new();
    let used = Rc::new(Cell::new(0));
    let remaining = Rc::new(Cell::new(5u32));
    let calls = Rc::new(Cell::new(0));
    let (count, left, closure_calls) = (used.clone(), remaining.clone(), calls.clone());
    lua.set_interrupt_with_budget(
        InterruptBudget::new(used.clone(), 10_000).with_countdown(remaining.clone()),
        move |_| {
            closure_calls.set(closure_calls.get() + 1);
            count.set(count.get() + 1);
            if left.get() == 0 {
                return Err(luaur_rt::Error::runtime("countdown spent"));
            }
            left.set(left.get() - 1);
            Ok(VmState::Continue)
        },
    );
    let error = lua
        .load("for i = 1, 200 do end")
        .exec()
        .unwrap_err()
        .to_string();
    assert!(error.contains("countdown spent"), "{error}");
    assert_eq!(remaining.get(), 0);
    assert_eq!(calls.get(), 1);
    assert_eq!(used.get(), 6);
    Ok(())
}
