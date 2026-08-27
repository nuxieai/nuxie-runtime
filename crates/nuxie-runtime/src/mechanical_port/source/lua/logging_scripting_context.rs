use crate::mechanical_port::source::{
    command_queue::ScriptingContextFactory,
    factory::Factory,
    lua::rive_lua_libs::{CPPRuntimeScriptingContext, LuaState, ScriptingContext},
};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ScriptingLogLevel {
    Info,
    Warn,
    Error,
}

pub type ScriptingLogSink = Box<dyn FnMut(ScriptingLogLevel, &[u8])>;

#[cfg(feature = "rive_scripting")]
pub struct LoggingScriptingContext {
    pub base: CPPRuntimeScriptingContext,
    sink: ScriptingLogSink,
    line: String,
}

#[cfg(feature = "rive_scripting")]
impl LoggingScriptingContext {
    pub fn new(factory: &mut Factory, sink: ScriptingLogSink) -> Self {
        Self {
            base: CPPRuntimeScriptingContext::new(factory),
            sink,
            line: String::new(),
        }
    }
}

#[cfg(feature = "rive_scripting")]
impl ScriptingContext for LoggingScriptingContext {
    fn print_begin_line(&mut self, _state: &mut LuaState) {
        self.line.clear();
    }

    fn print(&mut self, data: &[u8]) {
        self.line.push_str(std::str::from_utf8(data).unwrap());
    }

    fn print_end_line(&mut self) {
        (self.sink)(ScriptingLogLevel::Info, self.line.as_bytes());
        self.line.clear();
    }

    fn print_error(&mut self, state: &mut LuaState) {
        if let Some(error) = state.to_string(-1) {
            (self.sink)(ScriptingLogLevel::Error, error.as_bytes());
        }
    }
}

#[cfg(feature = "rive_scripting")]
pub fn make_logging_scripting_context_factory(
    sink: Option<ScriptingLogSink>,
) -> Option<ScriptingContextFactory> {
    let sink = sink?;
    Some(Box::new(move |factory| {
        Box::new(LoggingScriptingContext::new(factory, sink))
    }))
}

#[cfg(not(feature = "rive_scripting"))]
pub fn make_logging_scripting_context_factory(
    _sink: Option<ScriptingLogSink>,
) -> Option<ScriptingContextFactory> {
    None
}
