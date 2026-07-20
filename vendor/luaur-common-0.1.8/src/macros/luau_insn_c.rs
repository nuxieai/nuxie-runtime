#[allow(non_snake_case)]
#[inline]
pub const fn LUAU_INSN_C(insn: u32) -> u32 {
    (insn >> 24) & 0xff
}

// Macro shim: C++ LUAU_INSN_C is a #define; translated callers use both
// `LUAU_INSN_C(x)` (the const fn above) and `LUAU_INSN_C!(x)` forms.
#[allow(non_snake_case)]
#[macro_export]
macro_rules! __luau_insn_c_shim {
    ($insn:expr) => {
        $crate::macros::luau_insn_c::LUAU_INSN_C($insn)
    };
}
// Rename-re-export carries only the macro namespace, so it can share the
// module with the same-named const fn.
pub use __luau_insn_c_shim as LUAU_INSN_C;
