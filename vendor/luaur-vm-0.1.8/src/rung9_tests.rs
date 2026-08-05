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
        crate::functions::lua_close::lua_close(state);
    }
}
