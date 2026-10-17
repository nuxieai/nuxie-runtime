//! Luau interrupt support. Mirrors `mlua::Lua::set_interrupt` / `VmState`.
//!
//! Luau's VM calls a single global `interrupt` callback at safepoints (loop
//! back-edges, calls/returns, GC). mlua exposes this as `Lua::set_interrupt`,
//! taking a Rust closure that returns a [`VmState`] telling the VM whether to
//! continue or to **yield** the current coroutine.
//!
//! luaur's `lua_callbacks().interrupt` is a plain C function pointer, so we
//! install a fixed trampoline ([`interrupt_trampoline`]) and keep the Rust
//! closure in a thread-local keyed by the VM's *global* pointer (shared by all
//! threads of one `Lua`). The trampoline looks up the closure, runs it with a
//! borrowed [`Lua`], and:
//!
//! * `Ok(VmState::Continue)`  — returns normally; the VM keeps executing.
//! * `Ok(VmState::Yield)`     — calls `lua_break`, which sets the running
//!   thread's status so the VM unwinds back to `lua_resume` (a *yield* at a
//!   yieldable point; ignored otherwise, exactly like upstream Luau).
//! * `Err(e)`                 — raises `e` as a Lua error via `lua_error`.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use crate::app_data::WordMap;

use crate::error::{Error, Result};
use crate::state::Lua;
use crate::sys::*;

/// The action an interrupt callback asks the VM to take. Mirrors
/// `mlua::VmState`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VmState {
    /// Keep executing.
    Continue,
    /// Yield the currently running coroutine (no-op at a non-yieldable point).
    Yield,
}

type InterruptFn = Box<dyn Fn(&Lua) -> Result<VmState> + 'static>;

/// Safepoint counters an interrupt settles without running its closure.
///
/// While `used` is below `limit` and the optional countdown is above zero, a
/// safepoint increments `used`, decrements the countdown and continues: what
/// a closure that counts safepoints against those limits would do, without
/// building a [`Lua`] handle or calling it. Otherwise the closure runs and
/// stays the authority for the exhausted case.
#[derive(Clone)]
pub struct InterruptBudget {
    used: Rc<Cell<usize>>,
    limit: usize,
    countdown: Option<Rc<Cell<u32>>>,
}

impl InterruptBudget {
    /// A budget of safepoints counted up in `used`, free below `limit`.
    pub fn new(used: Rc<Cell<usize>>, limit: usize) -> Self {
        Self {
            used,
            limit,
            countdown: None,
        }
    }

    /// Also count `remaining` down; a safepoint is free only while it is
    /// above zero.
    pub fn with_countdown(mut self, remaining: Rc<Cell<u32>>) -> Self {
        self.countdown = Some(remaining);
        self
    }

    /// Settle one safepoint when both counters allow it.
    fn spend(&self) -> bool {
        let used = self.used.get();
        if used >= self.limit {
            return false;
        }
        if let Some(countdown) = &self.countdown {
            let remaining = countdown.get();
            if remaining == 0 {
                return false;
            }
            countdown.set(remaining - 1);
        }
        self.used.set(used + 1);
        true
    }
}

/// One VM's installed interrupt closure. `running` marks a call in progress:
/// an interrupt reached while the closure runs (re-entrantly, from Lua it
/// executes) is skipped, as when the closure was taken out of the map for the
/// call. A closure installed meanwhile is a new entry that runs at once.
struct InterruptEntry {
    callback: InterruptFn,
    budget: Option<InterruptBudget>,
    running: Cell<bool>,
}

/// This thread's interrupt closures, keyed by the `global_State` pointer
/// (stable for the lifetime of the VM and shared by all of its threads).
#[derive(Default)]
struct Interrupts {
    map: WordMap<*mut core::ffi::c_void, Rc<InterruptEntry>>,
    /// The entry the last safepoint found, checked before the map: a thread
    /// usually runs one VM. Always the map's current entry for its key.
    last: Option<(*mut core::ffi::c_void, Rc<InterruptEntry>)>,
}

impl Interrupts {
    fn insert(&mut self, key: *mut core::ffi::c_void, entry: Rc<InterruptEntry>) {
        self.forget(key);
        self.map.insert(key, entry);
    }

    fn remove(&mut self, key: *mut core::ffi::c_void) {
        self.forget(key);
        self.map.remove(&key);
    }

    fn forget(&mut self, key: *mut core::ffi::c_void) {
        if self.last.as_ref().is_some_and(|(last, _)| *last == key) {
            self.last = None;
        }
    }

    /// The entry for `key`, remembered for the next safepoint.
    fn get(&mut self, key: *mut core::ffi::c_void) -> Option<&Rc<InterruptEntry>> {
        if !self.last.as_ref().is_some_and(|(last, _)| *last == key) {
            let entry = Rc::clone(self.map.get(&key)?);
            self.last = Some((key, entry));
        }
        self.last.as_ref().map(|(_, entry)| entry)
    }
}

thread_local! {
    static INTERRUPTS: RefCell<Interrupts> = RefCell::new(Interrupts::default());
}

/// The `global_State` pointer for `state` — the per-VM key shared by all
/// threads of one `Lua`.
unsafe fn vm_key(state: *mut lua_State) -> *mut core::ffi::c_void {
    unsafe { (*state).global as *mut core::ffi::c_void }
}

impl Lua {
    /// Install an interrupt callback. Mirrors `mlua::Lua::set_interrupt`.
    ///
    /// The callback runs at VM safepoints; returning [`VmState::Yield`] yields
    /// the running coroutine, and returning `Err` raises a Lua error.
    pub fn set_interrupt<F>(&self, callback: F)
    where
        F: Fn(&Lua) -> Result<VmState> + crate::sync::MaybeSend + 'static,
    {
        self.install_interrupt(Box::new(callback), None);
    }

    /// [`Lua::set_interrupt`] for a callback that counts safepoints against
    /// `budget`: safepoints the budget settles do not call it (see
    /// [`InterruptBudget`]). The callback must do exactly what the budget
    /// does while it allows a safepoint.
    pub fn set_interrupt_with_budget<F>(&self, budget: InterruptBudget, callback: F)
    where
        F: Fn(&Lua) -> Result<VmState> + crate::sync::MaybeSend + 'static,
    {
        self.install_interrupt(Box::new(callback), Some(budget));
    }

    fn install_interrupt(&self, callback: InterruptFn, budget: Option<InterruptBudget>) {
        let state = self.state();
        unsafe {
            let key = vm_key(state);
            let entry = Rc::new(InterruptEntry {
                callback,
                budget,
                running: Cell::new(false),
            });
            INTERRUPTS.with(|m| m.borrow_mut().insert(key, entry));
            let cb = lua_callbacks(state);
            (*cb).interrupt = Some(interrupt_trampoline);
        }
    }

    /// Remove a previously installed interrupt callback. Mirrors
    /// `mlua::Lua::remove_interrupt`.
    pub fn remove_interrupt(&self) {
        let state = self.state();
        unsafe {
            let key = vm_key(state);
            INTERRUPTS.with(|m| m.borrow_mut().remove(key));
            let cb = lua_callbacks(state);
            (*cb).interrupt = None;
        }
    }
}

/// Drop this VM's interrupt closure. Called from `LuaInner::drop` so the closure
/// (and anything it captured) is released and the per-VM map entry does not leak
/// one slot per state created. (If a closure captured Lua handles it would pin
/// the VM and this never runs — but the common case captures non-Lua state.)
pub(crate) fn clear_interrupt(state: *mut lua_State) {
    let key = unsafe { vm_key(state) };
    INTERRUPTS.with(|m| m.borrow_mut().remove(key));
}

/// The fixed C trampoline installed as `lua_callbacks().interrupt`.
///
/// `gc` is non-negative only for GC interrupts; mlua ignores GC interrupts in
/// the user callback path, and so do we (return immediately) so the user
/// closure only sees real instruction safepoints.
unsafe fn interrupt_trampoline(
    state: *mut lua_State,
    gc: c_int,
) -> luaur_vm::records::lua_exception::LuaResult<()> {
    if gc >= 0 {
        // GC step interrupt — not surfaced to the user callback.
        return Ok(());
    }
    let key = unsafe { vm_key(state) };
    // A safepoint reached while the closure runs is skipped, and one its
    // budget settles needs no call. Otherwise hold the entry (not the map
    // borrow) for the duration of the call, so a re-entrant
    // `set_interrupt`/`remove_interrupt` from inside the callback can replace
    // the map's entry freely.
    let entry = INTERRUPTS.with(|m| {
        let mut interrupts = m.borrow_mut();
        let entry = interrupts.get(key)?;
        if entry.running.get() || entry.budget.as_ref().is_some_and(InterruptBudget::spend) {
            return None;
        }
        Some(Rc::clone(entry))
    });
    match entry {
        Some(entry) => unsafe { run_interrupt(state, entry) },
        None => Ok(()),
    }
}

/// Run `entry`'s closure for one safepoint. Kept out of line: most
/// safepoints return before it.
#[inline(never)]
unsafe fn run_interrupt(
    state: *mut lua_State,
    entry: Rc<InterruptEntry>,
) -> luaur_vm::records::lua_exception::LuaResult<()> {
    entry.running.set(true);

    let lua = unsafe { Lua::from_borrowed(state) };
    let result = (entry.callback)(&lua);
    entry.running.set(false);

    match result {
        Ok(VmState::Continue) => {}
        Ok(VmState::Yield) => unsafe {
            // Request a yield — but only at a yieldable point. Inside a
            // metamethod / C-call boundary Luau's `lua_break` would raise
            // "attempt to break across metamethod/C-call boundary"; upstream
            // (and mlua) silently ignore the yield request there, so we gate it
            // on `lua_isyieldable` and otherwise just continue.
            if lua_isyieldable(state) != 0 {
                luaur_vm::functions::lua_break::lua_break(state)?;
            }
        },
        Err(e) => unsafe {
            return raise_error(state, &e);
        },
    }
    Ok(())
}

/// Push `e`'s message and return its guest error to the protected VM boundary.
unsafe fn raise_error(
    state: *mut lua_State,
    e: &Error,
) -> luaur_vm::records::lua_exception::LuaResult<()> {
    // Use the bare message for a runtime error (so it round-trips back through
    // `pop_error` as `RuntimeError(msg)` without a doubled "runtime error: "
    // prefix); fall back to the full Display for other error kinds.
    let msg = match e {
        Error::RuntimeError(m) => m.clone(),
        other => other.to_string(),
    };
    unsafe {
        // The interrupt fires at an arbitrary VM safepoint where `L->top` may be
        // flush against the call-info top; make room before pushing so the
        // `api_incr_top` stack invariant in `lua_pushlstring` holds.
        lua_rawcheckstack(state, 1)?;
        lua_pushlstring(state, msg.as_ptr() as *const c_char, msg.len())?;
        lua_error(state)
    }
}

#[cfg(test)]
pub(crate) fn interrupts_len() -> usize {
    INTERRUPTS.with(|m| m.borrow().map.len())
}
