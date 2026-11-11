//! The Luau handle API the scripting bindings are written against.
//!
//! One binding source compiles for both VMs (docs/luau-safe-vm-plan.md,
//! Phase 2 lane 7):
//!
//! - **`luau`** (the default): the vendored `luaur-rt`, re-exported as it
//!   is. Nothing here changes its behaviour.
//! - **`nuxie-vm`**: a `luaur-rt`-shaped facade over the safe VM in
//!   `crates/nuxie-luau-vm` (`facade/`). Same type and method names,
//!   same error rendering, same userdata dispatch, so the bindings do not
//!   change. Its notes say where it differs.
//!
//! Code that needs the vendored VM's raw C API (the `exec_raw` sites and
//! `luaur_vm` symbols of `docs/luau-safe-vm-seam.md` section 2) is behind
//! `cfg(not(feature = "nuxie-vm"))` beside its facade replacement. The
//! audio processor engine is the exception: it is a separate raw embedding
//! of the vendored VM under both settings (seam rows R33 to R36).

#[cfg(not(any(feature = "luau", feature = "nuxie-vm")))]
compile_error!("nuxie-scripting's `luau-host` needs a VM: enable `luau` (vendored) or `nuxie-vm`");

#[cfg(not(feature = "nuxie-vm"))]
pub use luaur_rt::*;

#[cfg(feature = "nuxie-vm")]
mod facade;
#[cfg(feature = "nuxie-vm")]
pub use facade::*;
