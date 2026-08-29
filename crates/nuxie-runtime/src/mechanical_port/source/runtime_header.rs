use std::collections::HashMap;

use crate::mechanical_port::source::core::binary_reader::BinaryReader;

pub const RUNTIME_HEADER_FINGERPRINT: &[u8; 4] = b"RIVE";

#[derive(Default)]
pub struct RuntimeHeader {
    major_version: i32,
    minor_version: i32,
    file_id: i32,
    property_to_field_index: HashMap<i32, i32>,
}

impl RuntimeHeader {
    pub fn major_version(&self) -> i32 {
        self.major_version
    }

    pub fn minor_version(&self) -> i32 {
        self.minor_version
    }

    pub fn file_id(&self) -> i32 {
        self.file_id
    }

    pub fn property_field_id(&self, property_key: i32) -> i32 {
        self.property_to_field_index
            .get(&property_key)
            .copied()
            .unwrap_or(-1)
    }

    pub fn read(reader: &mut BinaryReader<'_>, header: &mut Self) -> bool {
        for expected in RUNTIME_HEADER_FINGERPRINT {
            if *expected != reader.read_byte() {
                return false;
            }
        }

        header.major_version = reader.read_var_uint_as::<i32>();
        if reader.did_overflow() {
            return false;
        }
        header.minor_version = reader.read_var_uint_as::<i32>();
        if reader.did_overflow() {
            return false;
        }
        header.file_id = reader.read_var_uint_as::<i32>();
        if reader.did_overflow() {
            return false;
        }

        let mut property_keys = Vec::new();
        loop {
            let property_key = reader.read_var_uint_as::<i32>();
            if property_key == 0 {
                break;
            }
            property_keys.push(property_key);
            if reader.did_overflow() {
                return false;
            }
        }

        let mut current_int = 0u32;
        let mut current_bit = 8;
        for property_key in property_keys {
            if current_bit == 8 {
                current_int = reader.read_uint32();
                current_bit = 0;
            }
            let field_index = ((current_int >> current_bit) & 3) as i32;
            header
                .property_to_field_index
                .insert(property_key, field_index);
            current_bit += 2;
            if reader.did_overflow() {
                return false;
            }
        }
        true
    }
}
