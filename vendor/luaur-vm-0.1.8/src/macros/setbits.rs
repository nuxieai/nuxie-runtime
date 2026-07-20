#[allow(non_snake_case)]
#[macro_export]
macro_rules! setbits {
    ($x:expr, $m:expr) => {
        // C promotes to int before |, then truncates on store
        $x = (($x as i32) | (($m) as i32)) as _
    };
}

pub use setbits;
