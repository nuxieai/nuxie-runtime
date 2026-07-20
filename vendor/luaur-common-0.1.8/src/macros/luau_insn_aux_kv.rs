#[inline(always)]
#[allow(non_snake_case)]
pub const fn LUAU_INSN_AUX_KV(aux: u32) -> u32 {
    aux & 0xffffff
}
