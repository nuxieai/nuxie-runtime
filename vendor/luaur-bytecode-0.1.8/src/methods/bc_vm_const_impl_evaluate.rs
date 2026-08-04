use crate::enums::bc_vm_const_kind::BcVmConstKind;
use crate::functions::find_or_add_const::find_or_add_const;
use crate::records::bc_op::BcOp;
use crate::records::bc_vm_const::BcVmConst;
use crate::records::bc_vm_const_impl::BcVmConstImpl;
use luaur_common::enums::luau_opcode::LuauOpcode;

impl BcVmConstImpl {
    pub fn evaluate(&self, lhs_op: &BcOp, rhs_op: &BcOp, op: LuauOpcode) -> Option<BcOp> {
        let func = unsafe { &mut *self.func };
        let lhs = *func.const_op(*lhs_op);
        let rhs = *func.const_op(*rhs_op);

        if lhs.kind != rhs.kind || lhs.kind != BcVmConstKind::Number {
            return None;
        }

        let a = unsafe { lhs.value.valueNumber };
        let b = unsafe { rhs.value.valueNumber };
        let r = match op {
            LuauOpcode::LOP_ADD => a + b,
            LuauOpcode::LOP_SUB => a - b,
            LuauOpcode::LOP_MUL => a * b,
            LuauOpcode::LOP_DIV if b != 0.0 => a / b,
            LuauOpcode::LOP_MOD if b != 0.0 => a - (a / b).floor() * b,
            LuauOpcode::LOP_POW => a.powf(b),
            LuauOpcode::LOP_IDIV if b != 0.0 => (a / b).floor(),
            _ => return None,
        };

        let mut result = BcVmConst::new();
        result.kind = BcVmConstKind::Number;
        result.value.valueNumber = r;
        Some(find_or_add_const(func, &result))
    }
}
