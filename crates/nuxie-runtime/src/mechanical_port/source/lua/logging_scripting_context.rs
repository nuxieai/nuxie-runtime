use crate::mechanical_port::source::{
    command_queue::ScriptingContextFactory,
    factory::Factory,
    lua::rive_lua_libs::{CPPRuntimeScriptingContext, LuaState, ScriptingContext},
};

#[repr(i32)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ScriptingLogLevel {
    Info = 0,
    Warn = 1,
    Error = 2,
}

pub type ScriptingLogSink = Box<dyn FnMut(ScriptingLogLevel, &[u8])>;

pub struct LoggingScriptingContext {
    pub base: CPPRuntimeScriptingContext,
    sink: ScriptingLogSink,
    line: Vec<u8>,
}

impl LoggingScriptingContext {
    pub fn new(factory: &mut Factory, sink: ScriptingLogSink) -> Self {
        Self {
            base: CPPRuntimeScriptingContext::new(factory),
            sink,
            line: Vec::new(),
        }
    }
}

impl ScriptingContext for LoggingScriptingContext {
    fn print_begin_line(&mut self, _state: &mut LuaState) {
        self.line.clear();
    }

    fn print(&mut self, data: &[u8]) {
        self.line.extend_from_slice(data);
    }

    fn print_end_line(&mut self) {
        (self.sink)(ScriptingLogLevel::Info, &self.line);
        self.line.clear();
    }

    fn print_error(&mut self, state: &mut LuaState) {
        if let Some(error) = state.to_string(-1) {
            (self.sink)(ScriptingLogLevel::Error, error.as_bytes());
        }
    }
}

pub fn make_logging_scripting_context_factory(
    sink: Option<ScriptingLogSink>,
) -> Option<ScriptingContextFactory> {
    let sink = sink?;
    Some(Box::new(move |factory| {
        Box::new(LoggingScriptingContext::new(factory, sink))
    }))
}
