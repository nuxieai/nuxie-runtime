use crate::records::bc_inst::BcInst;
use crate::records::bytecode_graph_serializer::BytecodeGraphSerializer;

impl<'a> BytecodeGraphSerializer<'a> {
    pub fn get_vm_const_input_aux(&mut self, insn: &mut BcInst, index: u8) -> u32 {
        self.get_vm_const_input_raw(insn, index)
    }
}
