use crate::enums::bc_imm_kind::BcImmKind;
use crate::records::bc_imm::{BcImm, BcImmValue};
use crate::records::bc_vm_const_impl::BcVmConstImpl;

impl BcVmConstImpl {
    pub fn make_imm_bool(&self, value: bool) -> BcImm {
        BcImm {
            kind: BcImmKind::Boolean,
            value: BcImmValue {
                valueBoolean: value,
            },
        }
    }

    pub fn make_imm_int(&self, value: i32) -> BcImm {
        BcImm {
            kind: BcImmKind::Int,
            value: BcImmValue { valueInt: value },
        }
    }
}
