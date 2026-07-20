//! Source: `Common/src/TimeTrace.cpp:94-98` (hand-ported)
//! C++ `~GlobalContext() { if (traceFile) fclose(traceFile); }`.
use crate::records::global_context::GlobalContext;

impl Drop for GlobalContext {
    fn drop(&mut self) {
        // Closing the file: dropping the `Option<File>` flushes and closes it,
        // matching `fclose(traceFile)`.
        if let Ok(mut state) = self.state.lock() {
            state.trace_file = None;
        }
    }
}
