#[cfg(not(target_arch = "wasm32"))]
#[allow(non_upper_case_globals)]
pub use luaur_common::macros::win_32_lean_and_mean::WIN32_LEAN_AND_MEAN;

#[cfg(not(target_arch = "wasm32"))]
#[allow(non_upper_case_globals)]
pub const WIN_32_LEAN_AND_MEAN: () = ();
