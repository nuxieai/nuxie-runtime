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
fn rive_fastcall_table_only_wires_the_fork_delta() {
    let missing = crate::functions::luau_f_missing::luau_f_missing as *const () as usize;
    assert_eq!(luauF_table.len(), 256);
    assert!(luauF_table[..243]
        .iter()
        .all(|function| address(*function) == missing));
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
