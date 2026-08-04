use crate::records::bytecode_builder::BytecodeBuilder;

impl BytecodeBuilder {
    pub fn clear_state(&mut self) {
        self.insns.clear();
        self.lines.clear();
        self.constants.clear();
        self.protos.clear();
        self.jumps.clear();
        self.fb_slots.clear();
        self.table_shapes.clear();

        self.debug_locals.clear();
        self.debug_upvals.clear();

        self.typed_locals.clear();
        self.typed_upvals.clear();

        self.constant_map.clear();
        self.table_shape_map.clear();
        self.proto_map.clear();

        self.debug_remarks.clear();
        self.debug_remark_buffer.clear();
    }
}
