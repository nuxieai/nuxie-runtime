use crate::enums::bc_op_kind::BcOpKind;
use crate::records::bc_op::BcOp;
use crate::records::bc_phi::BcPhi;
use crate::type_aliases::reg::Reg;

use luaur_common::macros::luau_assert::LUAU_ASSERT;
use luaur_common::macros::luau_unreachable::LUAU_UNREACHABLE;

impl<'a> crate::records::bytecode_graph_serializer::BytecodeGraphSerializer<'a> {
    pub fn get_register(&mut self, op: BcOp) -> Reg {
        match op.kind {
            BcOpKind::Phi => {
                // Avoid holding `&mut` to `self.func` while recursively calling `self.get_register`.
                let ops_len;
                let first_op;
                {
                    let phi: &mut BcPhi = self.func.phi_op(op);
                    LUAU_ASSERT!(phi.ops.len() > 0);
                    LUAU_ASSERT!(phi.ops[0] != op);

                    ops_len = phi.ops.len();
                    first_op = phi.ops[0];

                    // Additional assert: all phi operands map to the same register.
                    // We snapshot the operands count now to iterate after recursion.
                }

                let res = self.get_register(first_op);

                let mut i = 0usize;
                loop {
                    if i >= ops_len {
                        break;
                    }

                    let phi_op = {
                        let phi: &mut BcPhi = self.func.phi_op(op);
                        phi.ops[i]
                    };

                    LUAU_ASSERT!(res == self.get_register(phi_op));
                    i += 1;
                }

                res
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
