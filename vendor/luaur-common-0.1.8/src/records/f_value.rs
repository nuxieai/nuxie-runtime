//! Faithful port of Luau's FastFlag value `FValue<T>`. Reference:
//! `luau/Common/include/Luau/Common.h` (the `FValue<T>` template, the
//! `LUAU_FASTFLAG*` macros, `FValueVersionSetter`) and the list walkers in
//! `luau/CLI/src/Flags.cpp`. Oracle: `/tmp/fastflag_proto.rs` (reads, enumerate,
//! runtime-set, version-set-by-name, unknown-flag guard — all pass).
//!
//! Flags read as their `value` (the C++ `operator T()`, see [`FValue::get`]); a
//! per-type intrusive list lets the host enumerate flags and set them / their
//! version by name.
//!
//! Deviations from C++ (documented, behavior-faithful):
//! - The per-`T` `static FValue<T>* list` is inexpressible in Rust (no generic
//!   statics), so the head is supplied by the [`FValueList`] trait, implemented
//!   for exactly the instantiated types (`bool`, `i32`).
//! - The C++ ctor self-registers (`list = this`). Rust statics are
//!   const-initialized with no ctor side effects, so registration is the explicit
//!   [`FValue::register`], called once on a flag's `'static` instance. Reading a
//!   flag does NOT require registration — only enumeration / set-by-name does.
//! - Public mutable fields become `UnsafeCell` + `unsafe impl Sync`, matching the
//!   C++ contract (flags configured before worker threads start, read-only after;
//!   C++ has no synchronization here either).
//!
//! Downstream `LUAU_FASTFLAGVARIABLE(Foo)` becomes a
//! `static FOO: FValue<bool> = FValue::new(c"Foo", false, false);` registered at
//! startup; `FFlag::Foo` reads become `FOO.get()`.

use core::cell::UnsafeCell;
use core::ffi::{c_char, c_uint};
use core::sync::atomic::{AtomicBool, AtomicPtr, Ordering};

// ---------------------------------------------------------------------------
// Thread-local flag overrides (test isolation)
//
// C++ doctest runs single-threaded, so `ScopedFastFlag`/`ScopedFastInt` (which
// mutate a process-global flag) are safe. Rust's libtest runs tests in PARALLEL
// threads, so a global mutation by one test leaks into another reading the same
// flag — producing nondeterministic failures (and, when a recursion/length
// LIMIT leaks, runaway recursion that overflows a test thread's stack).
//
// Fix: a per-thread override layer. A scoped guard pushes its value onto a
// thread-local stack for that flag; `get()` returns the current thread's
// override when present, else the global. Mutations are thus private to the
// thread (and the scope) that made them — parallel tests no longer interfere,
// and the production path is unchanged.
//
// Cost in production: `get()` does one relaxed atomic load of `OVERRIDES_ACTIVE`
// (set the first time any scoped guard runs — i.e. never, outside tests) before
// the normal read. No cargo feature needed; the flag stays false in real runs.
// ---------------------------------------------------------------------------

/// Set true the first time a scoped override is pushed. Until then `get()` skips
/// the thread-local lookup entirely.
static OVERRIDES_ACTIVE: AtomicBool = AtomicBool::new(false);

/// Has any thread-local override ever been installed in this process?
#[inline]
pub fn overrides_active() -> bool {
    OVERRIDES_ACTIVE.load(Ordering::Relaxed)
}

/// Per-type thread-local override stacks, keyed by the flag's `'static` address.
/// A `Vec` (stack) so nested scopes restore correctly and the entry is removed
/// when the outermost scope ends (no stale leak across tests reusing a thread).
pub trait FValueOverridable: Copy {
    fn with_overrides<R>(
        f: impl FnOnce(&mut std::collections::HashMap<usize, Vec<Self>>) -> R,
    ) -> R;

    fn override_top(addr: usize) -> Option<Self> {
        Self::with_overrides(|m| m.get(&addr).and_then(|s| s.last().copied()))
    }
    fn override_push(addr: usize, value: Self) {
        OVERRIDES_ACTIVE.store(true, Ordering::Relaxed);
        Self::with_overrides(|m| m.entry(addr).or_default().push(value));
    }
    fn override_pop(addr: usize) {
        Self::with_overrides(|m| {
            if let Some(stack) = m.get_mut(&addr) {
                stack.pop();
                if stack.is_empty() {
                    m.remove(&addr);
                }
            }
        });
    }
}

thread_local! {
    static BOOL_OVERRIDES: core::cell::RefCell<std::collections::HashMap<usize, Vec<bool>>> =
        core::cell::RefCell::new(std::collections::HashMap::new());
    static INT_OVERRIDES: core::cell::RefCell<std::collections::HashMap<usize, Vec<i32>>> =
        core::cell::RefCell::new(std::collections::HashMap::new());
}

impl FValueOverridable for bool {
    fn with_overrides<R>(
        f: impl FnOnce(&mut std::collections::HashMap<usize, Vec<Self>>) -> R,
    ) -> R {
        BOOL_OVERRIDES.with(|c| f(&mut c.borrow_mut()))
    }
}

impl FValueOverridable for i32 {
    fn with_overrides<R>(
        f: impl FnOnce(&mut std::collections::HashMap<usize, Vec<Self>>) -> R,
    ) -> R {
        INT_OVERRIDES.with(|c| f(&mut c.borrow_mut()))
    }
}

impl<T: FValueOverridable> FValue<T> {
    /// Install a thread-local override for this flag (used by the test scope
    /// guard `ScopedFValue`). Visible only to the current thread until popped.
    pub fn push_test_override(&self, value: T) {
        T::override_push(self as *const FValue<T> as usize, value);
    }

    /// Remove the most recent thread-local override for this flag.
    pub fn pop_test_override(&self) {
        T::override_pop(self as *const FValue<T> as usize);
    }
}

pub struct FValue<T> {
    pub(crate) value: UnsafeCell<T>,
    pub(crate) dynamic: bool,
    pub(crate) name: *const c_char,
    pub(crate) next: UnsafeCell<*const FValue<T>>,
    pub(crate) version: UnsafeCell<c_uint>,
}

// See the module deviation note: flags are configured before threads start and
// treated as read-only afterwards.
unsafe impl<T: Sync> Sync for FValue<T> {}

/// Supplies the per-type intrusive-list head, replacing the inexpressible C++
/// `static FValue<T>* list`. Implemented for exactly the instantiated types.

/// C++ `setLuauFlagsDefault(bool)` analog (CLI default-on behavior): walk the
/// bool-flag registry and set every non-Debug flag. Call before threads start.
pub fn set_luau_bool_flags(value: bool) {
    unsafe {
        let mut cur = <bool as FValueList>::head().load(Ordering::Relaxed) as *const FValue<bool>;
        while !cur.is_null() {
            let name = core::ffi::CStr::from_ptr((*cur).name);
            if !name.to_bytes().starts_with(b"Debug") {
                *(*cur).value.get() = value;
            }
            cur = *(*cur).next.get();
        }
    }
}

pub trait FValueList: Sized {
    fn head() -> &'static AtomicPtr<FValue<Self>>;
}

static FVALUE_LIST_BOOL: AtomicPtr<FValue<bool>> = AtomicPtr::new(core::ptr::null_mut());
impl FValueList for bool {
    fn head() -> &'static AtomicPtr<FValue<bool>> {
        &FVALUE_LIST_BOOL
    }
}

static FVALUE_LIST_INT: AtomicPtr<FValue<i32>> = AtomicPtr::new(core::ptr::null_mut());
impl FValueList for i32 {
    fn head() -> &'static AtomicPtr<FValue<i32>> {
        &FVALUE_LIST_INT
    }
}

impl<T: Copy> FValue<T> {
    /// Runtime flag set (the CLI/host path mutates the public `value` field).
    pub fn set(&self, value: T) {
        unsafe { *self.value.get() = value };
    }

    /// Current `version` (0 unless a `LUAU_FLAGVERSION` setter ran).
    pub fn version(&self) -> c_uint {
        unsafe { *self.version.get() }
    }

    pub(crate) fn set_version(&self, version: c_uint) {
        unsafe { *self.version.get() = version };
    }
}

impl<T: FValueList> FValue<T> {
    /// The C++ ctor side effect `next = list; list = this;`. Call once, on the
    /// flag's `'static` instance, after construction.
    ///
    /// # Safety
    /// Must be called at most once per flag, before any concurrent list walk
    /// (registration is single-threaded startup work, as in C++).
    pub unsafe fn register(&'static self) {
        let head = T::head();
        let old = head.load(Ordering::Relaxed);
        *self.next.get() = old as *const FValue<T>;
        head.store(
            self as *const FValue<T> as *mut FValue<T>,
            Ordering::Relaxed,
        );
    }
}

impl<T: FValueList + Copy + 'static> FValue<T> {
    /// Walk the per-type `FValue<T>::list` and set every flag whose `name`
    /// matches to `value` (the C++ test harness `setFastValue<T>(name, value)`
    /// in `tests/main.cpp`). Configured at startup before worker threads — the
    /// same single-threaded contract as flag construction.
    pub fn set_value_by_name(name: &str, value: T) {
        unsafe {
            let mut cur = T::head().load(Ordering::Relaxed) as *const FValue<T>;
            while !cur.is_null() {
                let fvalue = &*cur;
                if core::ffi::CStr::from_ptr(fvalue.name).to_bytes() == name.as_bytes() {
                    *fvalue.value.get() = value;
                }
                cur = *fvalue.next.get();
            }
        }
    }

    /// Walk the per-type `FValue<T>::list` and set every flag to `value` unless
    /// the host-supplied `skip` predicate (matched on the flag's UTF-8 name)
    /// returns true. Models the bool `--fflags=true|false` branch of the C++
    /// test harness `setFastFlags`, which sets every non-skipped flag. Startup-
    /// only, single-threaded — the same contract as flag construction.
    pub fn set_all_unless(value: T, skip: impl Fn(&str) -> bool) {
        unsafe {
            let mut cur = T::head().load(Ordering::Relaxed) as *const FValue<T>;
            while !cur.is_null() {
                let fvalue = &*cur;
                let name = core::ffi::CStr::from_ptr(fvalue.name)
                    .to_str()
                    .unwrap_or("");
                if !skip(name) {
                    *fvalue.value.get() = value;
                }
                cur = *fvalue.next.get();
            }
        }
    }
}
