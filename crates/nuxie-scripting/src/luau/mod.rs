//! The Luau handle API the scripting bindings are written against
//! (docs/luau-safe-vm-plan.md, Phase 2 lane 7 and Phase 3): a facade over
//! the safe VM in `crates/nuxie-luau-vm` (`facade/`) in the shape of the
//! `luaur-rt` crate the bindings were first written against (the vendored
//! VM, deleted in Phase 3). Same type and method names, same error
//! rendering, same userdata dispatch; its notes say where it differs.

#[cfg(not(feature = "luau"))]
compile_error!("nuxie-scripting's `luau-host` needs a VM: enable `luau` (the safe VM)");

mod facade;
pub use facade::*;
