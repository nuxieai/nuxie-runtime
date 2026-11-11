//! Full userdata (C++ `Udata`, `LUA_TUSERDATA`), phase 2 lane 6.
//!
//! A userdata is an arena object (`Heap::userdata`) holding a host value,
//! a metatable and a tag. The host value is a `T: 'static` behind
//! `Rc<RefCell<T>>`:
//!
//! - **Ownership.** The arena slot owns one `Rc`. The collector frees the
//!   slot when the userdata is unreachable, which drops that `Rc`; `T`'s
//!   `Drop` (the destructor, C++ `lua_newuserdatadtor`) runs when the last
//!   `Rc` goes, so exactly once: at collection, at `Vm` drop, or later if
//!   the host still holds a borrow guard. A destructor cannot reach the VM
//!   (it has no reference to it), so it can neither allocate in the VM nor
//!   run Lua.
//! - **Borrows.** The host borrows the value through the `Rc`, never
//!   through the arena, so a borrow does not hold the `Vm` borrowed and a
//!   callback can borrow a userdata and still call back into the VM.
//!   Borrows are `RefCell::try_borrow`/`try_borrow_mut`: a re-entrant
//!   callback that asks for a conflicting borrow (a method that calls Lua
//!   that calls a `_mut` method on the same object) gets an error, never a
//!   panic.
//! - **Accounting.** A userdata costs `USERDATA_BYTES` plus the size the
//!   host declared (by default `size_of::<T>()`), charged before it exists
//!   and credited by the sweep (budget.rs, rule 1).
//! - **Metatables.** Each userdata has its own metatable slot, filled at
//!   creation from the per-type metatable (`Vm::set_userdata_metatable`,
//!   keyed by `TypeId`) or the per-tag one (`Vm::set_tag_metatable`, C++
//!   `lua_setuserdatametatable`). Every metamethod (`__index`,
//!   `__newindex`, `__namecall`, arithmetic, comparison, `__eq`, `__len`,
//!   `__call`, `__tostring`, `__iter`, `__type`) goes through the same
//!   metatable lookup as for any other value (`Vm::metatable`).
//! - **Tags** are C++'s `lua_newuserdatatagged` tags, 0..=`UTAG_LIMIT`-1;
//!   plain userdata have tag 0.

use std::any::{Any, TypeId};
use std::cell::RefCell;
use std::rc::Rc;

use crate::value::TableId;

/// C++ `LUA_UTAG_LIMIT`: userdata tags are below it.
pub const UTAG_LIMIT: u8 = 128;

/// The arena object.
#[derive(Default)]
pub(crate) struct Userdata {
    /// `Rc<RefCell<T>>`, erased. `None` only for a freed slot.
    pub(crate) payload: Option<Rc<dyn Any>>,
    /// `TypeId::of::<T>()`.
    pub(crate) type_id: Option<TypeId>,
    /// `std::any::type_name::<T>()`, for errors.
    pub(crate) type_name: &'static str,
    pub(crate) meta: Option<TableId>,
    pub(crate) tag: u8,
    /// The payload bytes charged to the budget.
    pub(crate) size: usize,
}

impl Userdata {
    pub(crate) fn new<T: 'static>(value: T, size: usize, tag: u8, meta: Option<TableId>) -> Self {
        let payload: Rc<dyn Any> = Rc::new(RefCell::new(value));
        Userdata {
            payload: Some(payload),
            type_id: Some(TypeId::of::<T>()),
            type_name: std::any::type_name::<T>(),
            meta,
            tag,
            size,
        }
    }

    /// The payload as `Rc<RefCell<T>>`, if it holds a `T`.
    pub(crate) fn cell<T: 'static>(&self) -> Option<Rc<RefCell<T>>> {
        let payload = Rc::clone(self.payload.as_ref()?);
        payload.downcast::<RefCell<T>>().ok()
    }

    pub(crate) fn is<T: 'static>(&self) -> bool {
        self.type_id == Some(TypeId::of::<T>())
    }
}
