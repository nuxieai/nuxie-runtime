//! The Luau handle API the scripting bindings are written against.
//!
//! One binding source compiles for both VMs (docs/luau-safe-vm-plan.md,
//! Phase 2 lane 7, flipped in Phase 3):
//!
//! - **`luau`** (the default): a `luaur-rt`-shaped facade over the safe VM
//!   in `crates/nuxie-luau-vm` (`facade/`). Same type and method names,
//!   same error rendering, same userdata dispatch, so the bindings do not
//!   change. Its notes say where it differs from `luaur-rt`.
//! - **`vendored-vm`** (opt-out, kept until the vendored VM is deleted):
//!   the vendored `luaur-rt`, re-exported as it is. When it is on it is the
//!   VM the crate runs, whatever else is on.
//!
//! Code that needs the vendored VM's raw C API (the `exec_raw` sites and
//! `luaur_vm` symbols of `docs/luau-safe-vm-seam.md` section 2) is behind
//! `cfg(feature = "vendored-vm")` beside its facade replacement. The audio
//! processor engine follows the same split (`safe.rs` by default, `raw.rs`
//! under `vendored-vm`).

#[cfg(not(feature = "luau"))]
compile_error!("nuxie-scripting's `luau-host` needs a VM: enable `luau` (the safe VM, optionally with `vendored-vm`)");

#[cfg(feature = "vendored-vm")]
pub use luaur_rt::*;

#[cfg(not(feature = "vendored-vm"))]
mod facade;
#[cfg(not(feature = "vendored-vm"))]
pub use facade::*;
