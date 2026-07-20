#[macro_export]
macro_rules! LUAU_INSN_AUX_SLOT {
    ($aux:expr) => {
        ($aux) >> 16
    };
}

pub use LUAU_INSN_AUX_SLOT;
