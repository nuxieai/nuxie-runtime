#[allow(unused_macros)]
#[macro_export]
macro_rules! INT_IMM {
    ($name:ident, $idx:expr) => {
        #[allow(non_upper_case_globals)]
        pub(crate) const $name: u32 = $idx;

        #[allow(non_snake_case)]
        pub(crate) fn $name(&self) -> i32 {
            self.intImmInput($idx)
        }

        paste::paste! {
            #[allow(non_snake_case)]
            pub(crate) fn [<set $name>](&mut self, value: i32) {
                self.setImmInput($idx, value);
            }
        }
    };
}

pub use INT_IMM;
