//! Silence the default panic-hook output for the VM's `longjmp`-emulation unwinds.
//!
//! Luau's `luaD_throw` uses C++ exceptions for control flow; the faithful Rust
//! port (`lua_d_throw`) emulates this with [`std::panic::panic_any`] carrying a
//! [`lua_exception`], caught at the [`luaD_rawrunprotected`] boundary
//! (`VM/src/ldo.cpp`). These are NOT crashes — they are the normal mechanism by
//! which an ordinary Lua runtime error (`error(..)`, a failed `assert`, a type
//! error) propagates up to `pcall`/the resume boundary.
//!
//! The default Rust panic hook, however, prints `thread '...' panicked at ...`
//! to stderr for *every* unwind, including these caught ones — so a perfectly
//! normal Lua error made the CLI look like it had crashed (a `Box<dyn Any>`
//! message leaking before `catch_unwind` swallowed the payload).
//!
//! [`install_lua_exception_panic_hook`] installs (exactly once, process-wide) a
//! hook that suppresses the message for `lua_exception` payloads while
//! delegating every other panic to the previously-installed hook unchanged, so
//! genuine Rust bugs still surface with their full diagnostics.

use crate::records::lua_exception::lua_exception;
use std::sync::Once;

static INSTALL: Once = Once::new();

/// Install the `lua_exception`-silencing panic hook (idempotent; the first call
/// wins, subsequent calls are no-ops). Safe to call from any VM entry point.
pub fn install_lua_exception_panic_hook() {
    INSTALL.call_once(|| {
        let previous = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |info| {
            // A `lua_exception` payload is the VM's longjmp emulation, caught at
            // `luaD_rawrunprotected`; do not print anything for it.
            if info.payload().is::<lua_exception>() {
                return;
            }
            // Everything else is a real panic — preserve the prior behavior.
            previous(info);
        }));
    });
}
