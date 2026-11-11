//! A safe Luau virtual machine: plan `docs/luau-safe-vm-plan.md`, Phase 2,
//! lane 1 (the core).
//!
//! The crate executes Luau bytecode (versions 3 through 11) produced by
//! the vendored compiler. It contains no `unsafe` code, and the clippy lints
//! that flag a possible panic are denied at the crate root, so every
//! failure a script can cause is a returned [`Error`] (plan decision D3).
//! Every resource limit is a field of [`Limits`].
//!
//! Module map:
//!
//! - `value`: the 16-byte tagged [`Value`](value) and the typed object ids (D1).
//! - `budget`: the memory budget, the one accounting point for every
//!   allocation (D3); its module comment says how a builtin charges.
//! - `heap`: typed arenas, interning, the incremental collector with weak
//!   tables (D2). `gc`: safepoints, roots, pacing, `collectgarbage`.
//! - `table`: tables; the array part is C++'s, exactly (D4).
//! - `bytecode`: instruction encoding, prototypes, the loader and the
//!   verifier that runs on load (D7).
//! - `vm`: the VM state, frames, errors, upvalues, GC roots. `call`: the
//!   call protocol (every call a frame, lane 4). `interp`: the instruction
//!   loop. `meta`: operations with
//!   metamethod fallbacks. `num`: number formatting and parsing.
//! - `thread`: coroutines as arena objects; the running thread's state
//!   is checked out into the `Vm` (lane 4).
//! - `libs`: the base library and the parts of `string`, `table` and `math`
//!   lane 1 needs.
//! - `host`: the run entry points ([`Vm::run`], [`Vm::start`]), [`HostCtx`]
//!   and [`HostError`].
//! - `embed`, `handle`, `convert`, `userdata`: the embedding API (lane 6):
//!   rooted handles, typed conversions, host userdata, host functions with
//!   continuations, threads, the interrupt and the memory budget.
//!
//! Seams for the later lanes are described where they live: lane 2 in
//! `table`, lane 3 in `heap`, lane 4 in `vm`, lane 5 in `libs`, lane 6 in
//! `host`.

#![forbid(unsafe_code)]
#![deny(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::panic,
    clippy::unreachable,
    clippy::arithmetic_side_effects,
    clippy::unwrap_in_result
)]

mod budget;
mod bytecode;
mod call;
mod compat;
mod convert;
mod embed;
#[cfg(test)]
mod embed_tests;
mod error;
mod frames;
mod gc;
mod handle;
mod heap;
mod host;
mod hosted;
mod interp;
mod libs;
mod limits;
mod meta;
mod native;
mod num;
mod table;
mod thread;
mod userdata;
mod value;
mod vm;

pub use convert::{FromLua, FromLuaMulti, IntoLua, IntoLuaMulti};
pub use embed::{DirectField, RegistryKey, StackInfo, ThreadState, UserDataType};
pub use error::{Error, ErrorKind, LoadError};
pub use handle::{
    AnyUserData, Buffer, Function, LightUserData, LuaString, MultiValue, Table, Thread, Value,
};
pub use heap::CollectStats;
pub use compat::{HostContFn, HostFn, Vm};
pub use host::{CallStatus, GcTune, HostCtx, HostError, HostValue, RunState};
pub use hosted::{Host, HostedVm};
pub use limits::Limits;
pub use native::HostFnId;
pub use userdata::UTAG_LIMIT;
pub use vm::{Core, InterruptFn};

/// Luau's `tostring` of a number (C++ `luai_num2str`: the shortest
/// round-tripping digits, fixed notation when the decimal point falls within
/// -5..=21 digits, otherwise `d.ddde+XX`).
pub fn format_number(n: f64) -> String {
    num::num2str(n)
}
