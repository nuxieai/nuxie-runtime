use crate::records::bytecode_builder::BytecodeBuilder;

impl BytecodeBuilder {
    pub fn decompose_import_id(ids: u32, id0: &mut i32, id1: &mut i32, id2: &mut i32) -> i32 {
        let count = (ids >> 30) as i32;
        *id0 = if count > 0 {
            (ids >> 20) as i32 & 1023
        } else {
            -1
        };
        *id1 = if count > 1 {
            (ids >> 10) as i32 & 1023
        } else {
            -1
        };
        *id2 = if count > 2 { ids as i32 & 1023 } else { -1 };
        count
    }
}
