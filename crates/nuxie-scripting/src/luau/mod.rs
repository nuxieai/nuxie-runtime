//! The Luau handle API the scripting bindings are written against
//! (docs/luau-safe-vm-plan.md, Phase 2 lane 7 and Phase 3): a facade over
//! the safe VM in `crates/nuxie-luau-vm` (`facade/`) in the shape of the
//! crate the bindings were first written against, the binding layer of the
//! vendored VM (deleted in Phase 3,
//! `docs/evidence/luau-safe-vm-phase3-deletion-2026-11-13.md`). The facade's
//! notes call it "the old binding crate". Same type and method names, same
//! error rendering, same userdata dispatch; the notes say where it differs.

#[cfg(not(feature = "luau"))]
compile_error!("nuxie-scripting's `luau-host` needs a VM: enable `luau` (the safe VM)");

mod facade;
pub use facade::*;
