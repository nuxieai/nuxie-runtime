//! A handle API over the safe VM (`crates/nuxie-luau-vm`) in the shape of the
//! old binding crate (the vendored VM's binding layer, deleted in Phase 3; see
//! `super`).
//!
//! The scripting bindings are written against that shape: handles that
//! carry their VM (`table.get(k)`, no VM argument), host callbacks that get
//! `&Lua`, `UserData` registration through `add_methods`/`add_fields`. The
//! safe VM's API passes the VM explicitly instead (`table.get(vm, k)`,
//! callbacks get `&mut HostCtx`). This module bridges the two without
//! `unsafe`:
//!
//! - **The lend slot.** [`Lua`] owns the [`vm::Vm`] in a slot. A handle
//!   operation takes the VM out of the slot, runs, and puts it back
//!   (`Shared::with_vm`). A host callback runs inside the VM, which holds
//!   the VM through the callback's `&mut HostCtx`; the trampoline swaps
//!   the VM's contents with a spare VM (`std::mem::swap`, two moves of the
//!   `Vm` struct), parks the real VM in the slot for the callback's
//!   duration, and swaps it back when the callback returns (`lend`). So
//!   every handle operation inside a callback reaches the same VM the
//!   callback runs in, and nothing aliases it. The interpreter never
//!   touches the spare: it is parked in the callback's frame until the
//!   swap back.
//! - **Values** are the old binding crate's: a whole number reads back as
//!   [`Value::Integer`], numbers as `f64`, handles carry their [`Lua`].
//! - **Errors** render as the old binding crate renders them (`error.rs`): a
//!   callback's `Err` reaches Lua as its `Display` text, `Error::lua_l_runtime`
//!   is positioned like `luaL_error`, an interrupt's runtime error is raised
//!   bare.
//! - **Userdata** keeps the old binding crate's per-type metatable with getters,
//!   setters and methods, dispatched in the order the old binding crate's Luau
//!   dispatcher uses by the VM's native dispatchers (`Vm::userdata_dispatchers`,
//!   `userdata.rs`).
//!
//! Where this differs from the old binding crate, the item says so. The lane 7
//! evidence doc lists the differences that tests observe.

#![deny(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::panic,
    clippy::unreachable,
    clippy::arithmetic_side_effects,
    clippy::unwrap_in_result
)]

mod conversion;
mod error;
mod handles;
mod multi;
mod state;
mod userdata;
mod value;
#[cfg(all(test, feature = "compiler"))]
mod tests;

pub use error::{Error, ExternalError, ExternalResult, Result};
pub use handles::{Buffer, Function, LuaString, Table, Thread, ThreadStatus};
pub use multi::{MultiValue, Variadic};
pub use state::{
    AppDataRef, AppDataRefMut, Debug, DebugWhat, InterruptGuard, Lua, RegistryKey, Scope, VmState,
    WeakLua, luau_direct_field_get, set_luau_direct_field_get,
};
pub use userdata::{
    AnyUserData, MetaMethod, UserData, UserDataFields, UserDataMethods, UserDataRef,
    UserDataRefMut,
};
pub use value::{Integer, LightUserData, Nil, Number, Value, Vector};

pub use conversion::{FromLua, FromLuaMulti, IntoLua, IntoLuaMulti};

/// Compiles Luau source as `Lua::load` does (the product flag profile).
#[cfg(feature = "compiler")]
pub use state::{Chunk, compile};

/// The safe VM itself, for the code that needs what the old binding crate
/// exposed only through raw hatches (seam rows R24 to R29: tagged plain-data
/// userdata with direct fields). Reached through [`Lua::with_vm`].
pub use nuxie_luau_vm as vm;
