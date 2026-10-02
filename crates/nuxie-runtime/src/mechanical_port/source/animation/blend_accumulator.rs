//! Source-paired translation of animation/blend_accumulator.hpp and .cpp.
use crate::mechanical_port::source::{
    core::CoreHandle, generated::core_registry::CoreRegistry, math::FloatContract,
    shapes::paint::color::color_lerp,
};

struct Value {
    object: CoreHandle,
    property_key: i32,
    is_color: bool,
    frame: u32,
    double_value: f32,
    color_value: u32,
}

pub struct BlendAccumulator {
    values: Vec<Value>,
    index: Vec<u32>,
    written: Vec<u32>,
    frame: u32,
}

impl Default for BlendAccumulator {
    fn default() -> Self {
        Self {
            values: Vec::new(),
            index: Vec::new(),
            written: Vec::new(),
            frame: 1,
        }
    }
}

impl BlendAccumulator {
    fn slot_of(object: &CoreHandle, property_key: i32) -> u32 {
        (((object.slot_address() as u64) ^ ((property_key as u32 as u64) << 48))
            .wrapping_mul(0x9E3779B97F4A7C15)
            >> 32) as u32
    }

    fn find(&self, object: &CoreHandle, property_key: i32) -> u32 {
        if self.index.is_empty() {
            return 0;
        }
        let mask = self.index.len() as u32 - 1;
        let mut i = Self::slot_of(object, property_key) & mask;
        loop {
            let entry = self.index[i as usize];
            if entry == 0 {
                return 0;
            }
            let value = &self.values[(entry - 1) as usize];
            if value.object == *object && value.property_key == property_key {
                return entry;
            }
            i = (i + 1) & mask;
        }
    }

    fn index(&mut self, value_index: u32) {
        let mask = self.index.len() as u32 - 1;
        let value = &self.values[value_index as usize];
        let mut i = Self::slot_of(&value.object, value.property_key) & mask;
        while self.index[i as usize] != 0 {
            i = (i + 1) & mask;
        }
        self.index[i as usize] = value_index + 1;
    }

    fn value(
        &mut self,
        object: &CoreHandle,
        property_key: i32,
        is_color: bool,
        seeding: bool,
    ) -> &mut Value {
        let mut entry = self.find(object, property_key);
        if entry == 0 {
            self.values.push(Value {
                object: object.clone(),
                property_key,
                is_color,
                frame: 0,
                double_value: 0.0,
                color_value: 0,
            });
            if self.values.len() * 2 > self.index.len() {
                self.index = vec![
                    0;
                    if self.index.is_empty() {
                        16
                    } else {
                        self.index.len() * 2
                    }
                ];
                for i in 0..self.values.len() as u32 {
                    self.index(i);
                }
            } else {
                self.index(self.values.len() as u32 - 1);
            }
            entry = self.values.len() as u32;
        }
        let value = &mut self.values[(entry - 1) as usize];
        if value.frame != self.frame {
            value.frame = self.frame;
            self.written.push(entry - 1);
            if !seeding {
                if is_color {
                    value.color_value = CoreRegistry::get_color_handle(object, property_key)
                        .expect("live blend target") as u32;
                } else {
                    value.double_value = CoreRegistry::get_double_handle(object, property_key)
                        .expect("live blend target");
                }
            }
        }
        value
    }

    pub fn seed_double(&mut self, object: &CoreHandle, property_key: i32, value: f32) {
        self.value(object, property_key, false, true).double_value = value;
    }
    pub fn seed_color(&mut self, object: &CoreHandle, property_key: i32, value: u32) {
        self.value(object, property_key, true, true).color_value = value;
    }
    pub fn apply_double(&mut self, object: &CoreHandle, property_key: i32, mix: f32, value: f32) {
        let current = self.value(object, property_key, false, false);
        if mix == 1.0 {
            current.double_value = value;
        } else {
            let mixi = 1.0 - mix;
            current.double_value = current.double_value.contracted_mul_add(mixi, value * mix);
        }
    }
    pub fn apply_color(&mut self, object: &CoreHandle, property_key: i32, mix: f32, value: u32) {
        let current = self.value(object, property_key, true, false);
        current.color_value = if mix == 1.0 {
            value
        } else {
            color_lerp(current.color_value, value, mix)
        };
    }
    pub fn flush(&mut self) {
        for written in &self.written {
            let value = &self.values[*written as usize];
            if value.is_color {
                CoreRegistry::set_color_handle(
                    &value.object,
                    value.property_key,
                    value.color_value as i32,
                );
            } else {
                CoreRegistry::set_double_handle(
                    &value.object,
                    value.property_key,
                    value.double_value,
                );
            }
        }
        self.written.clear();
        self.frame = self.frame.wrapping_add(1);
        if self.frame == 0 {
            for value in &mut self.values {
                value.frame = 0;
            }
            self.frame = 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mechanical_port::source::{
        core::CoreArena, custom_property_number::CustomPropertyNumber,
        generated::custom_property_number_base::CustomPropertyNumberBase,
    };

    #[test]
    fn seeded_mixing_and_flush_preserve_the_mode_specific_value() {
        let arena = CoreArena::default();
        let object = arena.insert(CustomPropertyNumber::default());
        let key = i32::from(CustomPropertyNumberBase::PROPERTY_VALUE_PROPERTY_KEY);
        let current = f32::from_bits(0x3d82_a90a);
        let value = f32::from_bits(0x3f14_7ae1);
        let mix = f32::from_bits(0x3ed0_f81d);
        let mut accumulator = BlendAccumulator::default();
        accumulator.seed_double(&object, key, current);
        accumulator.apply_double(&object, key, mix, value);
        #[cfg(feature = "strict-fp")]
        let expected = current * (1.0 - mix) + value * mix;
        #[cfg(not(feature = "strict-fp"))]
        let expected = current.mul_add(1.0 - mix, value * mix);
        assert_eq!(
            accumulator.values[0].double_value.to_bits(),
            expected.to_bits()
        );
        assert_eq!(accumulator.written, [0]);
        accumulator.flush();
        assert_eq!(
            CoreRegistry::get_double_handle(&object, key)
                .unwrap()
                .to_bits(),
            expected.to_bits()
        );
        assert!(accumulator.written.is_empty());
        assert_eq!(accumulator.frame, 2);
        accumulator.apply_double(&object, key, 1.0, value);
        accumulator.flush();
        assert_eq!(
            CoreRegistry::get_double_handle(&object, key)
                .unwrap()
                .to_bits(),
            value.to_bits()
        );
    }
}
