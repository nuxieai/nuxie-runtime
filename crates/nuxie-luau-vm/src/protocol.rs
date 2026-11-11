//! The call protocol's modules: the only code that reaches the frame
//! stack directly (docs/luau-safe-vm-call-path.md, section 4.1).
//!
//! The frame list's methods are visible in this module only
//! (`pub(in crate::protocol)`, frames.rs). Everything else in the crate
//! (error positions, argument-error names, `debug.*`, `getfenv`/`setfenv`,
//! the hosted driver, the embedding API) reads frames through
//! `Core::frames_observed`, which first turns a frameless native into the
//! real frame it stands for. The files stay where they were; this module
//! only groups them so the compiler can enforce that rule.

#[path = "call.rs"]
pub(crate) mod call;
#[path = "frames.rs"]
pub(crate) mod frames;
#[path = "gc.rs"]
pub(crate) mod gc;
#[path = "interp.rs"]
pub(crate) mod interp;
#[path = "thread.rs"]
pub(crate) mod thread;
