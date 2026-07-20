//! Node: `cxx:Macro:Luau.VM:VM/src/lclass.h:51:lua_r_lookupmemberatoffset`
//! Source: `VM/src/lclass.h:51-54` (hand-fixed: the original translation used
//! C ternary syntax `? :` and could never have compiled). Yields `*mut TValue`.

#[allow(non_upper_case_globals)]
#[macro_export]
macro_rules! luaR_lookupmemberatoffset {
    ($inst:expr, $offset:expr) => {{
        let inst = $inst;
        let offset = $offset;
        luaur_common::LUAU_ASSERT!(
            $crate::macros::lua_r_checkoffsetinbounds::luaR_checkoffsetinbounds!(inst, offset)
        );
        unsafe {
            if offset < (*(*inst).lclass).numberofinstancemembers {
                (*inst).members.add(offset as usize)
            } else {
                (*(*inst).lclass)
                    .staticmembers
                    .add((offset - (*(*inst).lclass).numberofinstancemembers) as usize)
            }
        }
    }};
}

pub use luaR_lookupmemberatoffset;
