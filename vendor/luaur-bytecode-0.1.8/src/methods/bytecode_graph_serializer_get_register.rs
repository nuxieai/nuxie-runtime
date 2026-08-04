use crate::enums::bc_op_kind::BcOpKind;
use crate::records::bc_op::BcOp;
use crate::type_aliases::reg::Reg;

use luaur_common::macros::luau_assert::LUAU_ASSERT;
use luaur_common::macros::luau_unreachable::LUAU_UNREACHABLE;

impl<'a> crate::records::bytecode_graph_serializer::BytecodeGraphSerializer<'a> {
    pub fn get_register(&mut self, op: BcOp) -> Reg {
        match op.kind {
            BcOpKind::Phi => {
                LUAU_ASSERT!(!self.func.phis[op.index as usize].ops.is_empty());
                if let Some(reg) = self.func.regs.get(&op) {
                    return *reg;
                }

                let first_op = self.func.phis[op.index as usize].ops[0];
                LUAU_ASSERT!(first_op != op);
                self.get_register(first_op)
            }
            BcOpKind::Inst => {
                let res = self.func.regs.get(&op);
                LUAU_ASSERT!(res.is_some());
                *res.unwrap()
            }
            BcOpKind::Proj => {
                // Avoid holding `&mut` to `self.func` while recursively calling `self.get_register`.
                let proj = {
                    let proj: &mut crate::records::bc_proj::BcProj = self.func.proj_op(op);
                    *proj
                };
                let base = self.get_register(proj.op);
                base + proj.index as Reg
            }
            BcOpKind::VmReg => op.index as Reg,
            _ => {
                LUAU_UNREACHABLE!();
            }
        }
    }
}
