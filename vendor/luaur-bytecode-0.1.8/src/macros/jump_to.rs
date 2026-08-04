#[allow(unused_macros)]
#[macro_export]
macro_rules! JUMP_TO {
    ($name:ident, $idx:expr) => {
        #[allow(non_upper_case_globals)]
        pub(crate) const $name: u32 = $idx;

        #[allow(non_snake_case)]
        pub(crate) fn $name(
            &self,
        ) -> crate::records::bc_ref::BcRef<crate::records::bc_block::BcBlock> {
            self.getBlock($idx)
        }

        paste::paste! {
            #[allow(non_snake_case)]
            pub(crate) fn [<set $name>](&mut self, block: crate::records::bc_op::BcOp) {
                self.setBcOp($idx, block);
            }
        }
    };
}

pub use JUMP_TO;
