//! Port of `Luau::format` from `Common/src/StringUtils.cpp`.
//!
//! See [`crate::functions::vformat_append`] for the documented deviation: the
//! C++ variadic `std::string format(const char* fmt, ...)` becomes a
//! `core::fmt::Arguments` consumer (callers pass `format_args!(...)`) so the
//! port stays on stable + `wasm32`.

use alloc::string::String;

use crate::functions::vformat::vformat;

#[allow(non_snake_case)]
pub fn format(args: core::fmt::Arguments<'_>) -> String {
    vformat(args)
}
