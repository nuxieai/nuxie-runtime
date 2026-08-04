//! The [`Scope`] type: lifetime-bounded callbacks and userdata.
//!
//! Mirrors `mlua::Scope`. Constructed by [`Lua::scope`], a `Scope` lets you
//! create Lua callbacks and userdata that borrow **non-`'static`** data from the
//! enclosing stack frame. When the `scope` call returns (normally, via `?`, or
//! through a panic), every object the scope created is *invalidated*: its boxed
//! Rust closure / wrapped data is dropped (ending the borrows) and the
//! underlying Lua object is neutralised so any later use from Lua errors with
//! [`Error::CallbackDestructed`] / [`Error::UserDataDestructed`] instead of
//! touching freed memory.
//!
//! ## The soundness argument (the unsafe core)
//!
//! [`Scope::create_function`] accepts a closure `F: Fn(&Lua, A) -> Result<R> +
//! 'scope` — i.e. **not** `'static`. luaur-rt's normal callback machinery
//! ([`create_callback_function`]) stores a `'static`
//! [`BoxedCallback`](crate::callback). To bridge the gap we
//! [`mem::transmute`] the closure's lifetime to `'static` before boxing it.
//!
//! That transmute is, in isolation, unsound: it lets a closure borrowing
//! `'scope` data be stored where Lua believes it lives forever, and Lua may keep
//! the resulting [`Function`] handle past the end of `'scope` (e.g. stored in a
//! global). It is made sound by the **scope-exit invariant**:
//!
//! 1. On scope exit, *before* returning to the caller (and therefore before any
//!    `'scope`-borrowed data can be dropped), every registered destructor runs.
//! 2. A callback's destructor ([`destruct_callback`]) overwrites the boxed
//!    closure inside the function's upvalue with a sentinel that returns
//!    [`Error::CallbackDestructed`], and **drops the original box** — ending the
//!    borrows right there. The Lua function object itself stays valid; only its
//!    behavior changes. A post-scope call therefore hits the sentinel and
//!    surfaces as `CallbackError { cause: CallbackDestructed }`, never a
//!    use-after-free.
//! 3. A userdata's destructor `take()`s the wrapped value out of its cell
//!    (dropping the borrowed data) while leaving the cell memory valid; later
//!    dispatch finds `None` and returns [`Error::UserDataDestructed`].
//!
//! Because `'scope` outlives nothing the closures borrow until *after* the
//! destructors have run, no borrow can dangle. The destructors are run by a
//! drop guard (the `destructors` field's `Drop`), so they execute even if the
//! user closure returns `Err` or panics — preserving the invariant on every
//! exit path.
//!
//! The two-lifetime shape `Scope<'scope, 'env>` mirrors mlua: `'env` is the
//! lifetime of the data borrowed *into* the scope, `'scope` the (shorter)
//! lifetime of the scope itself; `'env: 'scope`. Created objects are bounded by
//! `'scope`, so they cannot escape the `scope` closure.

use std::cell::RefCell;
use std::marker::PhantomData;
use std::mem;

use crate::callback::{create_callback_function, destruct_callback, BoxedCallback};
use crate::error::{Error, Result};
use crate::function::Function;
use crate::multi::MultiValue;
use crate::state::Lua;
use crate::traits::{FromLuaMulti, IntoLuaMulti};
use crate::userdata::{create_scoped_userdata, AnyUserData, UserData};

/// A scope for creating lifetime-bounded Lua callbacks and userdata.
///
/// Mirrors `mlua::Scope`. See the [module docs](self) and [`Lua::scope`] for the
/// full picture, including the soundness argument for the lifetime erasure.
pub struct Scope<'scope, 'env: 'scope> {
    lua: Lua,
    /// Destructors run (in reverse registration order) on scope exit, by the
    /// `Drop` impl on this field. Held in a separate type so its `Drop` fires
    /// regardless of how the `scope` closure exits.
    destructors: Destructors,
    /// Invariance over `'scope` and `'env`, exactly as mlua, so created objects
    /// cannot outlive the scope and the borrowed data cannot be shortened.
    _scope_invariant: PhantomData<&'scope mut &'scope ()>,
    _env_invariant: PhantomData<&'env mut &'env ()>,
}

/// The registered destructors. Wrapped in its own struct so the `Drop` impl runs
/// the destructors even if the user's `scope` closure panics or returns `Err`.
struct Destructors {
    list: RefCell<Vec<Box<dyn FnOnce()>>>,
}

impl Drop for Destructors {
    fn drop(&mut self) {
        // Run destructors in reverse registration order (LIFO), so objects are
        // torn down opposite to how they were built — matching mlua. Each
        // destructor only drops Rust state / neutralises a Lua object and never
        // panics, so a `drain` loop is fine even mid-unwind.
        let mut list = mem::take(&mut *self.list.borrow_mut());
        while let Some(destructor) = list.pop() {
            destructor();
        }
    }
}

impl<'scope, 'env: 'scope> Scope<'scope, 'env> {
    pub(crate) fn new(lua: Lua) -> Self {
        Scope {
            lua,
            destructors: Destructors {
                list: RefCell::new(Vec::new()),
            },
            _scope_invariant: PhantomData,
            _env_invariant: PhantomData,
        }
    }

    /// Wrap a non-`'static` Rust closure into a callable Lua [`Function`] that is
    /// invalidated when the scope ends.
    ///
    /// This is the scoped version of [`Lua::create_function`]: the closure may
    /// borrow data living for `'scope`. See the [module docs](self) for why the
    /// lifetime erasure is sound.
    pub fn create_function<F, A, R>(&'scope self, func: F) -> Result<Function>
    where
        F: Fn(&Lua, A) -> Result<R> + 'scope,
        A: FromLuaMulti,
        R: IntoLuaMulti,
    {
        // The boxed callback borrows `'scope` data; erase the lifetime to the
        // `'static` the callback machinery expects. Sound only because the
        // destructor below drops the box before `'scope` data can die.
        let boxed: Box<dyn Fn(&Lua, MultiValue) -> Result<MultiValue> + 'scope> =
            Box::new(move |lua, args| {
                let a = A::from_lua_multi(args, lua)?;
                let r = func(lua, a)?;
                r.into_lua_multi(lua)
            });
        let boxed: BoxedCallback = unsafe {
            mem::transmute::<
                Box<dyn Fn(&Lua, MultiValue) -> Result<MultiValue> + 'scope>,
                BoxedCallback,
            >(boxed)
        };

        let f = create_callback_function(&self.lua, boxed)?;

        // Register the neutraliser: on scope exit, swap the upvalue's boxed
        // closure for a `CallbackDestructed` sentinel and drop the original.
        let f_for_dtor = f.clone();
        self.destructors
            .list
            .borrow_mut()
            .push(Box::new(move || destruct_callback(&f_for_dtor)));

        Ok(f)
    }

    /// Wrap a non-`'static` mutable Rust closure into a callable Lua [`Function`]
    /// that is invalidated when the scope ends.
    ///
    /// This is the scoped version of `create_function_mut`. The closure is
    /// guarded by a [`RefCell`]; re-entrant calls (the callback triggering Lua
    /// that calls the same callback) surface as a runtime error rather than a
    /// borrow panic, matching mlua's `RecursiveMutCallback` intent.
    pub fn create_function_mut<F, A, R>(&'scope self, func: F) -> Result<Function>
    where
        F: FnMut(&Lua, A) -> Result<R> + 'scope,
        A: FromLuaMulti,
        R: IntoLuaMulti,
    {
        let func = RefCell::new(func);
        self.create_function(move |lua, args| {
            let mut borrow = func
                .try_borrow_mut()
                .map_err(|_| Error::runtime("mutable callback called recursively"))?;
            (borrow)(lua, args)
        })
    }

    /// Create a Lua userdata wrapping a non-`'static` `T: UserData`, invalidated
    /// when the scope ends.
    ///
    /// This is the scoped version of [`Lua::create_userdata`]: `T` need not be
    /// `'static`, so it may borrow `'env` data. The trade-off (matching mlua) is
    /// that the userdata carries no `TypeId`, so the value cannot be read back
    /// out by concrete type from an [`AnyUserData`] handle — only metatable
    /// method/field/meta dispatch is supported. After the scope ends, any access
    /// from Lua errors with [`Error::UserDataDestructed`].
    pub fn create_userdata<T>(&'scope self, data: T) -> Result<AnyUserData>
    where
        T: UserData + 'env,
    {
        // Erase `T`'s `'env` lifetime down to what the userdata machinery needs.
        // Sound because the neutraliser drops `data` on scope exit (see module
        // docs); the userdata never exposes the value back to Rust by type.
        let (ud, neutralise) = create_scoped_userdata(&self.lua, data)?;
        self.destructors.list.borrow_mut().push(neutralise);
        Ok(ud)
    }

    /// Register an arbitrary destructor to run when the scope ends.
    ///
    /// Mirrors `mlua::Scope::add_destructor`. Useful for cleaning up resources
    /// tied to the scope. Destructors run in reverse registration order on every
    /// exit path (normal, `?`, or panic).
    pub fn add_destructor(&'scope self, destructor: impl FnOnce() + 'env) {
        // Erase `'env` to `'static`: the destructor runs before scope return, so
        // any `'env` data it touches is still alive.
        let destructor: Box<dyn FnOnce() + 'env> = Box::new(destructor);
        let destructor: Box<dyn FnOnce()> =
            unsafe { mem::transmute::<Box<dyn FnOnce() + 'env>, Box<dyn FnOnce()>>(destructor) };
        self.destructors.list.borrow_mut().push(destructor);
    }
}

impl Lua {
    /// Create a [`Scope`] in which non-`'static` callbacks and userdata can be
    /// created, borrowing data from the enclosing stack frame.
    ///
    /// Mirrors `mlua::Lua::scope`. Everything the scope creates is invalidated
    /// when this method returns (on every exit path), so the borrows it held are
    /// guaranteed to end before the borrowed data can. See [`Scope`] and the
    /// [`scope` module docs](crate) for the soundness argument.
    ///
    /// ```
    /// use luaur_rt::prelude::*;
    /// use std::cell::Cell;
    ///
    /// let lua = Lua::new();
    /// let counter = Cell::new(0);
    /// lua.scope(|scope| {
    ///     let f = scope.create_function(|_, ()| {
    ///         counter.set(counter.get() + 1);
    ///         Ok(())
    ///     })?;
    ///     f.call::<()>(())?;
    ///     Ok(())
    /// })
    /// .unwrap();
    /// assert_eq!(counter.get(), 1);
    /// ```
    pub fn scope<'env, R>(
        &self,
        f: impl for<'scope> FnOnce(&'scope Scope<'scope, 'env>) -> Result<R>,
    ) -> Result<R> {
        let scope = Scope::new(self.clone());
        // `f` runs; on return (or unwind) `scope` drops, running all destructors
        // via `Destructors::drop` — the invariant that makes the lifetime
        // erasure sound. We materialise the result before `scope` is dropped.
        f(&scope)
    }
}
