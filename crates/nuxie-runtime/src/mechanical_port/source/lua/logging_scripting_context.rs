//! Context-owned logging state from `src/lua/logging_scripting_context.cpp`.
//!
//! The Rust VM owns this state directly. Lua stack/error extraction
//! and VM construction stay at the `nuxie-scripting` backend boundary.

use std::{cell::RefCell, rc::Rc};

use crate::mechanical_port::source::logging_scripting_context::ScriptingLogLevel;

// VM-local callbacks need not cross threads. The public factory sink does;
// the backend moves it onto the server thread before installing this callback.
type LocalLogSink = Rc<dyn Fn(ScriptingLogLevel, &[u8])>;

/// Clones refer to the same context, including its single reentrant line buffer.
#[derive(Clone, Default)]
pub struct LoggingScriptingContext {
    sink: Rc<RefCell<Option<LocalLogSink>>>,
    line: Rc<RefCell<Vec<u8>>>,
}

impl LoggingScriptingContext {
    pub fn set_sink(&self, sink: Rc<dyn Fn(ScriptingLogLevel, &[u8])>) {
        *self.sink.borrow_mut() = Some(sink);
    }

    pub fn clear_sink(&self) {
        self.sink.borrow_mut().take();
    }

    pub fn begin_line(&self) {
        self.line.borrow_mut().clear();
    }

    pub fn append(&self, data: &[u8]) {
        self.line.borrow_mut().extend_from_slice(data);
    }

    pub fn end_line(&self) {
        // Release the actual line borrow before invoking host code. The clear
        // remains after the callback, matching the pinned context's ordering.
        let line = self.line.borrow().clone();
        self.log(ScriptingLogLevel::Info, &line);
        self.line.borrow_mut().clear();
    }

    pub fn print_error(&self, error: Option<&[u8]>) {
        if let Some(error) = error {
            // lua_tostring + strlen, unlike print's length-bearing Span.
            let length = error
                .iter()
                .position(|byte| *byte == 0)
                .unwrap_or(error.len());
            self.log(ScriptingLogLevel::Error, &error[..length]);
        }
    }

    fn log(&self, level: ScriptingLogLevel, line: &[u8]) {
        let sink = self.sink.borrow().clone();
        if let Some(sink) = sink {
            sink(level, line);
        }
    }
}
