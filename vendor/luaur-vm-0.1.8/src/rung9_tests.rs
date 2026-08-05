use crate::macros::luau_f_table::luauF_table;
use crate::macros::nvalue::nvalue;
use crate::macros::setnvalue::setnvalue;
use crate::macros::setvvalue::setvvalue;
use crate::macros::vvalue::vvalue;
use crate::type_aliases::lua_state::lua_State;
use crate::type_aliases::luau_fast_function::luau_FastFunction;
use crate::type_aliases::t_value::TValue;

fn address(function: luau_FastFunction) -> usize {
    function.expect("fastcall slot must be populated") as *const () as usize
}

#[test]
fn rawget_fastcall_reads_the_requested_table_entry() {
    unsafe {
        let state = crate::functions::lua_l_newstate::lua_l_newstate();
        assert!(!state.is_null());

        crate::functions::lua_createtable::lua_createtable(state, 0, 1);
        crate::functions::lua_pushnumber::lua_pushnumber(state, 7.0);
        crate::functions::lua_pushnumber::lua_pushnumber(state, 42.0);
        crate::functions::lua_rawset::lua_rawset(state, -3);

        let table = (*state).top.sub(1);
        crate::functions::lua_pushnumber::lua_pushnumber(state, 7.0);
        let key = (*state).top.sub(1);
        let result = (*state).top;

        assert_eq!(
            crate::functions::luau_f_rawget::luau_f_rawget(state, result, table, 1, key, 2),
            1
        );
        assert_eq!(nvalue!(result), 42.0);

        crate::functions::lua_close::lua_close(state);
    }
}

#[test]
fn select_fastcall_reads_varargs_before_the_frame_base() {
    unsafe {
        let state = crate::functions::lua_l_newstate::lua_l_newstate();
        assert!(!state.is_null());

        let original_base = (*state).base;
        let original_func = (*(*state).ci).func;
        let original_proto = (*(*state).ci).p;
        let stack = (*state).stack;
        let mut proto: crate::records::proto::Proto = core::mem::zeroed();
        proto.numparams = 2;

        (*(*state).ci).func = stack;
        (*(*state).ci).p = &mut proto;
        (*state).base = stack.add(6);
        setnvalue!(stack.add(3), 11.0);
        setnvalue!(stack.add(4), 22.0);
        setnvalue!(stack.add(5), 33.0);
        setnvalue!(stack.add(7), 2.0);

        luaur_common::FFlag::LuauCIProto.push_test_override(true);
        let status = crate::functions::luau_f_select::luau_f_select(
            state,
            stack.add(8),
            stack.add(7),
            1,
            core::ptr::null_mut(),
            1,
        );
        luaur_common::FFlag::LuauCIProto.pop_test_override();

        assert_eq!(status, 1);
        assert_eq!(nvalue!(stack.add(8)), 22.0);

        (*state).base = original_base;
        (*(*state).ci).func = original_func;
        (*(*state).ci).p = original_proto;
        crate::functions::lua_close::lua_close(state);
    }
}

#[cfg(target_arch = "x86_64")]
#[test]
fn ceil_sse_fastcall_rounds_toward_positive_infinity() {
    unsafe {
        let mut slots = [TValue::default(); 2];
        setnvalue!(slots.as_mut_ptr(), 1.25);

        assert_eq!(
            crate::functions::luau_f_ceil_sse_41::luau_f_ceil_sse_41(
                core::ptr::null_mut(),
                slots.as_mut_ptr().add(1),
                slots.as_mut_ptr(),
                1,
                core::ptr::null_mut(),
                1,
            ),
            1
        );
        assert_eq!(nvalue!(slots.as_ptr().add(1)), 2.0);
    }
}

#[test]
fn frexp_fastcall_handles_the_largest_finite_number() {
    unsafe {
        let mut slots = [TValue::default(); 3];
        setnvalue!(slots.as_mut_ptr(), f64::MAX);

        assert_eq!(
            crate::functions::luau_f_frexp::luau_f_frexp(
                core::ptr::null_mut(),
                slots.as_mut_ptr().add(1),
                slots.as_mut_ptr(),
                2,
                core::ptr::null_mut(),
                1,
            ),
            2
        );
        assert_eq!(nvalue!(slots.as_ptr().add(1)).to_bits(), 0x3fefffffffffffff);
        assert_eq!(nvalue!(slots.as_ptr().add(2)), 1024.0);
    }
}

#[test]
fn ldexp_fastcall_preserves_zero_when_the_scale_overflows() {
    unsafe {
        let mut slots = [TValue::default(); 3];
        setnvalue!(slots.as_mut_ptr(), -0.0);
        setnvalue!(slots.as_mut_ptr().add(1), 1024.0);

        assert_eq!(
            crate::functions::luau_f_ldexp::luau_f_ldexp(
                core::ptr::null_mut(),
                slots.as_mut_ptr().add(2),
                slots.as_mut_ptr(),
                1,
                slots.as_mut_ptr().add(1),
                2,
            ),
            1
        );
        assert_eq!(nvalue!(slots.as_ptr().add(2)).to_bits(), (-0.0f64).to_bits());
    }
}

#[test]
fn modf_fastcall_splits_infinity_into_infinity_and_signed_zero() {
    unsafe {
        let mut slots = [TValue::default(); 3];
        setnvalue!(slots.as_mut_ptr(), f64::NEG_INFINITY);

        assert_eq!(
            crate::functions::luau_f_modf::luauF_modf(
                core::ptr::null_mut(),
                slots.as_mut_ptr().add(1),
                slots.as_mut_ptr(),
                2,
                core::ptr::null_mut(),
                1,
            ),
            2
        );
        assert_eq!(nvalue!(slots.as_ptr().add(1)), f64::NEG_INFINITY);
        assert_eq!(nvalue!(slots.as_ptr().add(2)).to_bits(), (-0.0f64).to_bits());
    }
}

#[test]
fn arshift_fastcall_returns_the_uint32_bit_pattern() {
    unsafe {
        let mut slots = [TValue::default(); 3];
        setnvalue!(slots.as_mut_ptr(), 0x8000_0000u32 as f64);
        setnvalue!(slots.as_mut_ptr().add(1), 1.0);

        assert_eq!(
            crate::functions::luau_f_arshift::luau_f_arshift(
                core::ptr::null_mut(),
                slots.as_mut_ptr().add(2),
                slots.as_mut_ptr(),
                1,
                slots.as_mut_ptr().add(1),
                2,
            ),
            1
        );
        assert_eq!(nvalue!(slots.as_ptr().add(2)), 0xc000_0000u32 as f64);
    }
}

#[test]
fn byte_fastcall_falls_back_for_a_non_numeric_explicit_end() {
    unsafe {
        let state = crate::functions::lua_l_newstate::lua_l_newstate();
        assert!(!state.is_null());

        crate::functions::lua_pushstring::lua_pushstring(state, c"abc".as_ptr());
        crate::functions::lua_pushnumber::lua_pushnumber(state, 1.0);
        crate::functions::lua_pushnil::lua_pushnil(state);
        let arg0 = (*state).top.sub(3);
        let args = (*state).top.sub(2);

        assert_eq!(
            crate::functions::luau_f_byte::luauF_byte(
                state,
                (*state).top,
                arg0,
                1,
                args,
                3,
            ),
            -1
        );

        crate::functions::lua_close::lua_close(state);
    }
}

#[test]
fn getmetatable_fastcall_returns_the_tables_metatable() {
    unsafe {
        let state = crate::functions::lua_l_newstate::lua_l_newstate();
        assert!(!state.is_null());

        crate::functions::lua_createtable::lua_createtable(state, 0, 0);
        crate::functions::lua_createtable::lua_createtable(state, 0, 0);
        let expected = crate::hvalue!((*state).top.sub(1));
        assert_eq!(crate::functions::lua_setmetatable::lua_setmetatable(state, -2), 1);

        let table = (*state).top.sub(1);
        let result = (*state).top;
        assert_eq!(
            crate::functions::luau_f_getmetatable::luau_f_getmetatable(
                state,
                result,
                table,
                1,
                core::ptr::null_mut(),
                1,
            ),
            1
        );
        assert_eq!(crate::hvalue!(result), expected);

        crate::functions::lua_close::lua_close(state);
    }
}

#[test]
fn writefp_fastcall_instantiates_for_f32_and_writes_the_value() {
    unsafe {
        let state = crate::functions::lua_l_newstate::lua_l_newstate();
        assert!(!state.is_null());

        let data = crate::functions::lua_newbuffer::lua_newbuffer(state, 4).cast::<u8>();
        let buffer = (*state).top.sub(1);
        crate::functions::lua_pushnumber::lua_pushnumber(state, 0.0);
        crate::functions::lua_pushnumber::lua_pushnumber(state, 1.5);
        let args = (*state).top.sub(2);

        assert_eq!(
            crate::functions::luau_f_writefp::luau_f_writefp::<f32>(
                state,
                (*state).top,
                buffer,
                0,
                args,
                3,
            ),
            0
        );
        assert_eq!(core::ptr::read_unaligned(data.cast::<f32>()), 1.5f32);

        crate::functions::lua_close::lua_close(state);
    }
}

#[test]
fn bufferwritelong_fastcall_writes_only_in_bounds() {
    unsafe {
        let state = crate::functions::lua_l_newstate::lua_l_newstate();
        assert!(!state.is_null());

        let data = crate::functions::lua_newbuffer::lua_newbuffer(state, 8).cast::<u8>();
        let buffer = (*state).top.sub(1);
        crate::functions::lua_pushnumber::lua_pushnumber(state, 0.0);
        crate::functions::lua_pushinteger_64::lua_pushinteger_64(
            state,
            0x1020_3040_5060_7080,
        );
        let args = (*state).top.sub(2);

        assert_eq!(
            crate::functions::luau_f_bufferwritelong::luau_f_bufferwritelong(
                state,
                (*state).top,
                buffer,
                0,
                args,
                3,
            ),
            0
        );
        assert_eq!(
            core::ptr::read_unaligned(data.cast::<i64>()),
            0x1020_3040_5060_7080
        );

        setnvalue!(args, 1.0);
        assert_eq!(
            crate::functions::luau_f_bufferwritelong::luau_f_bufferwritelong(
                state,
                (*state).top,
                buffer,
                0,
                args,
                3,
            ),
            -1
        );

        crate::functions::lua_close::lua_close(state);
    }
}

#[test]
fn fastcall_table_wires_completed_families_and_rive_tail() {
    let missing = crate::functions::luau_f_missing::luau_f_missing as *const () as usize;
    assert_eq!(luauF_table.len(), 256);
    assert!(luauF_table[0].is_none());

    let math = [
        (2, crate::functions::luau_f_abs::luau_f_abs as *const () as usize),
        (3, crate::functions::luau_f_acos::luau_f_acos as *const () as usize),
        (4, crate::functions::luau_f_asin::luau_f_asin as *const () as usize),
        (5, crate::functions::luau_f_atan_2::luau_f_atan_2 as *const () as usize),
        (6, crate::functions::luau_f_atan::luau_f_atan as *const () as usize),
        (7, crate::functions::luau_f_ceil::luau_f_ceil as *const () as usize),
        (8, crate::functions::luau_f_cosh::luau_f_cosh as *const () as usize),
        (9, crate::functions::luau_f_cos::luau_f_cos as *const () as usize),
        (10, crate::functions::luau_f_deg::luau_f_deg as *const () as usize),
        (11, crate::functions::luau_f_exp::luau_f_exp as *const () as usize),
        (12, crate::functions::luau_f_floor::luau_f_floor as *const () as usize),
        (13, crate::functions::luau_f_fmod::luau_f_fmod as *const () as usize),
        (14, crate::functions::luau_f_frexp::luau_f_frexp as *const () as usize),
        (15, crate::functions::luau_f_ldexp::luau_f_ldexp as *const () as usize),
        (16, crate::functions::luau_f_log_10::luau_f_log_10 as *const () as usize),
        (17, crate::functions::luau_f_log::luau_f_log as *const () as usize),
        (18, crate::functions::luau_f_max::luau_f_max as *const () as usize),
        (19, crate::functions::luau_f_min::luau_f_min as *const () as usize),
        (20, crate::functions::luau_f_modf::luauF_modf as *const () as usize),
        (21, crate::functions::luau_f_pow::luau_f_pow as *const () as usize),
        (22, crate::functions::luau_f_rad::luau_f_rad as *const () as usize),
        (23, crate::functions::luau_f_sinh::luau_f_sinh as *const () as usize),
        (24, crate::functions::luau_f_sin::luau_f_sin as *const () as usize),
        (25, crate::functions::luau_f_sqrt::luau_f_sqrt as *const () as usize),
        (26, crate::functions::luau_f_tanh::luau_f_tanh as *const () as usize),
        (27, crate::functions::luau_f_tan::luau_f_tan as *const () as usize),
        (46, crate::functions::luau_f_clamp::luau_f_clamp as *const () as usize),
        (47, crate::functions::luau_f_sign::luau_f_sign as *const () as usize),
        (48, crate::functions::luau_f_round::luau_f_round as *const () as usize),
        (89, crate::functions::luau_f_lerp::luau_f_lerp as *const () as usize),
        (91, crate::functions::luau_f_isnan::luau_f_isnan as *const () as usize),
        (92, crate::functions::luau_f_isinf::luau_f_isinf as *const () as usize),
        (93, crate::functions::luau_f_isfinite::luau_f_isfinite as *const () as usize),
    ];
    for (slot, expected) in math {
        assert_eq!(address(luauF_table[slot]), expected, "slot {slot}");
    }

    let bit32 = [
        (28, crate::functions::luau_f_arshift::luau_f_arshift as *const () as usize),
        (29, crate::functions::luau_f_band::luau_f_band as *const () as usize),
        (30, crate::functions::luau_f_bnot::luau_f_bnot as *const () as usize),
        (31, crate::functions::luau_f_bor::luau_f_bor as *const () as usize),
        (32, crate::functions::luau_f_bxor::luau_f_bxor as *const () as usize),
        (33, crate::functions::luau_f_btest::luau_f_btest as *const () as usize),
        (34, crate::functions::luau_f_extract::luauF_extract as *const () as usize),
        (35, crate::functions::luau_f_lrotate::luau_f_lrotate as *const () as usize),
        (36, crate::functions::luau_f_lshift::luau_f_lshift as *const () as usize),
        (37, crate::functions::luau_f_replace::luau_f_replace as *const () as usize),
        (38, crate::functions::luau_f_rrotate::luau_f_rrotate as *const () as usize),
        (39, crate::functions::luau_f_rshift::luau_f_rshift as *const () as usize),
        (55, crate::functions::luau_f_countlz::luau_f_countlz as *const () as usize),
        (56, crate::functions::luau_f_countrz::luau_f_countrz as *const () as usize),
        (59, crate::functions::luau_f_extractk::luau_f_extractk as *const () as usize),
        (64, crate::functions::luau_f_byteswap::luau_f_byteswap as *const () as usize),
    ];
    for (slot, expected) in bit32 {
        assert_eq!(address(luauF_table[slot]), expected, "slot {slot}");
    }

    let core_and_string = [
        (1, crate::functions::luau_f_assert::luau_f_assert as *const () as usize),
        (40, crate::functions::luau_f_type::luau_f_type as *const () as usize),
        (41, crate::functions::luau_f_byte::luauF_byte as *const () as usize),
        (42, crate::functions::luau_f_char::luau_f_char as *const () as usize),
        (43, crate::functions::luau_f_len::luau_f_len as *const () as usize),
        (44, crate::functions::luau_f_typeof::luau_f_typeof as *const () as usize),
        (45, crate::functions::luau_f_sub::luau_f_sub as *const () as usize),
        (57, crate::functions::luau_f_select::luau_f_select as *const () as usize),
        (58, crate::functions::luau_f_rawlen::luau_f_rawlen as *const () as usize),
        (62, crate::functions::luau_f_tonumber::luau_f_tonumber as *const () as usize),
        (63, crate::functions::luau_f_tostring::luau_f_tostring as *const () as usize),
    ];
    for (slot, expected) in core_and_string {
        assert_eq!(address(luauF_table[slot]), expected, "slot {slot}");
    }

    let table = [
        (49, crate::functions::luau_f_rawset::luau_f_rawset as *const () as usize),
        (50, crate::functions::luau_f_rawget::luau_f_rawget as *const () as usize),
        (51, crate::functions::luau_f_rawequal::luauF_rawequal as *const () as usize),
        (52, crate::functions::luau_f_tinsert::luau_f_tinsert as *const () as usize),
        (53, crate::functions::luau_f_tunpack::luau_f_tunpack as *const () as usize),
        (60, crate::functions::luau_f_getmetatable::luau_f_getmetatable as *const () as usize),
        (61, crate::functions::luau_f_setmetatable::luau_f_setmetatable as *const () as usize),
    ];
    for (slot, expected) in table {
        assert_eq!(address(luauF_table[slot]), expected, "slot {slot}");
    }

    for slot in (54..55)
        .chain(65..89)
        .chain([90])
        .chain(94..243)
    {
        assert_eq!(address(luauF_table[slot]), missing, "slot {slot}");
    }
    assert_eq!(
        address(luauF_table[243]),
        crate::functions::luau_f_fround::luau_f_fround as *const () as usize
    );
    assert_eq!(address(luauF_table[244]), missing);

    let expected = [
        crate::functions::luau_f_vectordistance::luau_f_vectordistance as *const () as usize,
        crate::functions::luau_f_vectordistancesquared::luau_f_vectordistancesquared as *const ()
            as usize,
        crate::functions::luau_f_vectororigin::luau_f_vectororigin as *const () as usize,
        crate::functions::luau_f_vectorlengthsquared::luau_f_vectorlengthsquared as *const ()
            as usize,
        crate::functions::luau_f_vectordot::luau_f_vectordot as *const () as usize,
        crate::functions::luau_f_vectormagnitude::luau_f_vectormagnitude as *const () as usize,
        crate::functions::luau_f_rivevectornormalize::luau_f_rivevectornormalize as *const ()
            as usize,
        crate::functions::luau_f_vectorlerp::luau_f_vectorlerp as *const () as usize,
        crate::functions::luau_f_vector2cross::luau_f_vector2cross as *const () as usize,
        crate::functions::luau_f_vectorscaleandadd::luau_f_vectorscaleandadd as *const () as usize,
        crate::functions::luau_f_vectorscaleandsub::luau_f_vectorscaleandsub as *const () as usize,
    ];
    for (offset, expected) in expected.into_iter().enumerate() {
        assert_eq!(address(luauF_table[245 + offset]), expected);
    }
}

#[test]
fn rive_fastcalls_preserve_numeric_contracts() {
    unsafe {
        let mut slots = [TValue::default(); 4];
        setnvalue!(slots.as_mut_ptr(), 0.1f64);
        assert_eq!(
            crate::functions::luau_f_fround::luau_f_fround(
                core::ptr::null_mut(),
                slots.as_mut_ptr().add(3),
                slots.as_mut_ptr(),
                1,
                core::ptr::null_mut(),
                1,
            ),
            1
        );
        assert_eq!(nvalue!(slots.as_ptr().add(3)), (0.1f64 as f32) as f64);

        setvvalue!(
            core::ptr::null_mut::<lua_State>(),
            slots.as_mut_ptr(),
            3.0,
            4.0,
            0.0,
            0.0
        );
        assert_eq!(
            crate::functions::luau_f_rivevectornormalize::luau_f_rivevectornormalize(
                core::ptr::null_mut(),
                slots.as_mut_ptr().add(3),
                slots.as_mut_ptr(),
                1,
                core::ptr::null_mut(),
                1,
            ),
            1
        );
        let normalized = vvalue!(slots.as_ptr().add(3));
        assert_eq!(
            [normalized[0], normalized[1], normalized[2]],
            [0.6, 0.8, 0.0]
        );

        setvvalue!(
            core::ptr::null_mut::<lua_State>(),
            slots.as_mut_ptr(),
            0.0,
            0.0,
            0.0,
            0.0
        );
        crate::functions::luau_f_rivevectornormalize::luau_f_rivevectornormalize(
            core::ptr::null_mut(),
            slots.as_mut_ptr().add(3),
            slots.as_mut_ptr(),
            1,
            core::ptr::null_mut(),
            1,
        );
        let zero = vvalue!(slots.as_ptr().add(3));
        assert_eq!([zero[0], zero[1], zero[2]], [0.0, 0.0, 0.0]);
    }
}

#[test]
fn math_fround_fallback_matches_fastcall_round_trip() {
    unsafe {
        let state = crate::functions::lua_l_newstate::lua_l_newstate();
        assert!(!state.is_null());
        crate::functions::lua_pushnumber::lua_pushnumber(state, 0.1);
        assert_eq!(crate::functions::math_fround::math_fround(state), 1);
        assert_eq!(nvalue!((*state).top.sub(1)), (0.1f64 as f32) as f64);
        crate::functions::lua_close::lua_close(state);
    }
}

#[test]
fn lua_pushvector2_preserves_the_stack_slots_stale_z_component() {
    unsafe {
        let state = crate::functions::lua_l_newstate::lua_l_newstate();
        assert!(!state.is_null());
        let slot = (*state).top;
        setvvalue!(state, slot, 9.0, 8.0, 7.0, 0.0);

        crate::functions::lua_pushvector2::lua_pushvector2(state, 1.25, -2.5);

        assert_eq!((*state).top, slot.add(1));
        let value = vvalue!(slot);
        assert_eq!([value[0], value[1], value[2]], [1.25, -2.5, 7.0]);
        crate::functions::lua_close::lua_close(state);
    }
}

#[test]
fn rive_base_library_omits_print_and_newproxy() {
    unsafe {
        let state = crate::functions::lua_l_newstate::lua_l_newstate();
        assert!(!state.is_null());
        crate::functions::luaopen_base::luaopen_base(state);

        assert_eq!(
            crate::macros::lua_getglobal::lua_getglobal(state, c"print".as_ptr()),
            crate::enums::lua_type::lua_Type::LUA_TNIL as i32
        );
        assert_eq!(
            crate::macros::lua_getglobal::lua_getglobal(state, c"newproxy".as_ptr()),
            crate::enums::lua_type::lua_Type::LUA_TNIL as i32
        );
        crate::functions::lua_close::lua_close(state);
    }
}

#[test]
fn rive_error_prefixes_use_double_colon_fallbacks() {
    unsafe {
        let state = crate::functions::lua_l_newstate::lua_l_newstate();
        assert!(!state.is_null());

        crate::functions::lua_l_where::lua_l_where(state, 1);
        let mut length = 0;
        let where_text = crate::functions::lua_tolstring::lua_tolstring(state, -1, &mut length);
        assert_eq!(
            core::slice::from_raw_parts(where_text.cast::<u8>(), length),
            b":: "
        );

        crate::functions::pusherror::pusherror(state, c"boom".as_ptr());
        let error_text = crate::functions::lua_tolstring::lua_tolstring(state, -1, &mut length);
        assert_eq!(
            core::slice::from_raw_parts(error_text.cast::<u8>(), length),
            b":: boom"
        );

        let non_utf8_message = [b'x', 0xff, 0];
        crate::functions::pusherror::pusherror(state, non_utf8_message.as_ptr().cast());
        let error_text = crate::functions::lua_tolstring::lua_tolstring(state, -1, &mut length);
        assert_eq!(
            core::slice::from_raw_parts(error_text.cast::<u8>(), length),
            b":: x\xff"
        );
        crate::functions::lua_close::lua_close(state);
    }
}
