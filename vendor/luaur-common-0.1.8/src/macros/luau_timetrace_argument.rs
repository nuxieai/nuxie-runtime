//! `LUAU_TIMETRACE_ARGUMENT(name, value)`. Reference:
//! `luau/Common/include/Luau/TimeTrace.h`. No-op in the default build
//! (`LUAU_ENABLE_TIME_TRACE` off — the C++ `#else` is `do {} while (false)`); see
//! [`crate::macros::luau_timetrace_scope`] for the feature-gating note.

#[macro_export]
#[allow(non_snake_case)]
macro_rules! LUAU_TIMETRACE_ARGUMENT {
    ($name:expr, $value:expr) => {};
}

pub use LUAU_TIMETRACE_ARGUMENT;
