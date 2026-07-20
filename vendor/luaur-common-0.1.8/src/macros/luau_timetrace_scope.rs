//! `LUAU_TIMETRACE_SCOPE(name, category)`. Reference:
//! `luau/Common/include/Luau/TimeTrace.h`.
//!
//! The TimeTrace profiling machinery (`Scope`, `createScopeData`, `ThreadContext`
//! …) lives behind C++ `#if defined(LUAU_ENABLE_TIME_TRACE)`, which is OFF in the
//! default build — there the macro is the empty `#else` form. This is that
//! default: a no-op, so downstream profiling-instrumented code compiles. The
//! enabled form (and the machinery it needs) is a deferred port behind a future
//! `time_trace` feature.

#[macro_export]
#[allow(non_snake_case)]
macro_rules! LUAU_TIMETRACE_SCOPE {
    ($name:expr, $category:expr) => {};
}

pub use LUAU_TIMETRACE_SCOPE;
