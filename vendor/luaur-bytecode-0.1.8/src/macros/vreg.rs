#[allow(unused_macros)]
macro_rules! VREG {
    ($v:expr, $func:expr) => {
        LUAU_ASSERT!(($v as u32) < ($func.maxstacksize as u32))
    };
}

pub(crate) use VREG;
