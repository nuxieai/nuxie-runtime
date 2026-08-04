use crate::enums::bc_imm_kind::BcImmKind;
use crate::enums::bc_vm_const_kind::BcVmConstKind;
use crate::functions::three_way::three_way;
use crate::records::bc_imm::BcImm;
use crate::records::bc_op::BcOp;
use crate::records::bc_vm_const_impl::BcVmConstImpl;
use core::cmp::Ordering;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

impl BcVmConstImpl {
    pub fn cmp_bc_op(&self, lhs_op: &BcOp, rhs_op: &BcOp) -> i32 {
        let func = unsafe { &mut *self.func };
        let lhs = *func.const_op(*lhs_op);
        let rhs = *func.const_op(*rhs_op);
        LUAU_ASSERT!(lhs.kind == rhs.kind);

        unsafe {
            match lhs.kind {
                BcVmConstKind::Number => three_way(&lhs.value.valueNumber, &rhs.value.valueNumber),
                BcVmConstKind::Integer => {
                    three_way(&lhs.value.valueInteger, &rhs.value.valueInteger)
                }
                BcVmConstKind::Boolean => {
                    i32::from(lhs.value.valueBoolean != rhs.value.valueBoolean)
                }
                BcVmConstKind::String => match lhs.value.valueString.cmp(rhs.value.valueString) {
                    Ordering::Less => -1,
                    Ordering::Equal => 0,
                    Ordering::Greater => 1,
                },
                _ => 0,
            }
        }
    }

    pub fn cmp_bc_imm(&self, lhs_op: &BcOp, rhs: &BcImm) -> i32 {
        let func = unsafe { &mut *self.func };
        let lhs = *func.const_op(*lhs_op);

        unsafe {
            if rhs.kind == BcImmKind::Int {
                if lhs.kind == BcVmConstKind::Number {
                    return three_way(&lhs.value.valueNumber, &(rhs.value.valueInt as f64));
                }
                if lhs.kind == BcVmConstKind::Integer {
                    return three_way(&lhs.value.valueInteger, &(rhs.value.valueInt as i64));
                }
            } else if rhs.kind == BcImmKind::Boolean && lhs.kind == BcVmConstKind::Boolean {
                return i32::from(lhs.value.valueBoolean != rhs.value.valueBoolean);
            }
        }

        LUAU_ASSERT!(false, "incompatible types for immCmpBcVmConst");
        0
    }
}
