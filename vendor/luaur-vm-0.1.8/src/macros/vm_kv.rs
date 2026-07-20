#[allow(non_snake_case)]
#[macro_export]
macro_rules! VM_KV {
    ($i:expr, $cl:expr, $k:expr) => {{
        let i = $i;
        let cl = $cl;
        let k = $k;
        luaur_common::LUAU_ASSERT!(
            (i as u32)
                < (unsafe {
                    let l = &(*cl).inner.l;
                    (*l.p).sizek
                } as u32)
        );
        unsafe { &mut *k.add(i as usize) }
    }};
}

pub use VM_KV;
