use crate::type_aliases::t_value::TValue;

#[allow(non_camel_case_types)]
pub type sort_predicate = Option<
    unsafe fn(
        L: *mut crate::type_aliases::lua_state::lua_State,
        l: *const TValue,
        r: *const TValue,
    ) -> core::ffi::c_int,
>;

pub type SortPredicate = sort_predicate;
