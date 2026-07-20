#[allow(non_snake_case)]
#[macro_export]
macro_rules! VM_REG {
    ($i:expr, $L:expr, $base:expr) => {{
        let i = $i;
        let L = $L;
        let base = $base;
        luaur_common::LUAU_ASSERT!((i as u32) < (unsafe { (*L).top.offset_from(base) } as u32));
        unsafe { &mut *base.add(i as usize) }
    }};
}

pub use VM_REG;
