use crate::enums::bc_op_kind::BcOpKind;
use crate::records::bc_function::BcFunction;
use crate::records::bc_op::BcOp;
use crate::records::bc_phi::BcPhi;
use luaur_common::macros::luau_assert::LUAU_ASSERT;
use luaur_common::records::small_vector::SmallVector;

impl<'a> crate::records::bytecode_graph_parser::BytecodeGraphParser<'a> {
    pub fn add_to_phi(&mut self, op: BcOp, proj: BcOp) -> BcOp {
        if op.kind == BcOpKind::Phi {
            let phi = self.func.phi_op(op);
            for &p in &phi.ops {
                if p == proj {
                    return op;
                }
            }
            phi.ops.push_back(proj);
            op
        } else {
            let res = self.func.add_phi();
            let phi = self.func.phi_op(res);
            phi.ops = SmallVector::from_iter([op, proj]);
            res
        }
    }
}
