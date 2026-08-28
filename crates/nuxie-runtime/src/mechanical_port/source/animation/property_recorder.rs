#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PropertyFieldType {
    Double,
    Color,
    Uint,
    String,
    Bool,
    Other,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StateMachineInputType {
    Number,
    Bool,
    Trigger,
    Other,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LayerStateType {
    Animation,
    Blend,
    Other,
}

pub struct CoreObjectData {
    property_keys: Vec<u16>,
    pub object_id: u32,
}

impl CoreObjectData {
    pub fn new(id: u32) -> Self {
        Self {
            property_keys: Vec::new(),
            object_id: id,
        }
    }
    pub fn add_property_key(&mut self, key: u16) {
        if !self.property_keys.contains(&key) {
            self.property_keys.push(key);
        }
    }
    pub fn property_keys(&mut self) -> &mut Vec<u16> {
        &mut self.property_keys
    }
}

pub trait PropertyRecorderRuntime {
    fn artboard_state_machine(&self, artboard: &CoreHandle, index: usize) -> Option<CoreHandle>;
    fn artboard_data_binds(&self, artboard: &CoreHandle) -> Vec<CoreHandle>;
    fn data_bind_target(&self, data_bind: &CoreHandle) -> Option<CoreHandle>;
    fn data_bind_property_key(&self, data_bind: &CoreHandle) -> u16;
    fn artboard_object_index(&self, artboard: &CoreHandle, object: &CoreHandle) -> i32;
    fn artboard_resolve(&mut self, artboard: &CoreHandle, object_id: u32) -> Option<CoreHandle>;
    fn artboard_instance_resolve(
        &mut self,
        artboard: &RuntimeArtboardInstanceHandle,
        object_id: u32,
    ) -> Option<CoreHandle>;
    fn property_field_type(&self, property_key: u16) -> PropertyFieldType;
    fn get_double(&self, object: &CoreHandle, property_key: u16) -> f32;
    fn get_color(&self, object: &CoreHandle, property_key: u16) -> u32;
    fn get_uint(&self, object: &CoreHandle, property_key: u16) -> u32;
    fn get_string(&self, object: &CoreHandle, property_key: u16) -> String;
    fn get_bool(&self, object: &CoreHandle, property_key: u16) -> bool;
    fn set_double(&mut self, object: &CoreHandle, property_key: u16, value: f32);
    fn set_color(&mut self, object: &CoreHandle, property_key: u16, value: u32);
    fn set_uint(&mut self, object: &CoreHandle, property_key: u16, value: u32);
    fn set_string(&mut self, object: &CoreHandle, property_key: u16, value: String);
    fn set_bool(&mut self, object: &CoreHandle, property_key: u16, value: bool);
    fn state_machine_layer_count(&self, machine: &CoreHandle) -> usize;
    fn state_machine_layer(&self, machine: &CoreHandle, index: usize) -> Option<CoreHandle>;
    fn state_machine_input_count(&self, machine: &CoreHandle) -> usize;
    fn state_machine_input(&self, machine: &CoreHandle, index: usize) -> Option<CoreHandle>;
    fn state_machine_input_type(&self, input: &CoreHandle) -> StateMachineInputType;
    fn state_machine_number_value(&self, input: &CoreHandle) -> f32;
    fn state_machine_bool_value(&self, input: &CoreHandle) -> bool;
    fn state_machine_layer_state_count(&self, layer: &CoreHandle) -> usize;
    fn state_machine_layer_state(&self, layer: &CoreHandle, index: usize) -> Option<CoreHandle>;
    fn layer_state_type(&self, state: &CoreHandle) -> LayerStateType;
    fn animation_state_animation(&self, state: &CoreHandle) -> Option<CoreHandle>;
    fn blend_state_animations(&self, state: &CoreHandle) -> Vec<CoreHandle>;
    fn blend_animation_animation(&self, animation: &CoreHandle) -> Option<CoreHandle>;
    fn linear_animation_keyed_object_count(&self, animation: &CoreHandle) -> usize;
    fn linear_animation_keyed_object(
        &self,
        animation: &CoreHandle,
        index: usize,
    ) -> Option<CoreHandle>;
    fn keyed_object_id(&self, keyed_object: &CoreHandle) -> u32;
    fn keyed_object_property_count(&self, keyed_object: &CoreHandle) -> usize;
    fn keyed_object_property(&self, keyed_object: &CoreHandle, index: usize) -> Option<CoreHandle>;
    fn keyed_property_key(&self, keyed_property: &CoreHandle) -> u16;
    fn state_machine_instance_input_name(
        &self,
        instance: &RuntimeStateMachineInstanceHandle,
        index: usize,
    ) -> Option<String>;
    fn state_machine_instance_set_number(
        &mut self,
        instance: &RuntimeStateMachineInstanceHandle,
        name: &str,
        value: f32,
    );
    fn state_machine_instance_set_bool(
        &mut self,
        instance: &RuntimeStateMachineInstanceHandle,
        name: &str,
        value: bool,
    );
}

pub struct PropertyRecorder {
    write_buffer: Vec<u8>,
    writer_position: usize,
    reader_position: usize,
    reader_end: usize,
    write_buffer_sm: Vec<u8>,
    writer_position_sm: usize,
    reader_position_sm: usize,
    reader_end_sm: usize,
    core_objects_data: Vec<Box<CoreObjectData>>,
}

impl Default for PropertyRecorder {
    fn default() -> Self {
        Self::new()
    }
}

impl PropertyRecorder {
    pub fn new() -> Self {
        Self {
            write_buffer: Vec::new(),
            writer_position: 0,
            reader_position: 0,
            reader_end: 0,
            write_buffer_sm: Vec::new(),
            writer_position_sm: 0,
            reader_position_sm: 0,
            reader_end_sm: 0,
            core_objects_data: Vec::new(),
        }
    }
    fn write_bytes(buffer: &mut Vec<u8>, position: &mut usize, bytes: &[u8]) {
        let end = *position + bytes.len();
        if buffer.len() < end {
            buffer.resize(end, 0);
        }
        buffer[*position..end].copy_from_slice(bytes);
        *position = end;
    }
    fn write_var_uint(buffer: &mut Vec<u8>, position: &mut usize, mut value: u32) {
        loop {
            let mut byte = (value & 0x7f) as u8;
            value >>= 7;
            if value != 0 {
                byte |= 0x80;
            }
            Self::write_bytes(buffer, position, &[byte]);
            if value == 0 {
                break;
            }
        }
    }
    fn write_object_id(&mut self, value: u32) {
        Self::write_var_uint(&mut self.write_buffer, &mut self.writer_position, value);
    }
    fn write_total_properties(&mut self, value: u32) {
        Self::write_var_uint(&mut self.write_buffer, &mut self.writer_position, value);
    }
    fn write_property_key(&mut self, value: u32) {
        Self::write_var_uint(&mut self.write_buffer, &mut self.writer_position, value);
    }
    fn write_property_float(&mut self, value: f32) {
        Self::write_bytes(
            &mut self.write_buffer,
            &mut self.writer_position,
            &value.to_le_bytes(),
        );
    }
    fn write_property_int_sm(&mut self, value: i32) {
        Self::write_var_uint(
            &mut self.write_buffer_sm,
            &mut self.writer_position_sm,
            value as u32,
        );
    }
    fn write_property_float_sm(&mut self, value: f32) {
        Self::write_bytes(
            &mut self.write_buffer_sm,
            &mut self.writer_position_sm,
            &value.to_le_bytes(),
        );
    }
    fn write_property_uint(&mut self, value: u32) {
        Self::write_var_uint(&mut self.write_buffer, &mut self.writer_position, value);
    }
    fn write_property_string(&mut self, value: String) {
        Self::write_var_uint(
            &mut self.write_buffer,
            &mut self.writer_position,
            value.len() as u32,
        );
        Self::write_bytes(
            &mut self.write_buffer,
            &mut self.writer_position,
            value.as_bytes(),
        );
    }
    fn write_property_bool(&mut self, value: bool) {
        Self::write_bytes(
            &mut self.write_buffer,
            &mut self.writer_position,
            &[value as u8],
        );
    }
    fn write_property_bool_sm(&mut self, value: bool) {
        Self::write_bytes(
            &mut self.write_buffer_sm,
            &mut self.writer_position_sm,
            &[value as u8],
        );
    }
    pub fn clear(&mut self) {
        self.writer_position = 0;
    }
    fn complete_main(&mut self) {
        self.reader_position = 0;
        self.reader_end = self.writer_position;
    }
    fn complete_state_machine(&mut self) {
        self.reader_position_sm = 0;
        self.reader_end_sm = self.writer_position_sm;
    }

    pub fn record_artboard(
        &mut self,
        artboard: &CoreHandle,
        runtime: &mut dyn PropertyRecorderRuntime,
    ) {
        let machine = runtime.artboard_state_machine(artboard, 0);
        self.record_state_machine_inputs(machine.as_ref(), runtime);
        self.record_state_machine(machine.as_ref(), runtime);
        self.record_data_binds(artboard, runtime);
        self.write_properties(artboard, runtime);
        self.complete_main();
    }
    fn record_data_binds(&mut self, artboard: &CoreHandle, runtime: &dyn PropertyRecorderRuntime) {
        for bind in runtime.artboard_data_binds(artboard) {
            let Some(target) = runtime.data_bind_target(&bind) else {
                continue;
            };
            let index = self.get_object_id(artboard, &target, runtime);
            if index >= 0 {
                self.add_property_key(index as u32, runtime.data_bind_property_key(&bind), runtime);
            }
        }
    }
    fn add_property_key(
        &mut self,
        object_id: u32,
        key: u16,
        runtime: &dyn PropertyRecorderRuntime,
    ) {
        if matches!(
            runtime.property_field_type(key),
            PropertyFieldType::Double
                | PropertyFieldType::Color
                | PropertyFieldType::Uint
                | PropertyFieldType::String
                | PropertyFieldType::Bool
        ) {
            self.get_core_object_data(object_id).add_property_key(key);
        }
    }
    fn record_state_machine(
        &mut self,
        machine: Option<&CoreHandle>,
        runtime: &dyn PropertyRecorderRuntime,
    ) {
        let Some(machine) = machine else {
            return;
        };
        for i in 0..runtime.state_machine_layer_count(machine) {
            if let Some(layer) = runtime.state_machine_layer(machine, i) {
                self.record_state_machine_layer(&layer, runtime);
            }
        }
    }
    pub fn record_state_machine_inputs(
        &mut self,
        machine: Option<&CoreHandle>,
        runtime: &dyn PropertyRecorderRuntime,
    ) {
        if let Some(machine) = machine {
            for i in 0..runtime.state_machine_input_count(machine) {
                if let Some(input) = runtime.state_machine_input(machine, i) {
                    self.record_state_machine_input(&input, runtime);
                }
            }
        }
        self.complete_state_machine();
    }
    fn record_state_machine_input(
        &mut self,
        input: &CoreHandle,
        runtime: &dyn PropertyRecorderRuntime,
    ) {
        match runtime.state_machine_input_type(input) {
            StateMachineInputType::Number => {
                self.write_property_int_sm(0);
                self.write_property_float_sm(runtime.state_machine_number_value(input));
            }
            StateMachineInputType::Bool => {
                self.write_property_int_sm(1);
                self.write_property_bool_sm(runtime.state_machine_bool_value(input));
            }
            StateMachineInputType::Trigger => self.write_property_int_sm(2),
            StateMachineInputType::Other => {}
        }
    }
    fn record_state_machine_layer(
        &mut self,
        layer: &CoreHandle,
        runtime: &dyn PropertyRecorderRuntime,
    ) {
        for i in 0..runtime.state_machine_layer_state_count(layer) {
            if let Some(state) = runtime.state_machine_layer_state(layer, i) {
                self.record_state_machine_layer_state(&state, runtime);
            }
        }
    }
    fn record_state_machine_layer_state(
        &mut self,
        state: &CoreHandle,
        runtime: &dyn PropertyRecorderRuntime,
    ) {
        match runtime.layer_state_type(state) {
            LayerStateType::Animation => self.record_linear_animation(
                runtime.animation_state_animation(state).as_ref(),
                runtime,
            ),
            LayerStateType::Blend => {
                for blend in runtime.blend_state_animations(state) {
                    self.record_linear_animation(
                        runtime.blend_animation_animation(&blend).as_ref(),
                        runtime,
                    );
                }
            }
            LayerStateType::Other => {}
        }
    }
    fn record_linear_animation(
        &mut self,
        animation: Option<&CoreHandle>,
        runtime: &dyn PropertyRecorderRuntime,
    ) {
        let Some(animation) = animation else {
            return;
        };
        for i in 0..runtime.linear_animation_keyed_object_count(animation) {
            if let Some(object) = runtime.linear_animation_keyed_object(animation, i) {
                self.record_keyed_object(&object, runtime);
            }
        }
    }
    fn record_keyed_object(&mut self, object: &CoreHandle, runtime: &dyn PropertyRecorderRuntime) {
        let id = runtime.keyed_object_id(object);
        self.get_core_object_data(id);
        for i in 0..runtime.keyed_object_property_count(object) {
            if let Some(property) = runtime.keyed_object_property(object, i) {
                self.add_property_key(id, runtime.keyed_property_key(&property), runtime);
            }
        }
    }
    fn get_core_object_data(&mut self, id: u32) -> &mut CoreObjectData {
        if let Some(i) = self
            .core_objects_data
            .iter()
            .position(|v| v.object_id == id)
        {
            return &mut self.core_objects_data[i];
        }
        self.core_objects_data
            .push(Box::new(CoreObjectData::new(id)));
        self.core_objects_data.last_mut().unwrap()
    }
    fn write_properties(
        &mut self,
        artboard: &CoreHandle,
        runtime: &mut dyn PropertyRecorderRuntime,
    ) {
        for i in 0..self.core_objects_data.len() {
            let id = self.core_objects_data[i].object_id;
            let keys = self.core_objects_data[i].property_keys.clone();
            if keys.is_empty() {
                continue;
            }
            let Some(object) = runtime.artboard_resolve(artboard, id) else {
                continue;
            };
            self.write_object_id(id);
            self.write_total_properties(keys.len() as u32);
            for key in keys {
                match runtime.property_field_type(key) {
                    PropertyFieldType::Double => {
                        self.write_property_key(key as u32);
                        self.write_property_float(runtime.get_double(&object, key));
                    }
                    PropertyFieldType::Color => {
                        self.write_property_key(key as u32);
                        self.write_property_uint(runtime.get_color(&object, key));
                    }
                    PropertyFieldType::Uint => {
                        self.write_property_key(key as u32);
                        self.write_property_uint(runtime.get_uint(&object, key));
                    }
                    PropertyFieldType::String => {
                        self.write_property_key(key as u32);
                        self.write_property_string(runtime.get_string(&object, key));
                    }
                    PropertyFieldType::Bool => {
                        self.write_property_key(key as u32);
                        self.write_property_bool(runtime.get_bool(&object, key));
                    }
                    PropertyFieldType::Other => {}
                }
            }
        }
    }
    fn get_object_id(
        &self,
        artboard: &CoreHandle,
        object: &CoreHandle,
        runtime: &dyn PropertyRecorderRuntime,
    ) -> i32 {
        runtime.artboard_object_index(artboard, object)
    }
    fn read_byte(buffer: &[u8], position: &mut usize, end: usize) -> u8 {
        if *position >= end {
            0
        } else {
            let v = buffer[*position];
            *position += 1;
            v
        }
    }
    fn read_var_uint(buffer: &[u8], position: &mut usize, end: usize) -> u32 {
        let mut value = 0;
        let mut shift = 0;
        loop {
            let byte = Self::read_byte(buffer, position, end);
            value |= u32::from(byte & 0x7f) << shift;
            if byte & 0x80 == 0 || shift >= 28 {
                return value;
            }
            shift += 7;
        }
    }
    fn read_float(buffer: &[u8], position: &mut usize, end: usize) -> f32 {
        let mut b = [0; 4];
        for v in &mut b {
            *v = Self::read_byte(buffer, position, end);
        }
        f32::from_le_bytes(b)
    }
    fn read_string(buffer: &[u8], position: &mut usize, end: usize) -> String {
        let len = Self::read_var_uint(buffer, position, end) as usize;
        let len = len.min(end.saturating_sub(*position));
        let value = String::from_utf8_lossy(&buffer[*position..*position + len]).into_owned();
        *position += len;
        value
    }
    pub fn apply_state_machine(
        &mut self,
        instance: &RuntimeStateMachineInstanceHandle,
        runtime: &mut dyn PropertyRecorderRuntime,
    ) {
        let mut index = 0;
        self.reader_position_sm = 0;
        while self.reader_position_sm < self.reader_end_sm && index < 20 {
            let kind = Self::read_var_uint(
                &self.write_buffer_sm,
                &mut self.reader_position_sm,
                self.reader_end_sm,
            );
            let Some(name) = runtime.state_machine_instance_input_name(instance, index) else {
                break;
            };
            if kind == 0 {
                let value = Self::read_float(
                    &self.write_buffer_sm,
                    &mut self.reader_position_sm,
                    self.reader_end_sm,
                );
                runtime.state_machine_instance_set_number(instance, &name, value);
            } else if kind == 1 {
                let value = Self::read_byte(
                    &self.write_buffer_sm,
                    &mut self.reader_position_sm,
                    self.reader_end_sm,
                );
                runtime.state_machine_instance_set_bool(instance, &name, value != 0);
            }
            index += 1;
        }
    }
    pub fn apply_artboard(
        &mut self,
        artboard: &RuntimeArtboardInstanceHandle,
        runtime: &mut dyn PropertyRecorderRuntime,
    ) {
        self.reader_position = 0;
        while self.reader_position < self.reader_end {
            let id = Self::read_var_uint(
                &self.write_buffer,
                &mut self.reader_position,
                self.reader_end,
            );
            let Some(object) = runtime.artboard_instance_resolve(artboard, id) else {
                break;
            };
            let total = Self::read_var_uint(
                &self.write_buffer,
                &mut self.reader_position,
                self.reader_end,
            );
            for _ in 0..total {
                let key = Self::read_var_uint(
                    &self.write_buffer,
                    &mut self.reader_position,
                    self.reader_end,
                ) as u16;
                match runtime.property_field_type(key) {
                    PropertyFieldType::Double => {
                        let v = Self::read_float(
                            &self.write_buffer,
                            &mut self.reader_position,
                            self.reader_end,
                        );
                        runtime.set_double(&object, key, v);
                    }
                    PropertyFieldType::Color => {
                        let v = Self::read_var_uint(
                            &self.write_buffer,
                            &mut self.reader_position,
                            self.reader_end,
                        );
                        runtime.set_color(&object, key, v);
                    }
                    PropertyFieldType::Uint => {
                        let v = Self::read_var_uint(
                            &self.write_buffer,
                            &mut self.reader_position,
                            self.reader_end,
                        );
                        runtime.set_uint(&object, key, v);
                    }
                    PropertyFieldType::String => {
                        let v = Self::read_string(
                            &self.write_buffer,
                            &mut self.reader_position,
                            self.reader_end,
                        );
                        runtime.set_string(&object, key, v);
                    }
                    PropertyFieldType::Bool => {
                        let v = Self::read_byte(
                            &self.write_buffer,
                            &mut self.reader_position,
                            self.reader_end,
                        );
                        runtime.set_bool(&object, key, v != 0);
                    }
                    PropertyFieldType::Other => {}
                }
            }
        }
    }
}
use crate::mechanical_port::source::{
    animation::state_machine_instance::RuntimeStateMachineInstanceHandle,
    artboard::RuntimeArtboardInstanceHandle, core::CoreHandle,
};
