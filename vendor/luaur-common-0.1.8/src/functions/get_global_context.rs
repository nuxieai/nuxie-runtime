//! Source: `Common/src/TimeTrace.cpp:109-113` (hand-ported)
//! C++ `getGlobalContext()` returns a process-wide `shared_ptr` singleton:
//! ```cpp
//! static std::shared_ptr<GlobalContext> context = std::shared_ptr<GlobalContext>{new GlobalContext};
//! return context;
//! ```
use crate::records::global_context::GlobalContext;
use std::sync::{Arc, OnceLock};

pub fn get_global_context() -> Arc<GlobalContext> {
    static CONTEXT: OnceLock<Arc<GlobalContext>> = OnceLock::new();
    CONTEXT
        .get_or_init(|| Arc::new(GlobalContext::new()))
        .clone()
}
