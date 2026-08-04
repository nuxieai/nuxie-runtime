use crate::enums::bc_block_flag::BcBlockFlag;
use crate::enums::bc_op_kind::BcOpKind;
use crate::functions::has_use::has_use;
use crate::records::bc_function::BcFunction;
use crate::records::bc_op::BcOp;

pub fn verify_use_consistency(function: &BcFunction) -> bool {
    let check_operand = |consumer: BcOp, operand: BcOp| {
        if operand.kind != BcOpKind::Inst && operand.kind != BcOpKind::Phi {
            return true;
        }

        has_use(function, operand, consumer)
    };

    let mut result = true;
    for block in &function.blocks {
        if (block.flags & BcBlockFlag::Dead as u8) != 0 {
            continue;
        }

        for phi_op in &block.phis {
            for operand in &function.phis[phi_op.index as usize].ops {
                result &= check_operand(*phi_op, *operand);
            }
        }

        for inst_op in &block.ops {
            for operand in &function.instructions[inst_op.index as usize].ops {
                result &= check_operand(*inst_op, *operand);
            }
        }
    }

    result
}
