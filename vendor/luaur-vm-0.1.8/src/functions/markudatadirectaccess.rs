use crate::macros::markvalue::markvalue;
use crate::macros::utag_internal_limit::UTAG_INTERNAL_LIMIT;
use crate::type_aliases::global_state::global_State;
use crate::type_aliases::t_value::TValue;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

#[allow(non_snake_case)]
pub(crate) unsafe fn markudatadirectaccess(g: *mut global_State) {
    LUAU_ASSERT!(luaur_common::DFFlag::LuauGcMarkUdataAccess.get());

    for i in 0..UTAG_INTERNAL_LIMIT as usize {
        let udatadirect = core::ptr::addr_of_mut!((*g).udatadirect[i]);

        markvalue!(
            g,
            core::ptr::addr_of_mut!((*udatadirect).indextm) as *mut TValue
        );
        markvalue!(
            g,
            core::ptr::addr_of_mut!((*udatadirect).newindextm) as *mut TValue
        );
        markvalue!(
            g,
            core::ptr::addr_of_mut!((*udatadirect).namecalltm) as *mut TValue
        );
    }
}
