use super::blend_accumulator::BlendAccumulator;
use crate::mechanical_port::source::core::CoreHandle;

pub trait AnimationResetTarget {
    fn resolve(&self, object_id: u32) -> Option<CoreHandle>;
    fn resolves(&self, object_id: u32) -> bool;
    fn property_field_id(property_key: u32) -> u32;
    fn set_double(&mut self, object_id: u32, property_key: u32, value: f32) -> bool;
    fn set_color(&mut self, object_id: u32, property_key: u32, value: u32) -> bool;
}

#[derive(Default)]
pub struct AnimationReset {
    write_buffer: Vec<u8>,
    complete_len: usize,
}

impl AnimationReset {
    fn write_var_uint(&mut self, mut value: u32) {
        while value >= 0x80 {
            self.write_buffer.push((value as u8) | 0x80);
            value >>= 7;
        }
        self.write_buffer.push(value as u8);
    }
    pub fn write_object_id(&mut self, value: u32) {
        self.write_var_uint(value);
    }
    pub fn write_total_properties(&mut self, value: u32) {
        self.write_var_uint(value);
    }
    pub fn write_property_key(&mut self, value: u32) {
        self.write_var_uint(value);
    }
    pub fn write_property_value(&mut self, value: f32) {
        self.write_buffer.extend(value.to_le_bytes());
    }
    pub fn clear(&mut self) {
        self.write_buffer.clear();
        self.complete_len = 0;
    }
    pub fn complete(&mut self) {
        if !self.write_buffer.is_empty() {
            self.complete_len = self.write_buffer.len();
        }
    }

    fn read_var_uint(bytes: &[u8], position: &mut usize) -> u32 {
        let mut value = 0u32;
        let mut shift = 0;
        loop {
            let byte = bytes[*position];
            *position += 1;
            value |= ((byte & 0x7f) as u32) << shift;
            if byte & 0x80 == 0 {
                return value;
            }
            shift += 7;
        }
    }

    fn read_float32(bytes: &[u8], position: &mut usize) -> f32 {
        let Some(end) = position.checked_add(4) else {
            *position = bytes.len();
            return 0.0;
        };
        let Some(encoded) = bytes.get(*position..end) else {
            *position = bytes.len();
            return 0.0;
        };
        *position = end;
        f32::from_le_bytes(encoded.try_into().unwrap())
    }

    fn for_each_value(&self, mut visit: impl FnMut(u32, u32, f32)) {
        if self.write_buffer.is_empty() {
            return;
        }
        let end = if self.complete_len == 0 {
            self.write_buffer.len()
        } else {
            self.complete_len
        };
        let mut position = 0;
        while position < end {
            let object_id = Self::read_var_uint(&self.write_buffer, &mut position);
            let property_count = Self::read_var_uint(&self.write_buffer, &mut position);
            for _ in 0..property_count {
                let property_key = Self::read_var_uint(&self.write_buffer, &mut position);
                let value = Self::read_float32(&self.write_buffer[..end], &mut position);
                visit(object_id, property_key, value);
            }
        }
    }

    pub fn apply<T: AnimationResetTarget>(&self, artboard: &mut T) {
        self.for_each_value(|object_id, property_key, value| {
            let field_id = T::property_field_id(property_key);
            assert!(
                artboard.resolves(object_id),
                "AnimationReset resolved a missing object"
            );
            if field_id == 2 {
                artboard.set_double(object_id, property_key, value);
            } else if field_id == 3 {
                // CoreRegistry::setColor accepts a signed C++ int. Preserve
                // that conversion before returning the packed color bits.
                artboard.set_color(object_id, property_key, value as i32 as u32);
            }
        });
    }

    pub fn seed<T: AnimationResetTarget>(&self, artboard: &T, accumulator: &mut BlendAccumulator) {
        self.for_each_value(|object_id, property_key, value| {
            let object = artboard
                .resolve(object_id)
                .expect("AnimationReset resolved a missing object");
            match T::property_field_id(property_key) {
                2 => accumulator.seed_double(&object, property_key as i32, value),
                3 => accumulator.seed_color(&object, property_key as i32, value as i32 as u32),
                _ => (),
            }
        });
    }
}
