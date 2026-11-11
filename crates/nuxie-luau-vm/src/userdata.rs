//! Full userdata (C++ `Udata`, `LUA_TUSERDATA`), phase 2 lane 6; the
//! payload moved to the host in lane 8 (docs/luau-safe-vm-embedding-core.md,
//! section 0).
//!
//! A userdata is an arena object (`Heap::userdata`): the host's key for its
//! payload, a metatable, a tag and the payload size the host declared. The
//! core never looks behind the key:
//!
//! - **Ownership.** The host owns the payload, stored under the key. When
//!   the collector frees the userdata, the sweep lists the key
//!   (`Heap::take_freed_userdata`) and the host drops the payload at its
//!   next drain point; a key is listed exactly once. The lane-6 API's host
//!   (`compat.rs`) keeps the payload as `Rc<RefCell<T>>`, so `T`'s `Drop`
//!   runs once, when the last `Rc` goes. Key 0 names no payload.
//! - **Accounting.** A userdata costs `USERDATA_BYTES` plus the declared
//!   size, charged before it exists and credited by the sweep (budget.rs,
//!   rule 1).
//! - **Metatables.** Each userdata has its own metatable slot, filled at
//!   creation from the per-tag metatable (C++ `lua_setuserdatametatable`)
//!   or one the host picks. Every metamethod goes through the same
//!   metatable lookup as for any other value (`Core::metatable`).
//! - **Tags** are C++'s `lua_newuserdatatagged` tags, 0..=`UTAG_LIMIT`-1;
//!   plain userdata have tag 0.

use crate::value::TableId;

/// C++ `LUA_UTAG_LIMIT`: userdata tags are below it.
pub const UTAG_LIMIT: u8 = 128;

/// The arena object.
#[derive(Default)]
pub(crate) struct Userdata {
    /// The host's key for the payload; 0 for none (a freed slot).
    pub(crate) key: u64,
    pub(crate) meta: Option<TableId>,
    pub(crate) tag: u8,
    /// The payload bytes charged to the budget.
    pub(crate) size: usize,
}
