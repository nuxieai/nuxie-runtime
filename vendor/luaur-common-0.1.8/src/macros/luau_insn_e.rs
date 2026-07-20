#[allow(non_snake_case)]
#[inline]
pub const fn LUAU_INSN_E(insn: u32) -> i32 {
    (insn as i32) >> 8
}

// Macro shim: C++ LUAU_INSN_E is a #define; translated callers use both
// `LUAU_INSN_E(x)` (the const fn above) and `LUAU_INSN_E!(x)` forms.
#[allow(non_snake_case)]
#[macro_export]
macro_rules! __luau_insn_e_shim {
    ($insn:expr) => {
        $crate::macros::luau_insn_e::LUAU_INSN_E($insn)
    };
}
// Rename-re-export carries only the macro namespace, so it can share the
// module with the same-named const fn.
pub use __luau_insn_e_shim as LUAU_INSN_E;
