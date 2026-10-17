use std::cell::Cell;
use std::rc::Rc;

use luaur_rt::Lua;

// Lua::app_data_clone returns what app_data_ref(...).map(clone) would, and
// blocks the store against mutation while the clone runs, as the guard does.

#[derive(Clone, Debug, PartialEq)]
struct Shared(Rc<Cell<i64>>);

#[test]
fn clone_matches_a_cloned_borrow() {
    let lua = Lua::new();
    assert_eq!(lua.app_data_clone::<Shared>(), None);
    let value = Shared(Rc::new(Cell::new(4)));
    lua.set_app_data(value.clone());
    let cloned = lua.app_data_clone::<Shared>().expect("present");
    assert!(Rc::ptr_eq(&cloned.0, &value.0));
    assert_eq!(
        Some(cloned),
        lua.app_data_ref::<Shared>().map(|shared| shared.clone())
    );
    // Other types and other VMs stay separate.
    assert_eq!(lua.app_data_clone::<String>(), None);
    assert_eq!(Lua::new().app_data_clone::<Shared>(), None);
    // A borrowed handle (a callback's) sees the same store.
    lua.globals()
        .set(
            "read",
            lua.create_function(|lua, ()| {
                Ok(lua.app_data_clone::<Shared>().map(|shared| shared.0.get()))
            })
            .unwrap(),
        )
        .unwrap();
    assert_eq!(lua.load("return read()").eval::<i64>().unwrap(), 4);
    // Shared borrows coexist; the clone does not hold one afterwards.
    let held = lua.app_data_ref::<Shared>().expect("present");
    assert_eq!(lua.app_data_clone::<Shared>().map(|s| s.0.get()), Some(4));
    drop(held);
    assert!(lua.try_set_app_data(Shared(Rc::new(Cell::new(5)))).is_ok());
    assert_eq!(lua.app_data_clone::<Shared>().map(|s| s.0.get()), Some(5));
    assert!(lua.remove_app_data::<Shared>().is_some());
    assert_eq!(lua.app_data_clone::<Shared>(), None);
}

#[test]
fn clone_of_a_mutably_borrowed_value_panics_as_a_borrow_does() {
    let lua = Lua::new();
    lua.set_app_data(Shared(Rc::new(Cell::new(1))));
    let _held = lua.app_data_mut::<Shared>().expect("present");
    let clone = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        lua.app_data_clone::<Shared>()
    }));
    let borrow = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        lua.app_data_ref::<Shared>().map(|shared| shared.clone())
    }));
    let message = |payload: Box<dyn std::any::Any + Send>| {
        payload
            .downcast_ref::<&str>()
            .map(|s| s.to_string())
            .or_else(|| payload.downcast_ref::<String>().cloned())
            .unwrap_or_default()
    };
    assert_eq!(
        message(clone.unwrap_err()),
        message(borrow.unwrap_err())
    );
}

// A Clone that touches the store while it runs: setting fails as it does
// under an outstanding guard.
struct Meddling(Rc<Lua>);

thread_local! {
    static OUTCOME: std::cell::RefCell<Option<Result<Option<i64>, String>>> =
        const { std::cell::RefCell::new(None) };
}

impl Clone for Meddling {
    fn clone(&self) -> Self {
        let set = self.0.try_set_app_data(7_i64).map_err(|e| e.to_string());
        OUTCOME.with(|cell| *cell.borrow_mut() = Some(set));
        Meddling(Rc::clone(&self.0))
    }
}

#[test]
fn mutation_during_a_clone_fails_as_under_a_guard() {
    let lua = Rc::new(Lua::new());
    lua.set_app_data(Meddling(Rc::clone(&lua)));
    let guarded = {
        let _held = lua.app_data_ref::<Meddling>().expect("present");
        lua.try_set_app_data(7_i64).map_err(|e| e.to_string())
    };
    assert!(lua.app_data_clone::<Meddling>().is_some());
    let during_clone = OUTCOME
        .with(|cell| cell.borrow_mut().take())
        .expect("clone ran");
    assert!(guarded.is_err());
    assert_eq!(during_clone, guarded);
    // Neither attempt set anything.
    assert!(lua.app_data_ref::<i64>().is_none());
    // Break the Rc cycle so the VM closes.
    drop(lua.remove_app_data::<Meddling>());
}
