use crate::macros::markobject::markobject;
use crate::macros::utag_internal_limit::UTAG_INTERNAL_LIMIT;
use crate::type_aliases::global_state::global_State;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

#[allow(non_snake_case)]
pub(crate) unsafe fn markudatadirectfields(g: *mut global_State) {
    LUAU_ASSERT!(luaur_common::FFlag::LuauDirectFieldGet.get());

    for i in 0..UTAG_INTERNAL_LIMIT as usize {
        if !(*g).udatadirectfields[i].is_null() {
            markobject!(g, (*g).udatadirectfields[i]);
        }
    }
}
