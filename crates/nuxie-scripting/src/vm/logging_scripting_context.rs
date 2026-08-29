//! Lua error extraction for the one native logging context owned by the VM.

use luaur_rt::Error;

pub use nuxie_runtime::source::logging_scripting_context::{ScriptingLogLevel, ScriptingLogSink};
pub(super) use nuxie_runtime::source::lua::logging_scripting_context::LoggingScriptingContext;

pub(super) trait LoggingScriptingContextLua {
    fn log_error(&self, error: &Error);
}

impl LoggingScriptingContextLua for LoggingScriptingContext {
    fn log_error(&self, error: &Error) {
        match error {
            // These messages are the values that were on the Lua stack. The
            // Display implementation adds prefixes absent from lua_tostring.
            Error::RuntimeError(message)
            | Error::SyntaxError { message, .. }
            | Error::MemoryError(message) => self.print_error(Some(message.as_bytes())),
            Error::CallbackError { cause, .. } => self.log_error(cause),
            _ => self.print_error(Some(error.to_string().as_bytes())),
        }
    }
}
