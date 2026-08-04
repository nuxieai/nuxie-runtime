use crate::records::bytecode_builder::BytecodeBuilder;

impl BytecodeBuilder {
    pub fn add_child_function(&mut self, fid: u32) -> i16 {
        if let Some(cache) = self.proto_map.find(&fid) {
            return *cache;
        }

        let id = self.protos.len() as u32;

        const K_MAX_CLOSURE_COUNT: u32 = 0x7fff;
        if id >= K_MAX_CLOSURE_COUNT {
            return -1;
        }

        self.proto_map.try_insert(fid, id as i16);
        self.protos.push(fid);

        id as i16
    }
}
