//! Concrete VM factory for `rive/logging_scripting_context.hpp`.
//!
//! Construction lives with the approved Rust scripting backend, while the
//! native runtime owns the logging contract and context's retained line state.

use nuxie_runtime::source::command_queue::ScriptingContextFactory;
pub use nuxie_runtime::source::logging_scripting_context::{ScriptingLogLevel, ScriptingLogSink};

pub fn make_logging_scripting_context_factory(
    sink: Option<ScriptingLogSink>,
) -> Option<ScriptingContextFactory> {
    #[cfg(feature = "luau")]
    {
        let sink = sink?;
        Some(Box::new(move |factory| {
            let vm = crate::vm::ScriptVm::new_with_log_sink(move |level, line| sink(level, line));
            let vm = nuxie_runtime::RuntimeScriptingVmHandle::new(Box::new(vm));
            vm.install_render_factory(&factory).ok()?;
            Some(vm)
        }))
    }
    #[cfg(not(feature = "luau"))]
    {
        // The pinned !WITH_RIVE_SCRIPTING implementation returns nullptr even
        // when a sink was supplied.
        let _ = sink;
        None
    }
}
