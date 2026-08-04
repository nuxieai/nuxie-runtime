#[allow(non_snake_case)]
#[macro_export]
macro_rules! getproto {
    ($cl:expr) => {{
        let cl = $cl;
        if unsafe { (*cl).isC != 0 } {
            core::ptr::null_mut()
        } else {
            let p = unsafe {
                let l =
                    core::ptr::addr_of!((*cl).inner.l) as *const $crate::records::closure::LClosure;
                (*l).p
            };

            if luaur_common::FFlag::LuauPromoteProto.get() && unsafe { !(*p).optimized.is_null() } {
                unsafe { $crate::functions::lua_f_promoteproto::luaF_promoteproto(cl) }
            } else {
                p
            }
        }
    }};
}

pub use getproto;
