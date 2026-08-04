#[allow(unused_macros)]
macro_rules! VUPVAL {
    ($v:expr, $func:expr) => {
        LUAU_ASSERT!(($v as u32) < ($func.numupvalues as u32))
    };
}

pub(crate) use VUPVAL;
