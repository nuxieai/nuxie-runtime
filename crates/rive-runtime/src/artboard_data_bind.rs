use crate::{
    ArtboardInstance, RuntimeArtboardCustomPropertyBindingInstance,
    RuntimeArtboardDataBindValueKind, RuntimeArtboardSoloBindingInstance,
    RuntimeDataBindGraphValue, runtime_data_bind_graph_convert_value,
    runtime_default_view_model_value_for_path,
};
use rive_binary::RuntimeFile;

impl ArtboardInstance {
    pub fn bind_default_view_model_artboard_list_context(&mut self, file: &RuntimeFile) -> bool {
        let Some(default_instance) = file.view_model_default_instance(0) else {
            return false;
        };

        let mut changed = false;
        let paths = self
            .artboard_custom_property_bindings
            .iter()
            .map(|binding| binding.path.clone())
            .chain(
                self.artboard_solo_bindings
                    .iter()
                    .map(|binding| binding.path.clone()),
            )
            .collect::<Vec<_>>();
        for path in paths {
            let Some(value) =
                runtime_default_view_model_value_for_path(file, default_instance.object, &path)
            else {
                continue;
            };
            if self.artboard_data_bind_values.get(&path) != Some(&value) {
                self.artboard_data_bind_values.insert(path, value);
                changed = true;
            }
        }
        for binding in &mut self.artboard_list_bindings {
            let Some(source_value) = binding.default_value.resolve_from_view_model_instance(
                file,
                default_instance.object,
                &binding.path,
            ) else {
                continue;
            };
            let target_value = match binding.converter.as_ref() {
                Some(converter) => runtime_data_bind_graph_convert_value(converter, &source_value),
                None => Some(source_value.clone()),
            };
            binding.source_list_size = match &source_value {
                RuntimeDataBindGraphValue::List { item_count } => Some(*item_count),
                _ => None,
            };
            binding.source_number_value = match source_value {
                RuntimeDataBindGraphValue::Number(value) => Some(value),
                _ => None,
            };
            binding.should_reset_instances = binding.source_number_value.is_some();
            let target_list_size = match target_value {
                Some(RuntimeDataBindGraphValue::List { .. }) => Some(0),
                _ => None,
            };
            if binding.target_list_size != target_list_size {
                changed = true;
                binding.target_list_size = target_list_size;
            }
        }
        changed
    }

    pub fn advance_artboard_data_binds(&mut self) -> bool {
        let mut changed = false;
        for binding in self.artboard_custom_property_bindings.clone() {
            changed |= self.update_artboard_custom_property_binding(&binding);
        }
        for binding in &mut self.artboard_list_bindings {
            let target_value = match binding.converter.as_ref() {
                Some(converter) => {
                    runtime_data_bind_graph_convert_value(converter, &binding.default_value)
                }
                None => Some(binding.default_value.clone()),
            };
            let target_list_size = match target_value {
                Some(RuntimeDataBindGraphValue::List { item_count }) => Some(item_count),
                _ => None,
            };
            if binding.target_list_size != target_list_size {
                binding.target_list_size = target_list_size;
                changed = true;
            }
        }
        changed |= self.apply_artboard_solo_bindings();
        changed
    }

    fn update_artboard_custom_property_binding(
        &mut self,
        binding: &RuntimeArtboardCustomPropertyBindingInstance,
    ) -> bool {
        let value = match binding.value_kind {
            RuntimeArtboardDataBindValueKind::Number => self
                .double_property(binding.target_local_id, binding.property_key)
                .map(RuntimeDataBindGraphValue::Number),
            RuntimeArtboardDataBindValueKind::String => self
                .string_property(binding.target_local_id, binding.property_key)
                .map(|value| RuntimeDataBindGraphValue::String(value.to_vec())),
        };
        let Some(value) = value else {
            return false;
        };
        if self.artboard_data_bind_values.get(&binding.path) == Some(&value) {
            return false;
        }
        self.artboard_data_bind_values
            .insert(binding.path.clone(), value);
        true
    }

    fn apply_artboard_solo_bindings(&mut self) -> bool {
        let mut changed = false;
        for binding in self.artboard_solo_bindings.clone() {
            let Some(value) = self.artboard_data_bind_values.get(&binding.path).cloned() else {
                continue;
            };
            changed |= self.apply_artboard_solo_binding_value(&binding, &value);
        }
        changed
    }

    fn apply_artboard_solo_binding_value(
        &mut self,
        binding: &RuntimeArtboardSoloBindingInstance,
        value: &RuntimeDataBindGraphValue,
    ) -> bool {
        match value {
            RuntimeDataBindGraphValue::Number(value) => {
                self.set_solo_active_child_by_index(binding.target_local_id, *value)
            }
            RuntimeDataBindGraphValue::String(value) => {
                self.set_solo_active_child_by_name(binding.target_local_id, value)
            }
            _ => false,
        }
    }

    pub fn artboard_list_binding_source_list_size_for_data_bind(
        &self,
        data_bind_index: usize,
    ) -> Option<usize> {
        self.artboard_list_bindings
            .iter()
            .find(|binding| binding.data_bind_index == data_bind_index)
            .and_then(|binding| binding.source_list_size)
    }

    pub fn artboard_list_binding_source_number_value_for_data_bind(
        &self,
        data_bind_index: usize,
    ) -> Option<f32> {
        self.artboard_list_bindings
            .iter()
            .find(|binding| binding.data_bind_index == data_bind_index)
            .and_then(|binding| binding.source_number_value)
    }

    pub fn artboard_list_binding_target_list_size_for_data_bind(
        &self,
        data_bind_index: usize,
    ) -> Option<usize> {
        self.artboard_list_bindings
            .iter()
            .find(|binding| binding.data_bind_index == data_bind_index)
            .and_then(|binding| binding.target_list_size)
    }

    pub fn artboard_list_binding_target_local_for_data_bind(
        &self,
        data_bind_index: usize,
    ) -> Option<usize> {
        self.artboard_list_bindings
            .iter()
            .find(|binding| binding.data_bind_index == data_bind_index)
            .map(|binding| binding.target_local_id)
    }

    pub fn artboard_list_binding_should_reset_instances_for_data_bind(
        &self,
        data_bind_index: usize,
    ) -> Option<bool> {
        self.artboard_list_bindings
            .iter()
            .find(|binding| binding.data_bind_index == data_bind_index)
            .map(|binding| binding.should_reset_instances)
    }
}
