//! Lua-free host logging contract from `rive/logging_scripting_context.hpp`.
//!
//! The factory is constructed by `nuxie_scripting::make_logging_scripting_context_factory`,
//! where the approved Rust VM implementation lives. The runtime never constructs
//! a second VM or depends back on its scripting backend.

use std::sync::Arc;

#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScriptingLogLevel {
    Info = 0,
    Warn = 1,
    Error = 2,
}

/// One complete line, without a trailing newline. The bytes are valid only
/// during the callback. The factory carries this callback to the command
/// server thread; the shared callback must therefore be thread-safe.
pub type ScriptingLogSink = Arc<dyn Fn(ScriptingLogLevel, &[u8]) + Send + Sync>;
