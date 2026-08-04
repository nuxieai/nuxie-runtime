use crate::enums::bc_op_kind::BcOpKind;
use crate::records::bc_function::BcFunction;
use crate::records::bc_op::BcOp;
use alloc::vec::Vec;

pub fn uses_of(function: &BcFunction, def: BcOp) -> &Vec<BcOp> {
    if def.kind == BcOpKind::Inst {
        &function.instructions[def.index as usize].uses
    } else {
        luaur_common::LUAU_ASSERT!(def.kind == BcOpKind::Phi);
        &function.phis[def.index as usize].uses
    }
}
