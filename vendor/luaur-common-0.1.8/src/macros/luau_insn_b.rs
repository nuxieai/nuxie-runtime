#[allow(non_snake_case)]
#[inline]
pub const fn LUAU_INSN_B(insn: u32) -> u32 {
    (insn >> 16) & 0xff
}

// Macro shim: C++ LUAU_INSN_B is a #define; translated callers use both
// `LUAU_INSN_B(x)` (the const fn above) and `LUAU_INSN_B!(x)` forms.
#[allow(non_snake_case)]
#[macro_export]
macro_rules! __luau_insn_b_shim {
    ($insn:expr) => {
        $crate::macros::luau_insn_b::LUAU_INSN_B($insn)
    };
}
// Rename-re-export carries only the macro namespace, so it can share the
// module with the same-named const fn.
pub use __luau_insn_b_shim as LUAU_INSN_B;
