//! `isdead(g, v)` — true when the GCObject `v` is dead in global state `g`.
//! C++ definition: `VM/src/lgc.h:68`
//!   `#define isdead(g,v) (((v)->gch.marked & (WHITEBITS|bitmask(FIXEDBIT))) == (otherwhite(g) & WHITEBITS))`
//!
//! The macro body lives in `otherwhite.rs` (which also defines `otherwhite!`).
//! This module is the canonical import point; callers use `crate::isdead!` which
//! resolves to the `#[macro_export]`-ed macro regardless of which module `use`s it.

// Re-export the macro so that `use crate::macros::isdead::isdead;` still works
// for callers that import by path.
pub use crate::isdead;
