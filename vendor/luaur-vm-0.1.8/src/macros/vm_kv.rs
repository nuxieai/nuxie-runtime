#[allow(non_snake_case)]
#[macro_export]
macro_rules! VM_KV {
    ($i:expr, $L:expr, $cl:expr, $k:expr) => {{
        let i = $i;
        let L = $L;
        let cl = $cl;
        let k = $k;
        let p = if luaur_common::FFlag::LuauCIProto.get() {
            unsafe { (*(*L).ci).p }
        } else {
            unsafe {
                let l = &(*cl).inner.l;
                l.p
            }
        };
        luaur_common::LUAU_ASSERT!((i as u32) < (unsafe { (*p).sizek } as u32));
        unsafe { &mut *k.add(i as usize) }
    }};
}

pub use VM_KV;
