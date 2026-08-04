use crate::records::bc_imm::BcImm;
use crate::records::bc_op::BcOp;
use crate::records::bc_ref::BcRef;
use luaur_common::enums::luau_opcode::LuauOpcode;

pub trait VmConstOps {
    fn evaluate(&self, lhs_op: &BcOp, rhs_op: &BcOp, op: LuauOpcode) -> Option<BcOp>;
    fn falsey(&self, falsey_op: &BcOp) -> bool;
    fn cmp_bc_op(&self, lhs_op: &BcOp, rhs_op: &BcOp) -> i32;
    fn cmp_bc_imm(&self, lhs_op: &BcOp, rhs: &BcImm) -> i32;
    fn make_nil(&self) -> BcOp;
    fn make_imm_bool(&self, value: bool) -> BcImm;
    fn make_imm_int(&self, value: i32) -> BcImm;
    fn is_orderable(&self, vm_const_op: &BcOp) -> bool;
    fn kind_equals(&self, lhs_op: &BcOp, rhs_op: &BcOp) -> bool;
    fn eq_bc_op(&self, lhs_op: &BcOp, rhs_op: &BcOp) -> Option<bool>;
    fn eq_bool(&self, lhs_op: &BcOp, rhs: bool) -> Option<bool>;
    fn eq_int(&self, lhs_op: &BcOp, rhs: i32) -> Option<bool>;
    fn is_arithmetic_constant(&self, vm_const_op: &BcOp) -> bool;
    fn as_number(&self, vm_const_op: &BcOp) -> f64;
    fn as_imm(&self, op: BcOp) -> BcRef<'_, BcImm>;
}
