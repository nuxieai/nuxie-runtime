use crate::records::bc_function::BcFunction;
use crate::records::bc_inst::BcInst;
use crate::records::bc_ref::BcRef;

#[derive(Debug)]
pub struct BcInstHelper<'a> {
    pub(crate) graph: &'a mut BcFunction,
    pub(crate) inst: BcRef<'a, BcInst>,
}

impl<'a> BcInstHelper<'a> {
    pub(crate) fn new(graph: &'a mut BcFunction, inst: BcRef<'a, BcInst>) -> Self {
        Self { graph, inst }
    }

    pub fn detach(&mut self) {
        let block = self.inst.operator_deref().block;
        if block.kind != crate::enums::bc_op_kind::BcOpKind::Block {
            return;
        }
        let op = self.inst.op;
        self.graph.blocks[block.index as usize].ops.retain(|candidate| *candidate != op);
        self.inst.operator_deref_mut().block = crate::records::bc_op::BcOp::default();
    }
}
