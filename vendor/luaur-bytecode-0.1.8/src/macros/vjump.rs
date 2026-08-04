#[allow(unused_macros)]
macro_rules! VJUMP {
    ($v:expr, $i:expr, $insns:expr, $insnvalid:expr) => {
        LUAU_ASSERT!(
            (($i as isize) + 1 + ($v as isize)) >= 0
                && (($i as isize) + 1 + ($v as isize)) < ($insns.len() as isize)
                && $insnvalid[($i as isize + 1 + ($v as isize)) as usize] != 0
        )
    };
}

pub(crate) use VJUMP;
