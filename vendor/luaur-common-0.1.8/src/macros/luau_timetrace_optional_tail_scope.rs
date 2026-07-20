//! `LUAU_TIMETRACE_OPTIONAL_TAIL_SCOPE(name, category, microsec)`. Reference:
//! `luau/Common/include/Luau/TimeTrace.h`. No-op in the default build
//! (`LUAU_ENABLE_TIME_TRACE` off — the C++ `#else` form); see
//! [`crate::macros::luau_timetrace_scope`] for the feature-gating note.

#[macro_export]
#[allow(non_snake_case)]
macro_rules! LUAU_TIMETRACE_OPTIONAL_TAIL_SCOPE {
    ($name:expr, $category:expr, $microsec:expr) => {};
}

pub use LUAU_TIMETRACE_OPTIONAL_TAIL_SCOPE;
