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
//! - `heap`: typed arenas, interning, the stop-the-world collector (D2).
//! - `table`: tables; the array part is C++'s, exactly (D4).
//! - `bytecode`: instruction encoding, prototypes, the loader and the
//!   verifier that runs on load (D7).
//! - `vm`: the VM state, frames, the call protocol, errors, upvalues, GC
//!   roots. `interp`: the instruction loop. `meta`: operations with
//!   metamethod fallbacks. `num`: number formatting and parsing.
//! - `libs`: the base library and the parts of `string`, `table` and `math`
//!   lane 1 needs.
//! - `host`: the embedding API ([`Vm`], [`HostCtx`], [`HostValue`]).
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

mod bytecode;
mod error;
mod heap;
mod host;
mod interp;
mod libs;
mod limits;
mod meta;
mod native;
mod num;
mod table;
mod value;
mod vm;

pub use error::{Error, ErrorKind, LoadError};
pub use heap::CollectStats;
pub use host::{HostCtx, HostError, HostValue};
pub use limits::Limits;
pub use native::HostFn;
pub use vm::Vm;

/// Luau's `tostring` of a number (C++ `luai_num2str`: the shortest
/// round-tripping digits, fixed notation when the decimal point falls within
/// -5..=21 digits, otherwise `d.ddde+XX`).
pub fn format_number(n: f64) -> String {
    num::num2str(n)
}
