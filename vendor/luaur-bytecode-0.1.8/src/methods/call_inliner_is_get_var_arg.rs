use crate::enums::bc_op_kind::BcOpKind;
use crate::records::bc_inst::BcInst;
use crate::records::bc_op::BcOp;
use crate::records::bc_ref::BcRef;
use crate::records::call_inliner::CallInliner;
use luaur_common::enums::luau_opcode::LuauOpcode;

impl<'a> CallInliner<'a> {
    pub fn is_get_var_arg(&mut self, target_op: BcOp) -> bool {
        if target_op.kind != BcOpKind::Inst {
            return false;
        }
        let inst: BcRef<BcInst> = self.target.inst(target_op);
        inst.operator_deref().op == LuauOpcode::LOP_GETVARARGS
    }
}
