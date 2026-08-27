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
    fn artboard_state_machine(&self, artboard: *const (), index: usize) -> *const ();
    fn artboard_data_binds(&self, artboard: *const ()) -> Vec<*const ()>;
    fn data_bind_target(&self, data_bind: *const ()) -> *mut ();
    fn data_bind_property_key(&self, data_bind: *const ()) -> u16;
    fn artboard_object_index(&self, artboard: *const (), object: *mut ()) -> i32;
    fn artboard_resolve(&mut self, artboard: *mut (), object_id: u32) -> *mut ();
    fn property_field_type(&self, property_key: u16) -> PropertyFieldType;
    fn get_double(&self, object: *const (), property_key: u16) -> f32;
    fn get_color(&self, object: *const (), property_key: u16) -> u32;
    fn get_uint(&self, object: *const (), property_key: u16) -> u32;
    fn get_string(&self, object: *const (), property_key: u16) -> String;
    fn get_bool(&self, object: *const (), property_key: u16) -> bool;
    fn set_double(&mut self, object: *mut (), property_key: u16, value: f32);
    fn set_color(&mut self, object: *mut (), property_key: u16, value: u32);
    fn set_uint(&mut self, object: *mut (), property_key: u16, value: u32);
    fn set_string(&mut self, object: *mut (), property_key: u16, value: String);
    fn set_bool(&mut self, object: *mut (), property_key: u16, value: bool);
    fn state_machine_layer_count(&self, machine: *const ()) -> usize;
    fn state_machine_layer(&self, machine: *const (), index: usize) -> *const ();
    fn state_machine_input_count(&self, machine: *const ()) -> usize;
    fn state_machine_input(&self, machine: *const (), index: usize) -> *const ();
    fn state_machine_input_type(&self, input: *const ()) -> StateMachineInputType;
    fn state_machine_number_value(&self, input: *const ()) -> f32;
    fn state_machine_bool_value(&self, input: *const ()) -> bool;
    fn state_machine_layer_state_count(&self, layer: *const ()) -> usize;
    fn state_machine_layer_state(&self, layer: *const (), index: usize) -> *const ();
    fn layer_state_type(&self, state: *const ()) -> LayerStateType;
    fn animation_state_animation(&self, state: *const ()) -> *const ();
    fn blend_state_animations(&self, state: *const ()) -> Vec<*const ()>;
    fn blend_animation_animation(&self, animation: *const ()) -> *const ();
    fn linear_animation_keyed_object_count(&self, animation: *const ()) -> usize;
    fn linear_animation_keyed_object(&self, animation: *const (), index: usize) -> *const ();
    fn keyed_object_id(&self, keyed_object: *const ()) -> u32;
    fn keyed_object_property_count(&self, keyed_object: *const ()) -> usize;
    fn keyed_object_property(&self, keyed_object: *const (), index: usize) -> *const ();
    fn keyed_property_key(&self, keyed_property: *const ()) -> u16;
    fn state_machine_instance_input(&self, instance: *mut (), index: usize) -> *mut ();
    fn state_machine_instance_input_name(&self, input: *const ()) -> String;
    fn state_machine_instance_set_number(&mut self, instance: *mut (), name: &str, value: f32);
    fn state_machine_instance_set_bool(&mut self, instance: *mut (), name: &str, value: bool);
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
        artboard: *const (),
        runtime: &mut dyn PropertyRecorderRuntime,
    ) {
        let machine = runtime.artboard_state_machine(artboard, 0);
        self.record_state_machine_inputs(machine, runtime);
        self.record_state_machine(machine, runtime);
        self.record_data_binds(artboard, runtime);
        self.write_properties(artboard, runtime);
        self.complete_main();
    }
    fn record_data_binds(&mut self, artboard: *const (), runtime: &dyn PropertyRecorderRuntime) {
        for bind in runtime.artboard_data_binds(artboard) {
            let index = self.get_object_id(artboard, runtime.data_bind_target(bind), runtime);
            if index >= 0 {
                self.add_property_key(index as u32, runtime.data_bind_property_key(bind), runtime);
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
    fn record_state_machine(&mut self, machine: *const (), runtime: &dyn PropertyRecorderRuntime) {
        if machine.is_null() {
            return;
        }
        for i in 0..runtime.state_machine_layer_count(machine) {
            self.record_state_machine_layer(runtime.state_machine_layer(machine, i), runtime);
        }
    }
    pub fn record_state_machine_inputs(
        &mut self,
        machine: *const (),
        runtime: &dyn PropertyRecorderRuntime,
    ) {
        if !machine.is_null() {
            for i in 0..runtime.state_machine_input_count(machine) {
                self.record_state_machine_input(runtime.state_machine_input(machine, i), runtime);
            }
        }
        self.complete_state_machine();
    }
    fn record_state_machine_input(
        &mut self,
        input: *const (),
        runtime: &dyn PropertyRecorderRuntime,
    ) {
        if input.is_null() {
            return;
        }
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
        layer: *const (),
        runtime: &dyn PropertyRecorderRuntime,
    ) {
        for i in 0..runtime.state_machine_layer_state_count(layer) {
            self.record_state_machine_layer_state(
                runtime.state_machine_layer_state(layer, i),
                runtime,
            );
        }
    }
    fn record_state_machine_layer_state(
        &mut self,
        state: *const (),
        runtime: &dyn PropertyRecorderRuntime,
    ) {
        match runtime.layer_state_type(state) {
            LayerStateType::Animation => {
                self.record_linear_animation(runtime.animation_state_animation(state), runtime)
            }
            LayerStateType::Blend => {
                for blend in runtime.blend_state_animations(state) {
                    self.record_linear_animation(runtime.blend_animation_animation(blend), runtime);
                }
            }
            LayerStateType::Other => {}
        }
    }
    fn record_linear_animation(
        &mut self,
        animation: *const (),
        runtime: &dyn PropertyRecorderRuntime,
    ) {
        if animation.is_null() {
            return;
        }
        for i in 0..runtime.linear_animation_keyed_object_count(animation) {
            self.record_keyed_object(runtime.linear_animation_keyed_object(animation, i), runtime);
        }
    }
    fn record_keyed_object(&mut self, object: *const (), runtime: &dyn PropertyRecorderRuntime) {
        if object.is_null() {
            return;
        }
        let id = runtime.keyed_object_id(object);
        self.get_core_object_data(id);
        for i in 0..runtime.keyed_object_property_count(object) {
            let property = runtime.keyed_object_property(object, i);
            self.add_property_key(id, runtime.keyed_property_key(property), runtime);
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
    fn write_properties(&mut self, artboard: *const (), runtime: &mut dyn PropertyRecorderRuntime) {
        for i in 0..self.core_objects_data.len() {
            let id = self.core_objects_data[i].object_id;
            let keys = self.core_objects_data[i].property_keys.clone();
            if keys.is_empty() {
                continue;
            }
            let object = runtime.artboard_resolve(artboard as *mut (), id);
            self.write_object_id(id);
            self.write_total_properties(keys.len() as u32);
            for key in keys {
                match runtime.property_field_type(key) {
                    PropertyFieldType::Double => {
                        self.write_property_key(key as u32);
                        self.write_property_float(runtime.get_double(object, key));
                    }
                    PropertyFieldType::Color => {
                        self.write_property_key(key as u32);
                        self.write_property_uint(runtime.get_color(object, key));
                    }
                    PropertyFieldType::Uint => {
                        self.write_property_key(key as u32);
                        self.write_property_uint(runtime.get_uint(object, key));
                    }
                    PropertyFieldType::String => {
                        self.write_property_key(key as u32);
                        self.write_property_string(runtime.get_string(object, key));
                    }
                    PropertyFieldType::Bool => {
                        self.write_property_key(key as u32);
                        self.write_property_bool(runtime.get_bool(object, key));
                    }
                    PropertyFieldType::Other => {}
                }
            }
        }
    }
    fn get_object_id(
        &self,
        artboard: *const (),
        object: *mut (),
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
        instance: *mut (),
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
            let input = runtime.state_machine_instance_input(instance, index);
            let name = runtime.state_machine_instance_input_name(input);
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
    pub fn apply_artboard(&mut self, artboard: *mut (), runtime: &mut dyn PropertyRecorderRuntime) {
        self.reader_position = 0;
        while self.reader_position < self.reader_end {
            let id = Self::read_var_uint(
                &self.write_buffer,
                &mut self.reader_position,
                self.reader_end,
            );
            let object = runtime.artboard_resolve(artboard, id);
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
                        runtime.set_double(object, key, v);
                    }
                    PropertyFieldType::Color => {
                        let v = Self::read_var_uint(
                            &self.write_buffer,
                            &mut self.reader_position,
                            self.reader_end,
                        );
                        runtime.set_color(object, key, v);
                    }
                    PropertyFieldType::Uint => {
                        let v = Self::read_var_uint(
                            &self.write_buffer,
                            &mut self.reader_position,
                            self.reader_end,
                        );
                        runtime.set_uint(object, key, v);
                    }
                    PropertyFieldType::String => {
                        let v = Self::read_string(
                            &self.write_buffer,
                            &mut self.reader_position,
                            self.reader_end,
                        );
                        runtime.set_string(object, key, v);
                    }
                    PropertyFieldType::Bool => {
                        let v = Self::read_byte(
                            &self.write_buffer,
                            &mut self.reader_position,
                            self.reader_end,
                        );
                        runtime.set_bool(object, key, v != 0);
                    }
                    PropertyFieldType::Other => {}
                }
            }
        }
    }
}
