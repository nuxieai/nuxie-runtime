use crate::records::bc_inst::BcInst;
use crate::records::bytecode_graph_serializer::BytecodeGraphSerializer;

impl<'a> BytecodeGraphSerializer<'a> {
    pub fn get_vm_const_input_d(&mut self, insn: &mut BcInst, index: u8) -> u16 {
        let cid: u32 = self.get_vm_const_input_raw(insn, index);

        if cid > 0xffff {
            self.error = true;
        }

        cid as u16
    }
}
