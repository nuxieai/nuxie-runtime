#[allow(unused_macros)]
macro_rules! VCONSTANY {
    ($v:expr, $builder:expr) => {
        if luaur_common::FFlag::LuauVirtualBcBuilder.get() {
            $builder.validate_const($v as i32);
        } else {
            LUAU_ASSERT!(($v as usize) < $builder.constants.len());
        }
    };
}

pub(crate) use VCONSTANY;
