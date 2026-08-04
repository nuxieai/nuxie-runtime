use crate::enums::bc_imm_kind::BcImmKind;
use crate::enums::bc_op_kind::BcOpKind;
use crate::enums::bc_vm_const_kind::BcVmConstKind;
use crate::records::bc_op::BcOp;
use crate::records::bc_vm_const_impl::BcVmConstImpl;

impl BcVmConstImpl {
    pub fn falsey(&self, falsey_op: &BcOp) -> bool {
        let func = unsafe { &mut *self.func };
        if falsey_op.kind == BcOpKind::VmConst {
            let vm_const = *func.const_op(*falsey_op);
            return vm_const.kind == BcVmConstKind::Nil
                || (vm_const.kind == BcVmConstKind::Boolean
                    && !unsafe { vm_const.value.valueBoolean });
        }
        if falsey_op.kind == BcOpKind::Imm {
            let imm = *func.imm_op(*falsey_op);
            return imm.kind == BcImmKind::Boolean && !unsafe { imm.value.valueBoolean };
        }
        false
    }
}
