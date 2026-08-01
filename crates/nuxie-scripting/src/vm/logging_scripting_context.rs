//! Host-routed script logging, mirroring `logging_scripting_context.cpp`.

use std::cell::RefCell;
use std::rc::Rc;

use luaur_rt::Error;

/// Severity attached to one complete host-routed script log line.
///
/// `Warn` is retained for parity with C++ `ScriptingLogLevel`; the pinned Lua
/// bindings currently emit `Info` for `_G.print` and `Error` for Lua failures.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScriptingLogLevel {
    Info,
    Warn,
    Error,
}

/// Host callback for one fully assembled script log line, without a newline.
pub type ScriptingLogSink = Rc<dyn Fn(ScriptingLogLevel, &[u8])>;

/// VM-owned equivalent of the pinned C++ `LoggingScriptingContext`.
///
/// The line buffer intentionally belongs to the context rather than a single
/// `print` invocation. This preserves C++ behavior when `__tostring` performs
/// a reentrant `print`: the nested begin-line clears the outer prefix.
#[derive(Clone, Default)]
pub(super) struct LoggingScriptingContext {
    sink: Rc<RefCell<Option<ScriptingLogSink>>>,
    line: Rc<RefCell<Vec<u8>>>,
}

impl LoggingScriptingContext {
    pub(super) fn set_sink(&self, sink: ScriptingLogSink) {
        *self.sink.borrow_mut() = Some(sink);
    }

    pub(super) fn clear_sink(&self) {
        self.sink.borrow_mut().take();
    }

    pub(super) fn begin_line(&self) {
        self.line.borrow_mut().clear();
    }

    pub(super) fn append(&self, data: &[u8]) {
        self.line.borrow_mut().extend_from_slice(data);
    }

    pub(super) fn end_line(&self) {
        let line = self.line.borrow().clone();
        self.log(ScriptingLogLevel::Info, &line);
        self.line.borrow_mut().clear();
    }

    fn log(&self, level: ScriptingLogLevel, line: &[u8]) {
        // Release the RefCell borrow before invoking user code so a host sink
        // may replace or clear itself synchronously.
        let sink = self.sink.borrow().clone();
        if let Some(sink) = sink {
            sink(level, line);
        }
    }

    pub(super) fn log_error(&self, error: &Error) {
        match error {
            // These messages are the values that were on the Lua stack. The
            // Error Display implementation adds category prefixes that pinned
            // `lua_tostring(state, -1)` does not emit.
            Error::RuntimeError(message)
            | Error::SyntaxError { message, .. }
            | Error::MemoryError(message) => self.log(ScriptingLogLevel::Error, message.as_bytes()),
            Error::CallbackError { cause, .. } => self.log_error(cause),
            _ => {
                let message = error.to_string();
                self.log(ScriptingLogLevel::Error, message.as_bytes());
            }
        }
    }
}
