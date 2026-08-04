use crate::enums::bc_imm_kind::BcImmKind;
use crate::enums::bc_op_kind::BcOpKind;
use crate::enums::bc_vm_const_kind::BcVmConstKind;
use crate::records::bc_op::BcOp;
use crate::records::bc_vm_const_impl::BcVmConstImpl;

impl BcVmConstImpl {
    pub fn eq_bc_op(&self, lhs_op: &BcOp, rhs_op: &BcOp) -> Option<bool> {
        let func = unsafe { &mut *self.func };

        if lhs_op.kind == BcOpKind::VmConst && rhs_op.kind == BcOpKind::VmConst {
            let lhs = *func.const_op(*lhs_op);
            let rhs = *func.const_op(*rhs_op);

            unsafe {
                return match (lhs.kind, rhs.kind) {
                    (BcVmConstKind::Number, BcVmConstKind::Number) => {
                        Some(lhs.value.valueNumber == rhs.value.valueNumber)
                    }
                    (BcVmConstKind::Integer, BcVmConstKind::Integer) => {
                        Some(lhs.value.valueInteger == rhs.value.valueInteger)
                    }
                    (BcVmConstKind::Number, BcVmConstKind::Integer) => {
                        Some(lhs.value.valueNumber == rhs.value.valueInteger as f64)
                    }
                    (BcVmConstKind::Integer, BcVmConstKind::Number) => {
                        Some(lhs.value.valueInteger as f64 == rhs.value.valueNumber)
                    }
                    (BcVmConstKind::String, BcVmConstKind::String) => {
                        Some(lhs.value.valueString == rhs.value.valueString)
                    }
                    _ => None,
                };
            }
        }

        if lhs_op.kind == BcOpKind::VmConst && rhs_op.kind == BcOpKind::Imm {
            let lhs = *func.const_op(*lhs_op);
            let rhs = *func.imm_op(*rhs_op);
            if lhs.kind == BcVmConstKind::Boolean && rhs.kind == BcImmKind::Boolean {
                return Some(unsafe { lhs.value.valueBoolean == rhs.value.valueBoolean });
            }
        } else if lhs_op.kind == BcOpKind::Imm && rhs_op.kind == BcOpKind::Imm {
            let lhs = *func.imm_op(*lhs_op);
            let rhs = *func.imm_op(*rhs_op);
            unsafe {
                if lhs.kind == BcImmKind::Boolean && rhs.kind == BcImmKind::Boolean {
                    return Some(lhs.value.valueBoolean == rhs.value.valueBoolean);
                }
                if lhs.kind == BcImmKind::Int && rhs.kind == BcImmKind::Int {
                    return Some(lhs.value.valueInt == rhs.value.valueInt);
                }
            }
        }

        None
    }

    pub fn eq_bool(&self, lhs_op: &BcOp, rhs: bool) -> Option<bool> {
        let lhs = *unsafe { &mut *self.func }.const_op(*lhs_op);
        if lhs.kind == BcVmConstKind::Boolean {
            Some(unsafe { lhs.value.valueBoolean == rhs })
        } else {
            None
        }
    }

    pub fn eq_int(&self, lhs_op: &BcOp, rhs: i32) -> Option<bool> {
        let lhs = *unsafe { &mut *self.func }.const_op(*lhs_op);
        unsafe {
            match lhs.kind {
                BcVmConstKind::Number => Some(rhs as f64 == lhs.value.valueNumber),
                BcVmConstKind::Integer => Some(rhs as i64 == lhs.value.valueInteger),
                _ => None,
            }
        }
    }
}
