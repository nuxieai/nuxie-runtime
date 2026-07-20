//! Port of `Luau::formatAppend` from `Common/src/StringUtils.cpp`.
//!
//! See [`crate::functions::vformat_append`] for the documented deviation: the
//! C++ variadic `void formatAppend(std::string& str, const char* fmt, ...)`
//! becomes a `core::fmt::Arguments` consumer (callers pass `format_args!(...)`)
//! so the port stays on stable + `wasm32`.

use alloc::string::String;

use crate::functions::vformat_append::vformatAppend;

#[allow(non_snake_case)]
pub fn formatAppend(str: &mut String, args: core::fmt::Arguments<'_>) {
    vformatAppend(str, args);
}
