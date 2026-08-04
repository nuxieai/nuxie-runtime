use crate::records::bc_inst::BcInst;
use crate::records::bytecode_graph_serializer::BytecodeGraphSerializer;

impl<'a> BytecodeGraphSerializer<'a> {
    pub fn get_vm_const_input_abc(&mut self, insn: &mut BcInst, index: u8) -> u8 {
        let cid: u32 = self.get_vm_const_input_raw(insn, index);

        if cid > 0xff {
            self.error = true;
        }

        cid as u8
    }
}
