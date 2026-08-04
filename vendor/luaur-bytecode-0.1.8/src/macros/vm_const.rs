#[allow(unused_macros)]
#[macro_export]
macro_rules! VM_CONST {
    ($name:ident, $idx:expr) => {
        #[allow(non_upper_case_globals)]
        pub(crate) const $name: u32 = $idx;

        #[allow(non_snake_case)]
        pub(crate) fn $name(
            &self,
        ) -> crate::records::bc_ref::BcRef<crate::records::vm_const::VmConst> {
            self.getVmConst($idx)
        }

        paste::paste! {
            #[allow(non_snake_case)]
            pub(crate) fn [<set $name>](&mut self, cid: u32) {
                self.setVmConst($idx, cid);
            }
        }
    };
}

pub use VM_CONST;
