//! C++ `extern const luau_FastFunction luauF_table[256]` (lbuiltins.h:9).
//! Source: `VM/src/lbuiltins.cpp` (hand-ported fallback plus Rive tail)
use crate::type_aliases::luau_fast_function::luau_FastFunction;

const fn make_fastcall_table() -> [luau_FastFunction; 256] {
    // This fork deliberately leaves the untranslated upstream prefix missing.
    let mut table: [luau_FastFunction; 256] =
        [Some(crate::functions::luau_f_missing::luau_f_missing); 256];
    table[243] = Some(crate::functions::luau_f_fround::luau_f_fround);
    // Slot 244 is reserved and remains missing.
    table[245] = Some(crate::functions::luau_f_vectordistance::luau_f_vectordistance);
    table[246] = Some(crate::functions::luau_f_vectordistancesquared::luau_f_vectordistancesquared);
    table[247] = Some(crate::functions::luau_f_vectororigin::luau_f_vectororigin);
    table[248] = Some(crate::functions::luau_f_vectorlengthsquared::luau_f_vectorlengthsquared);
    table[249] = Some(crate::functions::luau_f_vectordot::luau_f_vectordot);
    table[250] = Some(crate::functions::luau_f_vectormagnitude::luau_f_vectormagnitude);
    table[251] = Some(crate::functions::luau_f_rivevectornormalize::luau_f_rivevectornormalize);
    table[252] = Some(crate::functions::luau_f_vectorlerp::luau_f_vectorlerp);
    table[253] = Some(crate::functions::luau_f_vector2cross::luau_f_vector2cross);
    table[254] = Some(crate::functions::luau_f_vectorscaleandadd::luau_f_vectorscaleandadd);
    table[255] = Some(crate::functions::luau_f_vectorscaleandsub::luau_f_vectorscaleandsub);
    table
}

#[allow(non_upper_case_globals)]
#[export_name = "luaur_luauF_table"]
pub static luauF_table: [luau_FastFunction; 256] = make_fastcall_table();

const _: [(); 256] = [(); luauF_table.len()];
