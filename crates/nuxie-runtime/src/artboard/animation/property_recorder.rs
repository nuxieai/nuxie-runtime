use super::super::ArtboardInstance;
use crate::animation::{
    RuntimeBlendAnimation, RuntimeKeyedCallback, RuntimeKeyedObject, RuntimeLinearAnimation,
    RuntimeLinearAnimationHandle,
};
use crate::constraints::set_runtime_scroll_double_property;
use crate::properties::{artboard_index_for_graph, property_key_for_name};
use crate::state_machine::{
    RuntimeLayerState, RuntimeStateMachine, RuntimeStateMachineLayer, StateMachineInputKind,
    StateMachineInstance,
};
use nuxie_binary::{BinaryDataReader, BinaryStream, BinaryWriter};
use nuxie_schema::{FieldKind, core_registry_getter_field_kind_by_property_key};

/// Direct Rust owner for pinned C++ `CoreObjectData`.
#[derive(Debug)]
pub(crate) struct CoreObjectData {
    property_keys: Vec<u16>,
    pub(crate) object_id: u32,
}

impl CoreObjectData {
    pub(crate) fn new(object_id: u32) -> Self {
        Self {
            property_keys: Vec::new(),
            object_id,
        }
    }

    pub(crate) fn add_property_key(&mut self, key: u16) {
        if !self.property_keys.contains(&key) {
            self.property_keys.push(key);
        }
    }

    pub(crate) fn property_keys(&mut self) -> &mut Vec<u16> {
        &mut self.property_keys
    }
}

/// `VectorBinaryWriter` keeps its vector allocation and only rewinds its
/// logical cursor in `clear()`. Keeping the cursor separate from the bytes is
/// the safe-Rust representation of that observable upstream quirk.
#[derive(Debug, Default)]
struct RecorderBuffer {
    bytes: Vec<u8>,
    writer_position: usize,
    reader_length: usize,
}

struct RecorderWriteStream<'a> {
    buffer: &'a mut RecorderBuffer,
}

impl BinaryStream for RecorderWriteStream<'_> {
    fn write(&mut self, bytes: &[u8]) {
        let start = self.buffer.writer_position;
        let Some(end) = start.checked_add(bytes.len()) else {
            return;
        };
        if self.buffer.bytes.len() < end {
            self.buffer.bytes.resize(end, 0);
        }
        if let Some(destination) = self.buffer.bytes.get_mut(start..end) {
            destination.copy_from_slice(bytes);
            self.buffer.writer_position = end;
        }
    }

    fn flush(&mut self) {}

    fn clear(&mut self) {
        self.buffer.writer_position = 0;
    }
}

impl RecorderBuffer {
    fn write(&mut self, write: impl FnOnce(&mut BinaryWriter<'_>)) {
        let mut stream = RecorderWriteStream { buffer: self };
        let mut writer = BinaryWriter::new(&mut stream);
        write(&mut writer);
    }

    fn clear(&mut self) {
        self.writer_position = 0;
    }

    fn complete(&mut self) {
        self.reader_length = self.writer_position;
    }

    fn reader(&self) -> BinaryDataReader<'_> {
        BinaryDataReader::new(
            self.bytes
                .get(..self.reader_length)
                .unwrap_or(self.bytes.as_slice()),
        )
    }
}

/// Mechanical translation of pinned `PropertyRecorder`.
///
/// Rust cannot retain readers and writers that borrow buffers in the same
/// struct. `RecorderBuffer` owns the corresponding writer cursor and reader
/// length; short-lived `BinaryWriter`/`BinaryDataReader` values perform the
/// exact wire operations against that retained state.
#[derive(Debug, Default)]
pub(crate) struct PropertyRecorder {
    binary: RecorderBuffer,
    binary_sm: RecorderBuffer,
    core_objects_data: Vec<CoreObjectData>,
}

impl PropertyRecorder {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    fn write_object_id(object_id: u32, writer: &mut RecorderBuffer) {
        writer.write(|writer| writer.write_var_uint32(object_id));
    }

    fn write_total_properties(value: u32, writer: &mut RecorderBuffer) {
        writer.write(|writer| writer.write_var_uint32(value));
    }

    fn write_property_key(value: u32, writer: &mut RecorderBuffer) {
        writer.write(|writer| writer.write_var_uint32(value));
    }

    fn write_property_value_float(value: f32, writer: &mut RecorderBuffer) {
        writer.write(|writer| writer.write_float(value));
    }

    fn write_property_value_int(value: i32, writer: &mut RecorderBuffer) {
        writer.write(|writer| writer.write_var_uint32(value as u32));
    }

    fn write_property_value_uint(value: u32, writer: &mut RecorderBuffer) {
        writer.write(|writer| writer.write_var_uint32(value));
    }

    fn write_property_value_string(value: &[u8], writer: &mut RecorderBuffer) {
        writer.write(|writer| writer.write_string(value));
    }

    fn write_property_value_bool(value: bool, writer: &mut RecorderBuffer) {
        writer.write(|writer| writer.write_u8(u8::from(value)));
    }

    pub(crate) fn clear(&mut self) {
        // Intentionally does not rewind the state-machine writer or clear the
        // retained object/key catalog, matching the upstream method's scope.
        self.binary.clear();
    }

    fn complete(writer: &mut RecorderBuffer) {
        writer.complete();
    }

    pub(crate) fn record_artboard(&mut self, artboard: &ArtboardInstance) {
        let state_machine = artboard.state_machine(0);
        self.record_state_machine_inputs(state_machine);
        self.record_state_machine(
            state_machine,
            artboard.linear_animations(),
            &artboard.empty_linear_animation,
        );
        self.record_data_binds(artboard);
        self.write_properties(artboard);
        Self::complete(&mut self.binary);
    }

    fn record_data_binds(&mut self, artboard: &ArtboardInstance) {
        let Some(file) = artboard.runtime_file() else {
            return;
        };
        let Some(graph) = artboard.runtime_graph() else {
            return;
        };
        let Some(artboard_index) = artboard_index_for_graph(file, graph) else {
            return;
        };
        for data_bind in file.artboard_data_binds(artboard_index) {
            let Some(target_local_id) = data_bind.target_local_id else {
                continue;
            };
            let index = Self::get_object_id(artboard, target_local_id);
            let property_key = data_bind.object.uint_property("propertyKey").unwrap_or(0);
            if index >= 0 {
                let core_object_data = self.get_core_object_data(index as u32);
                if let Ok(property_key) = u16::try_from(property_key) {
                    Self::add_property_key(core_object_data, property_key);
                }
            }
        }
    }

    fn add_property_key(core_object_data: &mut CoreObjectData, property_key: u16) {
        match core_registry_getter_field_kind_by_property_key(property_key) {
            Some(
                FieldKind::Double
                | FieldKind::Color
                | FieldKind::Uint
                | FieldKind::String
                | FieldKind::Bool,
            ) => core_object_data.add_property_key(property_key),
            _ => {}
        }
    }

    fn record_state_machine(
        &mut self,
        state_machine: Option<&RuntimeStateMachine>,
        animations: &[RuntimeLinearAnimation],
        empty_animation: &RuntimeLinearAnimation,
    ) {
        let Some(state_machine) = state_machine else {
            return;
        };
        for state_machine_layer in state_machine.layers.iter() {
            self.record_state_machine_layer(state_machine_layer, animations, empty_animation);
        }
    }

    pub(crate) fn record_state_machine_inputs(
        &mut self,
        state_machine: Option<&RuntimeStateMachine>,
    ) {
        if let Some(state_machine) = state_machine {
            for state_machine_input in state_machine.inputs.iter() {
                self.record_state_machine_input(state_machine_input.as_ref());
            }
        }
        Self::complete(&mut self.binary_sm);
    }

    fn record_state_machine_input(
        &mut self,
        state_machine_input: Option<&crate::state_machine::RuntimeStateMachineInput>,
    ) {
        let Some(state_machine_input) = state_machine_input else {
            return;
        };
        match state_machine_input.kind {
            StateMachineInputKind::Number => {
                Self::write_property_value_int(0, &mut self.binary_sm);
                Self::write_property_value_float(
                    state_machine_input.number_value().unwrap_or(0.0),
                    &mut self.binary_sm,
                );
            }
            StateMachineInputKind::Bool => {
                Self::write_property_value_int(1, &mut self.binary_sm);
                Self::write_property_value_bool(
                    state_machine_input.bool_value().unwrap_or(false),
                    &mut self.binary_sm,
                );
            }
            StateMachineInputKind::Trigger => {
                Self::write_property_value_int(2, &mut self.binary_sm);
            }
        }
    }

    fn record_state_machine_layer(
        &mut self,
        state_machine_layer: &RuntimeStateMachineLayer,
        animations: &[RuntimeLinearAnimation],
        empty_animation: &RuntimeLinearAnimation,
    ) {
        for layer_state in &state_machine_layer.states {
            self.record_state_machine_layer_state(layer_state, animations, empty_animation);
        }
    }

    fn record_state_machine_layer_state(
        &mut self,
        layer_state: &RuntimeLayerState,
        animations: &[RuntimeLinearAnimation],
        empty_animation: &RuntimeLinearAnimation,
    ) {
        if layer_state.type_name == Some("AnimationState") {
            self.record_animation_handle(layer_state.animation(), animations, empty_animation);
        } else if let Some(blend_state) = layer_state.blend_state_1d.as_ref() {
            for blend_animation in &blend_state.animations {
                self.record_animation_handle(
                    Some(blend_animation.animation()),
                    animations,
                    empty_animation,
                );
            }
        } else if let Some(blend_state) = layer_state.blend_state_direct.as_ref() {
            for blend_animation in &blend_state.animations {
                self.record_animation_handle(
                    Some(blend_animation.animation()),
                    animations,
                    empty_animation,
                );
            }
        }
    }

    fn record_animation_handle(
        &mut self,
        animation: Option<RuntimeLinearAnimationHandle>,
        animations: &[RuntimeLinearAnimation],
        empty_animation: &RuntimeLinearAnimation,
    ) {
        self.record_linear_animation(
            animation.and_then(|animation| animation.resolve(animations, empty_animation)),
        );
    }

    fn record_linear_animation(&mut self, linear_animation: Option<&RuntimeLinearAnimation>) {
        let Some(linear_animation) = linear_animation else {
            return;
        };
        for keyed_object in linear_animation.keyed_objects.iter() {
            self.record_keyed_object(Some(keyed_object));
        }
    }

    fn record_keyed_object(&mut self, keyed_object: Option<&RuntimeKeyedObject>) {
        let Some(keyed_object) = keyed_object else {
            return;
        };
        let Ok(object_id) = u32::try_from(keyed_object.object_id) else {
            return;
        };
        let core_object_data = self.get_core_object_data(object_id);
        for keyed_property in &keyed_object.keyed_properties {
            Self::add_property_key(core_object_data, keyed_property.property_key);
        }
    }

    fn get_core_object_data(&mut self, id: u32) -> &mut CoreObjectData {
        if let Some(index) = self
            .core_objects_data
            .iter()
            .position(|core_object_data| core_object_data.object_id == id)
        {
            return &mut self.core_objects_data[index];
        }
        self.core_objects_data.push(CoreObjectData::new(id));
        let index = self.core_objects_data.len() - 1;
        &mut self.core_objects_data[index]
    }

    fn write_properties(&mut self, artboard: &ArtboardInstance) {
        for core_object_data in &mut self.core_objects_data {
            let property_keys = core_object_data.property_keys().clone();
            if property_keys.is_empty() {
                continue;
            }
            let Ok(local_id) = usize::try_from(core_object_data.object_id) else {
                continue;
            };
            Self::write_object_id(core_object_data.object_id, &mut self.binary);
            Self::write_total_properties(property_keys.len() as u32, &mut self.binary);
            for property_key in property_keys {
                match core_registry_getter_field_kind_by_property_key(property_key) {
                    Some(FieldKind::Double) => {
                        Self::write_property_key(u32::from(property_key), &mut self.binary);
                        Self::write_property_value_float(
                            artboard
                                .double_property(local_id, property_key)
                                .unwrap_or(0.0),
                            &mut self.binary,
                        );
                    }
                    Some(FieldKind::Color) => {
                        Self::write_property_key(u32::from(property_key), &mut self.binary);
                        Self::write_property_value_uint(
                            artboard.color_property(local_id, property_key).unwrap_or(0),
                            &mut self.binary,
                        );
                    }
                    Some(FieldKind::Uint) => {
                        Self::write_property_key(u32::from(property_key), &mut self.binary);
                        Self::write_property_value_uint(
                            artboard
                                .uint_property(local_id, property_key)
                                .and_then(|value| u32::try_from(value).ok())
                                .unwrap_or(0),
                            &mut self.binary,
                        );
                    }
                    Some(FieldKind::String) => {
                        Self::write_property_key(u32::from(property_key), &mut self.binary);
                        Self::write_property_value_string(
                            artboard
                                .string_property(local_id, property_key)
                                .unwrap_or_default(),
                            &mut self.binary,
                        );
                    }
                    Some(FieldKind::Bool) => {
                        Self::write_property_key(u32::from(property_key), &mut self.binary);
                        Self::write_property_value_bool(
                            artboard
                                .bool_property(local_id, property_key)
                                .unwrap_or(false),
                            &mut self.binary,
                        );
                    }
                    _ => {}
                }
            }
        }
    }

    fn get_object_id(artboard: &ArtboardInstance, target_local_id: usize) -> i32 {
        if artboard.slot(target_local_id).is_none() {
            return -1;
        }
        i32::try_from(target_local_id).unwrap_or(-1)
    }

    pub(crate) fn apply_state_machine(&self, state_machine_instance: &mut StateMachineInstance) {
        let mut index = 0usize;
        let mut reader = self.binary_sm.reader();
        while !reader.is_eof() && index < 20 {
            let input_type = reader.read_var_uint32();
            let Some(name) = state_machine_instance
                .input(index)
                .map(|input| input.name().unwrap_or_default().to_owned())
            else {
                break;
            };
            if input_type == 0 {
                let value = reader.read_float32();
                if let Some(target_index) = (0..state_machine_instance.input_count()).find(|&i| {
                    state_machine_instance.input(i).is_some_and(|input| {
                        input.kind() == StateMachineInputKind::Number
                            && input.name() == Some(name.as_str())
                    })
                }) {
                    state_machine_instance.set_number(target_index, value);
                }
            } else if input_type == 1 {
                let value = reader.read_byte() != 0;
                if let Some(target_index) = (0..state_machine_instance.input_count()).find(|&i| {
                    state_machine_instance.input(i).is_some_and(|input| {
                        input.kind() == StateMachineInputKind::Bool
                            && input.name() == Some(name.as_str())
                    })
                }) {
                    state_machine_instance.set_bool(target_index, value);
                }
            }
            index += 1;
        }
    }

    pub(crate) fn apply_artboard(&self, artboard: &mut ArtboardInstance) {
        let mut reader = self.binary.reader();
        while !reader.is_eof() {
            let object_id = reader.read_var_uint32();
            let Ok(local_id) = usize::try_from(object_id) else {
                break;
            };
            let total_properties = reader.read_var_uint32();
            let mut current_property_index = 0;
            while current_property_index < total_properties {
                let property_key = reader.read_var_uint32();
                let Ok(property_key) = u16::try_from(property_key) else {
                    current_property_index += 1;
                    continue;
                };
                match core_registry_getter_field_kind_by_property_key(property_key) {
                    Some(FieldKind::Double) => {
                        let property_value = reader.read_float32();
                        artboard.set_double_property(local_id, property_key, property_value);
                    }
                    Some(FieldKind::Color) => {
                        let property_value = reader.read_var_uint32();
                        artboard.set_color_property(local_id, property_key, property_value);
                    }
                    Some(FieldKind::Uint) => {
                        let property_value = reader.read_var_uint32();
                        artboard.set_uint_property(
                            local_id,
                            property_key,
                            u64::from(property_value),
                        );
                    }
                    Some(FieldKind::String) => {
                        let property_value = reader.read_string();
                        artboard.set_string_property(local_id, property_key, property_value);
                    }
                    Some(FieldKind::Bool) => {
                        let property_value = reader.read_byte() != 0;
                        artboard.set_bool_property(local_id, property_key, property_value);
                    }
                    _ => {}
                }
                current_property_index += 1;
            }
        }
    }
}

impl ArtboardInstance {
    pub(crate) fn set_keyed_color_property(
        &mut self,
        local_id: usize,
        property_key: u16,
        value: u32,
    ) -> bool {
        let previous = self.color_property(local_id, property_key);
        if !self
            .objects
            .set_generated_color_property(local_id, property_key, value)
        {
            return false;
        }
        self.after_color_property_set(local_id, property_key, previous, value)
    }

    /// C++ keyed animations retain a concrete Core pointer, so a known
    /// `SolidColor::colorValue` write does not rediscover its type or property
    /// on every frame. Keep the same observer and invalidation effects as the
    /// generic color setter while skipping branches that cannot apply to a
    /// SolidColor target (text, view-model, gradient, and layout topology).
    pub(crate) fn set_keyed_solid_color_property(
        &mut self,
        local_id: usize,
        property_key: u16,
        data_bind_observed: bool,
        value: u32,
    ) -> bool {
        let Some(previous) = self.objects.replace_solid_color_value(local_id, value) else {
            return false;
        };
        // Generated C++ setters return before the property callback when the
        // stored value is unchanged (`solid_color_base.hpp:38-46`). Active
        // animations may apply the same keyed value every frame; do not
        // rebuild or reconfigure the retained ShapePaint owner in that case.
        if previous == value {
            return false;
        }
        if data_bind_observed {
            self.notify_artboard_data_bind_target_property_changed(local_id, property_key);
        }
        // `SolidColor::renderOpacityChanged()` mutates the retained
        // RenderPaint and calls only `Artboard::changed()`; it does not dirty
        // component/path preparation (`solid_color.cpp:23-54`).
        self.mark_render_paint_changed();
        // Pinned C++ `SolidColor::colorValueChanged` immediately calls
        // `renderOpacityChanged` and mutates the ShapePaint-owned paint
        // (`solid_color.cpp:23-54`). It does not dirty or reconstruct the
        // ShapePaint owner.
        self.settle_runtime_solid_color_callback(local_id, value);
        if let Some(revision) = self.solid_color_paint_revisions.get_mut(local_id) {
            *revision = revision.wrapping_add(1);
        }
        self.mark_prepared_changed_for_solid_color_visibility(Some(previous), value);
        true
    }

    pub(crate) fn set_keyed_double_property(
        &mut self,
        local_id: usize,
        property_key: u16,
        value: f32,
    ) -> bool {
        if self.slot(local_id).and_then(|slot| slot.type_name) == Some("NestedNumber")
            && property_key_for_name("NestedNumber", "nestedValue") == Some(property_key)
        {
            return self.set_nested_number_value(local_id, value);
        }
        if let Some(changed) =
            set_runtime_scroll_double_property(self, local_id, property_key, value)
        {
            if !changed {
                return false;
            }
            let _ = self
                .objects
                .set_generated_double_property(local_id, property_key, value);
            return self.after_double_property_set(local_id, property_key, value);
        }
        if self.runtime_images.has_public_scale(local_id, property_key)
            && self.double_property(local_id, property_key) == Some(value)
        {
            return false;
        }
        let object_changed =
            self.objects
                .set_generated_double_property(local_id, property_key, value);
        let image_scale_changed = self
            .runtime_images
            .mark_public_scale_written(local_id, property_key);
        if !object_changed && !image_scale_changed {
            return false;
        }
        self.after_double_property_set(local_id, property_key, value)
    }

    pub(crate) fn apply_keyed_callback(&mut self, callback: RuntimeKeyedCallback) -> bool {
        let _seconds_delay = callback.seconds_delay;
        // Pinned `CoreRegistry::setCallback` dispatches these three concrete
        // trigger callbacks before its `EventBase::trigger` case. The caller
        // performs that event report immediately after this property phase.
        match self
            .slot(callback.target_local_id)
            .and_then(|slot| slot.type_name)
        {
            Some("ViewModelInstanceTrigger")
                if property_key_for_name("ViewModelInstanceTrigger", "fire")
                    == Some(callback.property_key) =>
            {
                let Some(property_value_key) =
                    property_key_for_name("ViewModelInstanceTrigger", "propertyValue")
                else {
                    return false;
                };
                let value = self
                    .uint_property(callback.target_local_id, property_value_key)
                    .unwrap_or(0)
                    .wrapping_add(1);
                self.set_uint_property(callback.target_local_id, property_value_key, value)
            }
            Some("CustomPropertyTrigger")
                if property_key_for_name("CustomPropertyTrigger", "fire")
                    == Some(callback.property_key) =>
            {
                let Some(property_value_key) =
                    property_key_for_name("CustomPropertyTrigger", "propertyValue")
                else {
                    return false;
                };
                let value = self
                    .uint_property(callback.target_local_id, property_value_key)
                    .unwrap_or(0)
                    .wrapping_add(1);
                self.set_uint_property(callback.target_local_id, property_value_key, value)
            }
            Some("NestedTrigger")
                if property_key_for_name("NestedTrigger", "fire")
                    == Some(callback.property_key) =>
            {
                self.fire_nested_trigger_input(callback.target_local_id)
            }
            _ => false,
        }
    }
}
