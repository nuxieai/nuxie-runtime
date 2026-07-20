//! Source: `VM/src/lstate.h` — no-op unless HARDSTACKTESTS (off in default builds)
#[allow(non_snake_case)]
#[macro_export]
macro_rules! condhardstacktests {
    ($x:expr) => {
        ()
    };
}
pub use condhardstacktests;
