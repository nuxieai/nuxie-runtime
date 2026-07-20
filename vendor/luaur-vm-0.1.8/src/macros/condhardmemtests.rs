#[allow(non_snake_case)]
macro_rules! condhardmemtests {
    ($x:expr, $l:expr) => {
        #[cfg(feature = "hard_mem_tests")]
        {
            // HARDMEMTESTS is defined via a cfg or a constant in luaconf.h.
            // In the Rust port, we check the level against the configured HARDMEMTESTS value.
            if crate::conf::HARDMEMTESTS >= $l {
                $x;
            }
        }
        #[cfg(not(feature = "hard_mem_tests"))]
        {
            let _ = $l; // Silence unused variable warning for the level
        }
    };
}

pub(crate) use condhardmemtests;
