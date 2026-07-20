//! Source: `Common/src/TimeTrace.cpp:115-123` (hand-ported)
//! C++:
//! ```cpp
//! uint16_t createToken(GlobalContext& context, const char* name, const char* category)
//! {
//!     std::scoped_lock lock(context.mutex);
//!     LUAU_ASSERT(context.tokens.size() < 64 * 1024);
//!     context.tokens.push_back({name, category});
//!     return uint16_t(context.tokens.size() - 1);
//! }
//! ```
use crate::macros::luau_assert::LUAU_ASSERT;
use crate::records::global_context::GlobalContext;
use crate::records::token::Token;
use core::ffi::c_char;

pub fn create_token(context: &GlobalContext, name: *const c_char, category: *const c_char) -> u16 {
    // `scoped_lock lock(context.mutex)` — the lock guards the mutable state.
    let mut state = context
        .state
        .lock()
        .expect("TimeTrace GlobalContext mutex poisoned");

    LUAU_ASSERT!(state.tokens.len() < 64 * 1024);

    state.tokens.push(Token { name, category });
    (state.tokens.len() - 1) as u16
}
