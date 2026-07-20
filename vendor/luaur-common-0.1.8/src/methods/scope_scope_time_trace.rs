//! Source: `Common/include/Luau/TimeTrace.h:148-159` (hand-ported)
//! C++:
//! ```cpp
//! explicit Scope(uint16_t token) : context(getThreadContext())
//! {
//!     if (!FFlag::DebugLuauTimeTracing) return;
//!     context.eventEnter(token);
//! }
//! ```
use crate::functions::get_thread_context::get_thread_context;
use crate::records::scope::Scope;
use crate::FFlag::DebugLuauTimeTracing;

impl Scope {
    pub fn scope(token: u16) -> Self {
        let context = get_thread_context();
        let scope = Scope {
            context: context as *mut _,
        };

        if !DebugLuauTimeTracing.get() {
            return scope;
        }

        context.event_enter_u16(token);
        scope
    }
}
