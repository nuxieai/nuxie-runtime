#[allow(unused_macros)]
macro_rules! VREGRANGE {
    ($v:expr, $count:expr, $func:expr) => {
        LUAU_ASSERT!(
            (($v as i32)
                + (if ($count as i32) < 0 {
                    0
                } else {
                    $count as i32
                })) as u32
                <= ($func.maxstacksize as u32)
        )
    };
}

pub(crate) use VREGRANGE;
