use crate::mechanical_port::source::{
    animation::{
        animation_state::AnimationState, keyed_object::KeyedObject, keyed_property::KeyedProperty,
        linear_animation::LinearAnimation, state_machine::StateMachine,
        state_machine_bool::StateMachineBool,
        state_machine_instance::RuntimeStateMachineInstanceHandle,
        state_machine_layer::StateMachineLayer, state_machine_number::StateMachineNumber,
    },
    artboard::{Artboard, RuntimeArtboardInstanceHandle},
    core::{
        CoreHandle,
        field_types::{
            core_bool_type::CoreBoolType, core_color_type::CoreColorType,
            core_double_type::CoreDoubleType, core_string_type::CoreStringType,
            core_uint_type::CoreUintType,
        },
    },
    generated::{
        animation::{
            state_machine_bool_base::StateMachineBoolBase,
            state_machine_number_base::StateMachineNumberBase,
            state_machine_trigger_base::StateMachineTriggerBase,
        },
        core_registry::CoreRegistry,
    },
};

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

    pub fn record_artboard(&mut self, artboard: &CoreHandle) {
        let machine = artboard
            .with_downcast::<Artboard, _>(|artboard| artboard.state_machine_handle_at(0))
            .expect("PropertyRecorder records an Artboard occurrence");
        self.record_state_machine_inputs(machine.as_ref());
        self.record_state_machine(machine.as_ref());
        self.record_data_binds(artboard);
        self.write_properties(artboard);
        self.complete_main();
    }

    fn record_data_binds(&mut self, artboard: &CoreHandle) {
        let bindings = artboard
            .with_downcast::<Artboard, _>(Artboard::data_bind_handles)
            .expect("PropertyRecorder records an Artboard occurrence");
        for binding in bindings {
            let (target, key) = binding
                .with(|binding| {
                    let binding = binding.as_data_bind().expect("DataBind-derived owner");
                    (binding.target(), binding.base.property_key())
                })
                .expect("Artboard retains native DataBind occurrences");
            let Some(target) = target else { continue };
            let index = self.get_object_id(artboard, &target);
            if index >= 0 {
                self.add_property_key(index as u32, key);
            }
        }
    }

    fn add_property_key(&mut self, object_id: u32, key: u32) {
        let object_data = self.get_core_object_data(object_id);
        match CoreRegistry::property_field_id(key as i32) {
            CoreDoubleType::ID
            | CoreColorType::ID
            | CoreUintType::ID
            | CoreStringType::ID
            | CoreBoolType::ID => object_data.add_property_key(key as u16),
            _ => {}
        }
    }

    fn record_state_machine(&mut self, machine: Option<&CoreHandle>) {
        let Some(machine) = machine else { return };
        let layers = machine
            .with_downcast::<StateMachine, _>(|machine| {
                (0..machine.layer_count())
                    .map(|index| machine.layer(index).expect("state-machine layer index"))
                    .collect::<Vec<_>>()
            })
            .expect("PropertyRecorder records a StateMachine occurrence");
        for layer in layers {
            self.record_state_machine_layer(&layer);
        }
    }

    pub fn record_state_machine_inputs(&mut self, machine: Option<&CoreHandle>) {
        if let Some(machine) = machine {
            let inputs = machine
                .with_downcast::<StateMachine, _>(|machine| {
                    (0..machine.input_count())
                        .map(|index| machine.input(index))
                        .collect::<Vec<_>>()
                })
                .expect("PropertyRecorder records a StateMachine occurrence");
            for input in inputs.into_iter().flatten() {
                self.record_state_machine_input(&input);
            }
        }
        self.complete_state_machine();
    }

    fn record_state_machine_input(&mut self, input: &CoreHandle) {
        match input.core_type() {
            Some(StateMachineNumberBase::TYPE_KEY) => {
                self.write_property_int_sm(0);
                let value = input
                    .with_downcast::<StateMachineNumber, _>(|input| input.base.value())
                    .expect("StateMachineNumber occurrence");
                self.write_property_float_sm(value);
            }
            Some(StateMachineBoolBase::TYPE_KEY) => {
                self.write_property_int_sm(1);
                let value = input
                    .with_downcast::<StateMachineBool, _>(|input| input.base.value())
                    .expect("StateMachineBool occurrence");
                self.write_property_bool_sm(value);
            }
            Some(StateMachineTriggerBase::TYPE_KEY) => self.write_property_int_sm(2),
            _ => {}
        }
    }

    fn record_state_machine_layer(&mut self, layer: &CoreHandle) {
        let states = layer
            .with_downcast::<StateMachineLayer, _>(|layer| layer.states().to_vec())
            .expect("PropertyRecorder records a StateMachineLayer occurrence");
        for state in states {
            self.record_state_machine_layer_state(&state);
        }
    }

    fn record_state_machine_layer_state(&mut self, state: &CoreHandle) {
        if let Some(animation) = state.with_downcast::<AnimationState, _>(AnimationState::animation)
        {
            self.record_linear_animation(animation.as_ref());
        } else if let Some(animations) =
            state.with(|state| state.blend_state_animations()).flatten()
        {
            for blend in animations {
                let animation = blend
                    .with(|blend| blend.blend_animation_animation())
                    .expect("BlendState retains its BlendAnimation occurrences");
                self.record_linear_animation(animation.as_ref());
            }
        }
    }

    fn record_linear_animation(&mut self, animation: Option<&CoreHandle>) {
        let Some(animation) = animation else { return };
        let objects = animation
            .with_downcast::<LinearAnimation, _>(|animation| animation.keyed_objects().to_vec())
            .expect("PropertyRecorder records a LinearAnimation occurrence");
        for object in objects {
            self.record_keyed_object(&object);
        }
    }

    fn record_keyed_object(&mut self, object: &CoreHandle) {
        let (id, properties) = object
            .with_downcast::<KeyedObject, _>(|object| {
                (object.base.object_id(), object.keyed_properties().to_vec())
            })
            .expect("LinearAnimation retains native KeyedObject occurrences");
        self.get_core_object_data(id);
        for property in properties {
            let key = property
                .with_downcast::<KeyedProperty, _>(|property| property.base.property_key())
                .expect("KeyedObject retains native KeyedProperty occurrences");
            self.add_property_key(id, key);
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
    fn write_properties(&mut self, artboard: &CoreHandle) {
        for index in 0..self.core_objects_data.len() {
            let id = self.core_objects_data[index].object_id;
            let keys = self.core_objects_data[index].property_keys.clone();
            if keys.is_empty() {
                continue;
            }
            let object = artboard
                .with_downcast::<Artboard, _>(|artboard| artboard.resolve_handle(id))
                .flatten()
                .expect("recorded property target resolves in its source Artboard");
            self.write_object_id(id);
            self.write_total_properties(keys.len() as u32);
            for key in keys {
                match CoreRegistry::property_field_id(key as i32) {
                    CoreDoubleType::ID => {
                        self.write_property_key(key as u32);
                        let value = object
                            .with_mut(|object| CoreRegistry::get_double(object, key as i32))
                            .expect("recorded target remains live");
                        self.write_property_float(value);
                    }
                    CoreColorType::ID => {
                        self.write_property_key(key as u32);
                        let value = object
                            .with_mut(|object| CoreRegistry::get_color(object, key as i32))
                            .expect("recorded target remains live");
                        self.write_property_uint(value as u32);
                    }
                    CoreUintType::ID => {
                        self.write_property_key(key as u32);
                        let value = object
                            .with_mut(|object| CoreRegistry::get_uint(object, key as i32))
                            .expect("recorded target remains live");
                        self.write_property_uint(value);
                    }
                    CoreStringType::ID => {
                        self.write_property_key(key as u32);
                        let value = object
                            .with_mut(|object| CoreRegistry::get_string(object, key as i32))
                            .expect("recorded target remains live");
                        self.write_property_string(value);
                    }
                    CoreBoolType::ID => {
                        self.write_property_key(key as u32);
                        let value = object
                            .with_mut(|object| CoreRegistry::get_bool(object, key as i32))
                            .expect("recorded target remains live");
                        self.write_property_bool(value);
                    }
                    _ => {}
                }
            }
        }
    }

    fn get_object_id(&self, artboard: &CoreHandle, object: &CoreHandle) -> i32 {
        artboard
            .with_downcast::<Artboard, _>(|artboard| artboard.object_index(object))
            .expect("PropertyRecorder records an Artboard occurrence")
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
    pub fn apply_state_machine(&mut self, instance: &RuntimeStateMachineInstanceHandle) {
        let mut index = 0;
        self.reader_position_sm = 0;
        while self.reader_position_sm < self.reader_end_sm && index < 20 {
            let kind = Self::read_var_uint(
                &self.write_buffer_sm,
                &mut self.reader_position_sm,
                self.reader_end_sm,
            );
            if kind == 0 {
                let value = Self::read_float(
                    &self.write_buffer_sm,
                    &mut self.reader_position_sm,
                    self.reader_end_sm,
                );
                let name = instance.with_instance(|instance| {
                    instance
                        .input(index)
                        .expect("recorded state-machine input index")
                        .name()
                        .to_owned()
                });
                instance.set_number(&name, value);
            } else if kind == 1 {
                let value = Self::read_byte(
                    &self.write_buffer_sm,
                    &mut self.reader_position_sm,
                    self.reader_end_sm,
                );
                let name = instance.with_instance(|instance| {
                    instance
                        .input(index)
                        .expect("recorded state-machine input index")
                        .name()
                        .to_owned()
                });
                instance.set_bool(&name, value != 0);
            }
            index += 1;
        }
    }

    pub fn apply_artboard(&mut self, artboard: &RuntimeArtboardInstanceHandle) {
        self.reader_position = 0;
        while self.reader_position < self.reader_end {
            let id = Self::read_var_uint(
                &self.write_buffer,
                &mut self.reader_position,
                self.reader_end,
            );
            let object = artboard
                .with_artboard(|artboard| artboard.base.resolve_handle(id))
                .expect("recorded property target resolves in its Artboard instance");
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
                );
                match CoreRegistry::property_field_id(key as i32) {
                    CoreDoubleType::ID => {
                        let value = Self::read_float(
                            &self.write_buffer,
                            &mut self.reader_position,
                            self.reader_end,
                        );
                        CoreRegistry::set_double_handle(&object, key as i32, value);
                    }
                    CoreColorType::ID => {
                        let value = Self::read_var_uint(
                            &self.write_buffer,
                            &mut self.reader_position,
                            self.reader_end,
                        );
                        CoreRegistry::set_color_handle(&object, key as i32, value as i32);
                    }
                    CoreUintType::ID => {
                        let value = Self::read_var_uint(
                            &self.write_buffer,
                            &mut self.reader_position,
                            self.reader_end,
                        );
                        CoreRegistry::set_uint_handle(&object, key as i32, value);
                    }
                    CoreStringType::ID => {
                        let value = Self::read_string(
                            &self.write_buffer,
                            &mut self.reader_position,
                            self.reader_end,
                        );
                        CoreRegistry::set_string_handle(&object, key as i32, value);
                    }
                    CoreBoolType::ID => {
                        let value = Self::read_byte(
                            &self.write_buffer,
                            &mut self.reader_position,
                            self.reader_end,
                        );
                        CoreRegistry::set_bool_handle(&object, key as i32, value != 0);
                    }
                    _ => {}
                }
            }
        }
    }
}
