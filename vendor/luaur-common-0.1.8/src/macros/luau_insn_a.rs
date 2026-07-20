#[allow(non_snake_case)]
#[inline]
pub const fn LUAU_INSN_A(insn: u32) -> u32 {
    (insn >> 8) & 0xff
}

// Macro shim: C++ LUAU_INSN_A is a #define; translated callers use both
// `LUAU_INSN_A(x)` (the const fn above) and `LUAU_INSN_A!(x)` forms.
#[allow(non_snake_case)]
#[macro_export]
macro_rules! __luau_insn_a_shim {
    ($insn:expr) => {
        $crate::macros::luau_insn_a::LUAU_INSN_A($insn)
    };
}
// Rename-re-export carries only the macro namespace, so it can share the
// module with the same-named const fn.
pub use __luau_insn_a_shim as LUAU_INSN_A;
