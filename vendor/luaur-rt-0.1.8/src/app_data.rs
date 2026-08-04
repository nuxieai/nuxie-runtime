//! Per-VM **application data** — a typed, borrow-checked side store keyed by
//! Rust `TypeId`. Mirrors `mlua::Lua`'s app-data surface (`set_app_data`,
//! `app_data_ref`, `app_data_mut`, `remove_app_data`, and the `try_*` variants).
//!
//! Each `Lua` instance has its own store. Values are kept behind a per-entry
//! [`RefCell`], so a `&T` ([`AppDataRef`]) and a `&mut T` ([`AppDataRefMut`])
//! of **different** types can coexist (the usual aliasing rules apply only
//! within a single type). A VM-wide borrow counter (matching mlua's `AppData`)
//! additionally makes *any* outstanding borrow block `set_app_data` /
//! `remove_app_data` of *any* type, so the container is never mutated while a
//! guard is live.
//!
//! The store lives in a thread-local map keyed by the VM's global-state pointer
//! (the same pattern `luau_ext` uses for the per-VM compiler), since `LuaInner`
//! itself is shared immutably behind an `XRc`.

use std::any::{Any, TypeId};
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
use std::rc::Rc;

use crate::error::{Error, Result};
use crate::state::Lua;
use crate::sys::lua_State;

/// One entry: a value behind a `RefCell` so per-type borrows can be tracked,
/// shared via `Rc` so a returned guard can outlive a borrow of the outer map.
type Entry = Rc<RefCell<Box<dyn Any>>>;

/// Per-VM store: the entries plus a VM-wide outstanding-borrow counter (shared
/// via `Rc<Cell<usize>>` so a live guard can decrement it on drop).
#[derive(Default)]
struct Store {
    entries: HashMap<TypeId, Entry>,
    borrow: Rc<Cell<usize>>,
}

thread_local! {
    /// Per-VM application-data store, keyed by global-state pointer.
    static APP_DATA: RefCell<HashMap<*mut core::ffi::c_void, Store>> =
        RefCell::new(HashMap::new());
}

unsafe fn vm_key(state: *mut lua_State) -> *mut core::ffi::c_void {
    unsafe { (*state).global as *mut core::ffi::c_void }
}

/// An immutable borrow of an application-data value of type `T`. Mirrors
/// `mlua::AppDataRef`.
pub struct AppDataRef<T: 'static> {
    // Hold the `Rc<RefCell>` alive and the `Ref` borrow open for as long as the
    // guard lives. The `'static` `Ref` is sound because the `_owner` `Rc` we
    // also hold keeps the `RefCell` alive (both drop together).
    _owner: Entry,
    guard: std::cell::Ref<'static, Box<dyn Any>>,
    borrow: Rc<Cell<usize>>,
    _marker: std::marker::PhantomData<T>,
}

impl<T: 'static> Drop for AppDataRef<T> {
    fn drop(&mut self) {
        self.borrow.set(self.borrow.get().saturating_sub(1));
    }
}

impl<T: 'static> Deref for AppDataRef<T> {
    type Target = T;
    fn deref(&self) -> &T {
        self.guard
            .downcast_ref::<T>()
            .expect("app data type mismatch")
    }
}

impl<T: std::fmt::Debug + 'static> std::fmt::Debug for AppDataRef<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        (**self).fmt(f)
    }
}

impl<T: std::fmt::Display + 'static> std::fmt::Display for AppDataRef<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        (**self).fmt(f)
    }
}

impl<T: PartialEq + 'static> PartialEq<T> for AppDataRef<T> {
    fn eq(&self, other: &T) -> bool {
        (**self) == *other
    }
}

/// A mutable borrow of an application-data value of type `T`. Mirrors
/// `mlua::AppDataRefMut`.
pub struct AppDataRefMut<T: 'static> {
    _owner: Entry,
    guard: std::cell::RefMut<'static, Box<dyn Any>>,
    borrow: Rc<Cell<usize>>,
    _marker: std::marker::PhantomData<T>,
}

impl<T: 'static> Drop for AppDataRefMut<T> {
    fn drop(&mut self) {
        self.borrow.set(self.borrow.get().saturating_sub(1));
    }
}

impl<T: 'static> Deref for AppDataRefMut<T> {
    type Target = T;
    fn deref(&self) -> &T {
        self.guard
            .downcast_ref::<T>()
            .expect("app data type mismatch")
    }
}

impl<T: 'static> DerefMut for AppDataRefMut<T> {
    fn deref_mut(&mut self) -> &mut T {
        self.guard
            .downcast_mut::<T>()
            .expect("app data type mismatch")
    }
}

impl<T: std::fmt::Debug + 'static> std::fmt::Debug for AppDataRefMut<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        (**self).fmt(f)
    }
}

impl<T: PartialEq + 'static> PartialEq<T> for AppDataRefMut<T> {
    fn eq(&self, other: &T) -> bool {
        (**self) == *other
    }
}

impl Lua {
    /// Insert (or replace) a value of type `T` in this VM's application-data
    /// store. Mirrors `mlua::Lua::set_app_data`.
    ///
    /// # Panics
    /// Panics if **any** app-data value is currently borrowed.
    pub fn set_app_data<T: 'static>(&self, data: T) {
        self.try_set_app_data(data)
            .expect("cannot mutably borrow app data container");
    }

    /// Try to insert (or replace) a value of type `T`. Returns the previous
    /// value, or an error if any app-data value is currently borrowed. Mirrors
    /// `mlua::Lua::try_set_app_data`.
    pub fn try_set_app_data<T: 'static>(&self, data: T) -> Result<Option<T>> {
        let key = unsafe { vm_key(self.state()) };
        APP_DATA.with(|m| {
            let mut outer = m.borrow_mut();
            let store = outer.entry(key).or_default();
            // Any outstanding borrow blocks mutation of the container (mlua).
            if store.borrow.get() != 0 {
                return Err(Error::runtime("cannot mutably borrow app data container"));
            }
            let old = store
                .entries
                .insert(TypeId::of::<T>(), Rc::new(RefCell::new(Box::new(data))));
            Ok(old.and_then(|e| {
                Rc::try_unwrap(e)
                    .ok()
                    .and_then(|cell| cell.into_inner().downcast::<T>().ok().map(|b| *b))
            }))
        })
    }

    /// Borrow the application-data value of type `T` immutably, if present.
    /// Mirrors `mlua::Lua::app_data_ref`.
    ///
    /// # Panics
    /// Panics if the value is currently mutably borrowed.
    pub fn app_data_ref<T: 'static>(&self) -> Option<AppDataRef<T>> {
        match self.try_app_data_ref::<T>() {
            Ok(opt) => opt,
            Err(_) => panic!("already mutably borrowed"),
        }
    }

    /// Try to borrow the application-data value of type `T` immutably. Returns
    /// `Ok(None)` if absent, `Err` if it is currently mutably borrowed. Mirrors
    /// `mlua::Lua::try_app_data_ref`.
    pub fn try_app_data_ref<T: 'static>(&self) -> Result<Option<AppDataRef<T>>> {
        let key = unsafe { vm_key(self.state()) };
        let (entry, borrow) = match APP_DATA.with(|m| {
            let outer = m.borrow();
            outer.get(&key).and_then(|store| {
                store
                    .entries
                    .get(&TypeId::of::<T>())
                    .map(|e| (e.clone(), store.borrow.clone()))
            })
        }) {
            Some(pair) => pair,
            None => return Ok(None),
        };
        let guard = entry
            .try_borrow()
            .map_err(|_| Error::runtime("app data is currently mutably borrowed"))?;
        // Extend the borrow lifetime to `'static`; the `_owner` `Rc` we keep
        // alongside keeps the `RefCell` alive for as long as the guard lives.
        let guard: std::cell::Ref<'static, Box<dyn Any>> = unsafe { std::mem::transmute(guard) };
        borrow.set(borrow.get() + 1);
        Ok(Some(AppDataRef {
            _owner: entry,
            guard,
            borrow,
            _marker: std::marker::PhantomData,
        }))
    }

    /// Borrow the application-data value of type `T` mutably, if present.
    /// Mirrors `mlua::Lua::app_data_mut`.
    ///
    /// # Panics
    /// Panics if the value is currently borrowed (immutably or mutably).
    pub fn app_data_mut<T: 'static>(&self) -> Option<AppDataRefMut<T>> {
        match self.try_app_data_mut::<T>() {
            Ok(opt) => opt,
            Err(_) => panic!("already borrowed"),
        }
    }

    /// Try to borrow the application-data value of type `T` mutably. Returns
    /// `Ok(None)` if absent, `Err` if it is currently borrowed. Mirrors
    /// `mlua::Lua::try_app_data_mut`.
    pub fn try_app_data_mut<T: 'static>(&self) -> Result<Option<AppDataRefMut<T>>> {
        let key = unsafe { vm_key(self.state()) };
        let (entry, borrow) = match APP_DATA.with(|m| {
            let outer = m.borrow();
            outer.get(&key).and_then(|store| {
                store
                    .entries
                    .get(&TypeId::of::<T>())
                    .map(|e| (e.clone(), store.borrow.clone()))
            })
        }) {
            Some(pair) => pair,
            None => return Ok(None),
        };
        let guard = entry
            .try_borrow_mut()
            .map_err(|_| Error::runtime("app data is currently borrowed"))?;
        let guard: std::cell::RefMut<'static, Box<dyn Any>> = unsafe { std::mem::transmute(guard) };
        borrow.set(borrow.get() + 1);
        Ok(Some(AppDataRefMut {
            _owner: entry,
            guard,
            borrow,
            _marker: std::marker::PhantomData,
        }))
    }

    /// Remove and return the application-data value of type `T`, if present.
    /// Mirrors `mlua::Lua::remove_app_data`.
    ///
    /// # Panics
    /// Panics if **any** app-data value is currently borrowed.
    pub fn remove_app_data<T: 'static>(&self) -> Option<T> {
        let key = unsafe { vm_key(self.state()) };
        APP_DATA.with(|m| {
            let mut outer = m.borrow_mut();
            let store = outer.get_mut(&key)?;
            if store.borrow.get() != 0 {
                panic!("cannot mutably borrow app data container");
            }
            let entry = store.entries.remove(&TypeId::of::<T>())?;
            Rc::try_unwrap(entry)
                .ok()
                .and_then(|cell| cell.into_inner().downcast::<T>().ok().map(|b| *b))
        })
    }
}

/// Drop this VM's entire application-data store. Called from `LuaInner::drop`.
pub(crate) fn clear_app_data(state: *mut lua_State) {
    let key = unsafe { vm_key(state) };
    APP_DATA.with(|m| {
        m.borrow_mut().remove(&key);
    });
}
