#[allow(unused_macros)]
macro_rules! VCONSTANY {
    ($v:expr, $constants:expr) => {
        LUAU_ASSERT!(($v as usize) < $constants.len())
    };
}

pub(crate) use VCONSTANY;
