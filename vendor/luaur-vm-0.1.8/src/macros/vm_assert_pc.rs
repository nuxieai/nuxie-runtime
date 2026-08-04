#[allow(non_snake_case)]
#[macro_export]
macro_rules! VM_ASSERT_PC {
    ($pc:expr, $L:expr, $cl:expr) => {{
        let pc = $pc;
        let L = $L;
        let cl = $cl;
        let p = if luaur_common::FFlag::LuauCIProto.get() {
            unsafe { (*(*L).ci).p }
        } else {
            unsafe {
                let l =
                    core::ptr::addr_of!((*cl).inner.l) as *const $crate::records::closure::LClosure;
                (*l).p
            }
        };
        luaur_common::LUAU_ASSERT!(unsafe {
            (pc.offset_from((*p).code) as u32) < (*p).sizecode as u32
        });
    }};
}

pub use VM_ASSERT_PC;
