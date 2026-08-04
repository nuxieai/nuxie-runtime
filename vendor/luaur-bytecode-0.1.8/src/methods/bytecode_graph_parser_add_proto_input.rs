use crate::enums::bc_op_kind::BcOpKind;
use crate::records::bc_inst::BcInst;
use crate::records::bc_op::BcOp;
use crate::records::bytecode_graph_parser::BytecodeGraphParser;

impl<'a> BytecodeGraphParser<'a> {
    pub fn add_proto_input(&mut self, inst: *mut BcInst, idx: u32) {
        let inst = unsafe { &mut *inst };
        inst.ops
            .push_back(BcOp::bc_op_bc_op_kind_u32(BcOpKind::VmProto, idx));
    }
}
