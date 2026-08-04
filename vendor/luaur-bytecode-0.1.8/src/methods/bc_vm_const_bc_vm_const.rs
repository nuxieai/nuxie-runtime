use crate::enums::bc_vm_const_kind::BcVmConstKind;
use crate::records::bc_vm_const::BcVmConst;

impl BcVmConst {
    pub fn new() -> Self {
        Self {
            kind: BcVmConstKind::Nil,
            value: unsafe { core::mem::zeroed() },
        }
    }
}

impl Default for BcVmConst {
    fn default() -> Self {
        Self::new()
    }
}
