//! Silence the default panic-hook output for the parser's exception-emulation
//! unwinds (`ParseError`).
//!
//! Luau's parser uses C++ exceptions (`throw ParseError`) to abort on a fatal
//! syntax error or a recursion-/error-limit overflow; the faithful Rust port
//! (`ParseError::raise`) emulates this with [`std::panic::panic_any`] carrying a
//! [`ParseError`], caught at the `Parser::parse` boundary. These are NOT crashes
//! — they are how the parser reports e.g. a deeply-nested expression that
//! exceeds the recursion limit.
//!
//! As with the VM's `lua_exception`, the default Rust panic hook printed
//! `thread '...' panicked at ... Box<dyn Any>` to stderr for these caught
//! unwinds, making an ordinary parse failure look like a crash.
//! [`install_parse_error_panic_hook`] installs (exactly once, process-wide) a
//! hook that suppresses the message for `ParseError` payloads and delegates all
//! other panics to the previously-installed hook unchanged — so it composes
//! cleanly with the VM's hook and never hides a genuine Rust bug.

use crate::records::parse_error::ParseError;
use std::sync::Once;

static INSTALL: Once = Once::new();

/// Install the `ParseError`-silencing panic hook (idempotent).
pub fn install_parse_error_panic_hook() {
    INSTALL.call_once(|| {
        let previous = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |info| {
            if info.payload().is::<ParseError>() {
                return;
            }
            previous(info);
        }));
    });
}
