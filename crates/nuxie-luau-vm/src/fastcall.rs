//! The builtin fast table (C++ `luauF_table`, `lbuiltins.cpp`):
//! docs/luau-safe-vm-call-path.md, section 4.2.
//!
//! The compiler writes a builtin's id into `FASTCALL*` when a call names a
//! library function it knows (`math.abs`, `type`, `Vector.dot`, ...). Under
//! `safeenv` the handler (`interp.rs`) runs the id's fast body here instead
//! of the call: no frame, no argument window, no result move. A body that
//! does not apply (a wrong type, a count it does not handle) returns
//! `None` and the `CALL` that follows runs the library function, which
//! raises the library's error. Every body is ported from its vendored
//! `luau_f_*` (C++ Luau's `VM/src/lbuiltins.cpp`), bail conditions
//! included, and lives beside its library function in `libs/`; the two
//! share one computation where their results agree.
//!
//! Where the vendored body cannot fail but this VM's can (an allocation
//! over the memory budget), the body returns `None` too: the library
//! function then meets the same limit and raises the memory error.
//!
//! Ids 94 to 132 (`integer.*` and `buffer.readinteger`/`writeinteger`) are
//! emitted only under `LuauIntegerFastcalls`, which the pinned compiler
//! profiles leave off; their bodies are ported all the same (decision 2)
//! and checked by unit tests (`libs/integer.rs`).

use crate::libs::{base, bit32, buffer, integer, math, string, table, vector};
use crate::value::Value;
use crate::vm::Core;

/// A fast body: the VM, the result register `ra` (the call's function
/// slot), the results the call wants (`-1` for all), up to three
/// arguments by value, and the argument count. A fourth and later
/// argument is in register `ra + 1 + i` (only the variadic `FASTCALL`
/// form has more than three). Writes its results from `ra` and returns
/// their count, or `None` to fall through to the `CALL`.
pub(crate) type FastFn = fn(&mut Core, usize, i32, [Value; 3], usize) -> Option<usize>;

#[derive(Clone, Copy)]
pub(crate) struct FastEntry {
    pub(crate) f: FastFn,
    /// The body may allocate: the handler runs the safepoint after it.
    pub(crate) allocates: bool,
}

/// The entry for builtin `id`, if it has a fast body.
#[inline]
pub(crate) fn entry(id: u8) -> Option<FastEntry> {
    FAST.get(usize::from(id)).copied().flatten()
}

/// Argument `i` (0-based) of a fast call: one of the three passed by value,
/// or a register above them.
#[inline]
pub(crate) fn arg(vm: &Core, ra: usize, args: &[Value; 3], i: usize) -> Value {
    match args.get(i) {
        Some(&v) => v,
        None => vm.get(ra.saturating_add(1).saturating_add(i)),
    }
}

const fn plain(f: FastFn) -> Option<FastEntry> {
    Some(FastEntry { f, allocates: false })
}

const fn allocating(f: FastFn) -> Option<FastEntry> {
    Some(FastEntry { f, allocates: true })
}

/// `luauF_table`, indexed by the compiler's `LuauBuiltinFunction` id.
static FAST: [Option<FastEntry>; 256] = table();

const fn table() -> [Option<FastEntry>; 256] {
    let mut t: [Option<FastEntry>; 256] = [None; 256];
    t[1] = plain(base::fast_assert);
    t[2] = plain(math::fast_abs);
    t[3] = plain(math::fast_acos);
    t[4] = plain(math::fast_asin);
    t[5] = plain(math::fast_atan2);
    t[6] = plain(math::fast_atan);
    t[7] = plain(math::fast_ceil);
    t[8] = plain(math::fast_cosh);
    t[9] = plain(math::fast_cos);
    t[10] = plain(math::fast_deg);
    t[11] = plain(math::fast_exp);
    t[12] = plain(math::fast_floor);
    t[13] = plain(math::fast_fmod);
    t[14] = plain(math::fast_frexp);
    t[15] = plain(math::fast_ldexp);
    t[16] = plain(math::fast_log10);
    t[17] = plain(math::fast_log);
    t[18] = plain(math::fast_max);
    t[19] = plain(math::fast_min);
    t[20] = plain(math::fast_modf);
    t[21] = plain(math::fast_pow);
    t[22] = plain(math::fast_rad);
    t[23] = plain(math::fast_sinh);
    t[24] = plain(math::fast_sin);
    t[25] = plain(math::fast_sqrt);
    t[26] = plain(math::fast_tanh);
    t[27] = plain(math::fast_tan);
    t[28] = plain(bit32::fast_arshift);
    t[29] = plain(bit32::fast_band);
    t[30] = plain(bit32::fast_bnot);
    t[31] = plain(bit32::fast_bor);
    t[32] = plain(bit32::fast_bxor);
    t[33] = plain(bit32::fast_btest);
    t[34] = plain(bit32::fast_extract);
    t[35] = plain(bit32::fast_lrotate);
    t[36] = plain(bit32::fast_lshift);
    t[37] = plain(bit32::fast_replace);
    t[38] = plain(bit32::fast_rrotate);
    t[39] = plain(bit32::fast_rshift);
    t[40] = plain(base::fast_type);
    t[41] = plain(string::fast_byte);
    t[42] = allocating(string::fast_char);
    t[43] = plain(string::fast_len);
    t[44] = plain(base::fast_typeof);
    t[45] = allocating(string::fast_sub);
    t[46] = plain(math::fast_clamp);
    t[47] = plain(math::fast_sign);
    t[48] = plain(math::fast_round);
    t[49] = allocating(base::fast_rawset);
    t[50] = plain(base::fast_rawget);
    t[51] = plain(base::fast_rawequal);
    t[52] = allocating(table::fast_insert);
    t[53] = plain(table::fast_unpack);
    t[54] = plain(vector::fast_create);
    t[55] = plain(bit32::fast_countlz);
    t[56] = plain(bit32::fast_countrz);
    t[57] = plain(base::fast_select);
    t[58] = plain(base::fast_rawlen);
    t[59] = plain(bit32::fast_extractk);
    t[60] = plain(base::fast_getmetatable);
    t[61] = plain(base::fast_setmetatable);
    t[62] = plain(base::fast_tonumber);
    t[63] = allocating(base::fast_tostring);
    t[64] = plain(bit32::fast_byteswap);
    t[65] = plain(buffer::fast_readi8);
    t[66] = plain(buffer::fast_readu8);
    t[67] = plain(buffer::fast_writeu8);
    t[68] = plain(buffer::fast_readi16);
    t[69] = plain(buffer::fast_readu16);
    t[70] = plain(buffer::fast_writeu16);
    t[71] = plain(buffer::fast_readi32);
    t[72] = plain(buffer::fast_readu32);
    t[73] = plain(buffer::fast_writeu32);
    t[74] = plain(buffer::fast_readf32);
    t[75] = plain(buffer::fast_writef32);
    t[76] = plain(buffer::fast_readf64);
    t[77] = plain(buffer::fast_writef64);
    t[78] = plain(vector::fast_magnitude);
    t[79] = plain(vector::fast_normalize);
    t[80] = plain(vector::fast_cross);
    t[81] = plain(vector::fast_dot);
    t[82] = plain(vector::fast_floor);
    t[83] = plain(vector::fast_ceil);
    t[84] = plain(vector::fast_abs);
    t[85] = plain(vector::fast_sign);
    t[86] = plain(vector::fast_clamp);
    t[87] = plain(vector::fast_min);
    t[88] = plain(vector::fast_max);
    t[89] = plain(math::fast_lerp);
    t[90] = plain(vector::fast_lerp);
    t[91] = plain(math::fast_isnan);
    t[92] = plain(math::fast_isinf);
    t[93] = plain(math::fast_isfinite);
    t[94] = plain(integer::fast_create);
    t[95] = plain(integer::fast_tonumber);
    t[96] = plain(integer::fast_neg);
    t[97] = plain(integer::fast_add);
    t[98] = plain(integer::fast_sub);
    t[99] = plain(integer::fast_mul);
    t[100] = plain(integer::fast_div);
    t[101] = plain(integer::fast_min);
    t[102] = plain(integer::fast_max);
    t[103] = plain(integer::fast_rem);
    t[104] = plain(integer::fast_idiv);
    t[105] = plain(integer::fast_udiv);
    t[106] = plain(integer::fast_urem);
    t[107] = plain(integer::fast_mod);
    t[108] = plain(integer::fast_clamp);
    t[109] = plain(integer::fast_band);
    t[110] = plain(integer::fast_bor);
    t[111] = plain(integer::fast_bnot);
    t[112] = plain(integer::fast_bxor);
    t[113] = plain(integer::fast_lt);
    t[114] = plain(integer::fast_le);
    t[115] = plain(integer::fast_ult);
    t[116] = plain(integer::fast_ule);
    t[117] = plain(integer::fast_gt);
    t[118] = plain(integer::fast_ge);
    t[119] = plain(integer::fast_ugt);
    t[120] = plain(integer::fast_uge);
    t[121] = plain(integer::fast_lshift);
    t[122] = plain(integer::fast_rshift);
    t[123] = plain(integer::fast_arshift);
    t[124] = plain(integer::fast_lrotate);
    t[125] = plain(integer::fast_rrotate);
    t[126] = plain(integer::fast_extract);
    t[127] = plain(integer::fast_btest);
    t[128] = plain(integer::fast_countrz);
    t[129] = plain(integer::fast_countlz);
    t[130] = plain(integer::fast_bswap);
    t[131] = plain(buffer::fast_readinteger);
    t[132] = plain(buffer::fast_writeinteger);
    // The Rive tail. 244 is reserved and stays missing.
    t[243] = plain(math::fast_fround);
    t[245] = plain(vector::fast_rive_distance);
    t[246] = plain(vector::fast_rive_distance_squared);
    t[247] = plain(vector::fast_rive_origin);
    t[248] = plain(vector::fast_rive_length_squared);
    t[249] = plain(vector::fast_dot);
    t[250] = plain(vector::fast_magnitude);
    t[251] = plain(vector::fast_rive_normalize);
    t[252] = plain(vector::fast_lerp);
    t[253] = plain(vector::fast_rive_cross);
    t[254] = plain(vector::fast_rive_scale_and_add);
    t[255] = plain(vector::fast_rive_scale_and_sub);
    t
}

/// The bodies that take one number and give one (`luauF_abs` and the
/// rest): `nparams >= 1`, `nresults <= 1`, a number argument.
macro_rules! fast_number1 {
    ($name:ident, $f:expr) => {
        pub(crate) fn $name(
            vm: &mut Core,
            ra: usize,
            nresults: i32,
            [a0, ..]: [Value; 3],
            nparams: usize,
        ) -> Option<usize> {
            let Value::Number(x) = a0 else { return None };
            if nparams < 1 || nresults > 1 {
                return None;
            }
            let f: fn(f64) -> Value = $f;
            vm.set(ra, f(x));
            Some(1)
        }
    };
}

/// The bodies that take two numbers and give one (`luauF_atan2`, `pow`,
/// `fmod`, `ldexp`): `nparams >= 2`, `nresults <= 1`.
macro_rules! fast_number2 {
    ($name:ident, $f:expr) => {
        pub(crate) fn $name(
            vm: &mut Core,
            ra: usize,
            nresults: i32,
            [a0, a1, _]: [Value; 3],
            nparams: usize,
        ) -> Option<usize> {
            let (Value::Number(x), Value::Number(y)) = (a0, a1) else { return None };
            if nparams < 2 || nresults > 1 {
                return None;
            }
            let f: fn(f64, f64) -> Value = $f;
            vm.set(ra, f(x, y));
            Some(1)
        }
    };
}

pub(crate) use {fast_number1, fast_number2};

#[cfg(test)]
mod tests {
    /// Every id the vendored table fills is filled here, and no other.
    #[test]
    fn the_table_has_the_vendored_ids() {
        let filled: Vec<usize> = (0..256)
            .filter(|&i| u8::try_from(i).ok().and_then(super::entry).is_some())
            .collect();
        let mut want: Vec<usize> = (1..=132).collect();
        want.push(243);
        want.extend(245..=255);
        assert_eq!(filled, want);
    }
}
