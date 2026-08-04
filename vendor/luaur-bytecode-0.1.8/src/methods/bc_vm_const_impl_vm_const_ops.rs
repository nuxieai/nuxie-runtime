use crate::records::bc_imm::BcImm;
use crate::records::bc_op::BcOp;
use crate::records::bc_ref::BcRef;
use crate::records::bc_vm_const_impl::BcVmConstImpl;
use crate::records::vm_const_ops::VmConstOps;
use luaur_common::enums::luau_opcode::LuauOpcode;

impl VmConstOps for BcVmConstImpl {
    fn evaluate(&self, lhs_op: &BcOp, rhs_op: &BcOp, op: LuauOpcode) -> Option<BcOp> {
        BcVmConstImpl::evaluate(self, lhs_op, rhs_op, op)
    }

    fn falsey(&self, falsey_op: &BcOp) -> bool {
        BcVmConstImpl::falsey(self, falsey_op)
    }

    fn cmp_bc_op(&self, lhs_op: &BcOp, rhs_op: &BcOp) -> i32 {
        BcVmConstImpl::cmp_bc_op(self, lhs_op, rhs_op)
    }

    fn cmp_bc_imm(&self, lhs_op: &BcOp, rhs: &BcImm) -> i32 {
        BcVmConstImpl::cmp_bc_imm(self, lhs_op, rhs)
    }

    fn make_nil(&self) -> BcOp {
        BcVmConstImpl::make_nil(self)
    }

    fn make_imm_bool(&self, value: bool) -> BcImm {
        BcVmConstImpl::make_imm_bool(self, value)
    }

    fn make_imm_int(&self, value: i32) -> BcImm {
        BcVmConstImpl::make_imm_int(self, value)
    }

    fn is_orderable(&self, vm_const_op: &BcOp) -> bool {
        BcVmConstImpl::is_orderable(self, vm_const_op)
    }

    fn kind_equals(&self, lhs_op: &BcOp, rhs_op: &BcOp) -> bool {
        BcVmConstImpl::kind_equals(self, lhs_op, rhs_op)
    }

    fn eq_bc_op(&self, lhs_op: &BcOp, rhs_op: &BcOp) -> Option<bool> {
        BcVmConstImpl::eq_bc_op(self, lhs_op, rhs_op)
    }

    fn eq_bool(&self, lhs_op: &BcOp, rhs: bool) -> Option<bool> {
        BcVmConstImpl::eq_bool(self, lhs_op, rhs)
    }

    fn eq_int(&self, lhs_op: &BcOp, rhs: i32) -> Option<bool> {
        BcVmConstImpl::eq_int(self, lhs_op, rhs)
    }

    fn is_arithmetic_constant(&self, vm_const_op: &BcOp) -> bool {
        BcVmConstImpl::is_arithmetic_constant(self, vm_const_op)
    }

    fn as_number(&self, vm_const_op: &BcOp) -> f64 {
        BcVmConstImpl::as_number(self, vm_const_op)
    }

    fn as_imm(&self, op: BcOp) -> BcRef<'_, BcImm> {
        BcVmConstImpl::as_imm(self, op)
    }
}
