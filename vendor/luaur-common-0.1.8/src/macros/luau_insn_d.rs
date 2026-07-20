#[allow(non_snake_case)]
#[inline(always)]
pub const fn LUAU_INSN_D(insn: u32) -> i32 {
    (insn as i32) >> 16
}

// Macro shim: C++ LUAU_INSN_D is a #define; translated callers use both
// `LUAU_INSN_D(x)` (the const fn above) and `LUAU_INSN_D!(x)` forms.
#[allow(non_snake_case)]
#[macro_export]
macro_rules! __luau_insn_d_shim {
    ($insn:expr) => {
        $crate::macros::luau_insn_d::LUAU_INSN_D($insn)
    };
}
// Rename-re-export carries only the macro namespace, so it can share the
// module with the same-named const fn.
pub use __luau_insn_d_shim as LUAU_INSN_D;
