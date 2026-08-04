use crate::enums::bc_op_kind::BcOpKind;
use crate::records::bc_op::BcOp;
use crate::records::bytecode_graph_parser::BytecodeGraphParser;

impl BytecodeGraphParser<'_> {
    pub fn try_remove_trivial_phi(&mut self, phi_op: BcOp) -> BcOp {
        let mut trivial_value = None;
        for op in &self.func.phis[phi_op.index as usize].ops.clone() {
            if *op == phi_op || trivial_value == Some(*op) {
                continue;
            }
            if trivial_value.is_some() {
                return phi_op;
            }
            trivial_value = Some(*op);
        }

        let reg = *self.func.regs.get(&phi_op).unwrap();
        let trivial_value = trivial_value
            .unwrap_or_else(|| BcOp::bc_op_bc_op_kind_u32(BcOpKind::VmReg, reg as u32));
        let users = core::mem::take(&mut self.func.phis[phi_op.index as usize].uses);

        for user in &users {
            if *user == phi_op {
                continue;
            }

            let mut replacements = 0;
            let user_ops = if user.kind == BcOpKind::Phi {
                &mut self.func.phis[user.index as usize].ops
            } else {
                luaur_common::LUAU_ASSERT!(user.kind == BcOpKind::Inst);
                &mut self.func.instructions[user.index as usize].ops
            };
            for op in user_ops.iter_mut() {
                if *op == phi_op {
                    *op = trivial_value;
                    replacements += 1;
                }
            }
            for _ in 0..replacements {
                self.func.record_use(trivial_value, *user);
            }
        }

        if let Some(block) = self.phi_block.remove(&phi_op) {
            self.func.blocks[block.index as usize]
                .phis
                .retain(|op| *op != phi_op);
            let producers = &mut self.producers[block.index as usize];
            if producers.cached.get(&reg) == Some(&phi_op) {
                producers.cached.insert(reg, trivial_value);
            }
        }

        for user in users {
            if user.kind == BcOpKind::Phi && user != phi_op {
                self.try_remove_trivial_phi(user);
            }
        }

        trivial_value
    }
}
