use crate::records::table_shape::TableShape;
use crate::records::table_shape_hash::TableShapeHash;

impl TableShapeHash {
    #[allow(non_snake_case)]
    pub fn operator_call(&self, v: &TableShape) -> usize {
        // FNV-1a inspired hash (note that we feed integers instead of bytes)
        let mut hash: u32 = 2166136261;

        for i in 0..(v.length as usize) {
            hash ^= v.keys[i] as u32;
            hash = hash.wrapping_mul(16777619);

            // Note: FFlag::LuauCompileDuptableConstantPack2 is assumed true in this translation context
            // as we are translating the logic that depends on the shape's internal state.
            if v.hasConstants {
                hash ^= v.constants[i] as u32;
                hash = hash.wrapping_mul(16777619);
            }
        }

        hash as usize
    }
}
