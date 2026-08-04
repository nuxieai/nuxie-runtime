#[allow(unused_macros)]
#[macro_export]
macro_rules! BC_OP {
    ($name:ident, $idx:expr) => {
        #[allow(non_upper_case_globals)]
        pub(crate) const $name: u32 = $idx;

        #[allow(non_snake_case)]
        pub(crate) fn $name(&self) -> crate::records::bc_op::BcOp {
            self.getBcOp($idx)
        }

        paste::paste! {
            #[allow(non_snake_case)]
            pub(crate) fn [<set $name>](&mut self, value: crate::records::bc_op::BcOp) {
                self.setBcOp($idx, value);
            }
        }
    };
}

pub use BC_OP;
