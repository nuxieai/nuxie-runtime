pub trait AnimationResetTarget {
    type Object;
    fn resolve(&mut self, object_id: u32) -> &mut Self::Object;
    fn property_field_id(property_key: u32) -> u32;
    fn set_double(object: &mut Self::Object, property_key: u32, value: f32);
    fn set_color(object: &mut Self::Object, property_key: u32, value: u32);
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

    pub fn apply<T: AnimationResetTarget>(&self, artboard: &mut T) {
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
                let value = f32::from_le_bytes(
                    self.write_buffer[position..position + 4]
                        .try_into()
                        .unwrap(),
                );
                position += 4;
                let field_id = T::property_field_id(property_key);
                let object = artboard.resolve(object_id);
                if field_id == 2 {
                    T::set_double(object, property_key, value);
                } else if field_id == 3 {
                    T::set_color(object, property_key, value as u32);
                }
            }
        }
    }
}
