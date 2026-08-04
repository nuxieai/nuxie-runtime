// State-machine instance integration for the C++ `data_bind.cpp` source.
use super::state_machine_instance::RuntimeViewModelListenerSource;
use super::*;
impl StateMachineInstance {
    pub(super) fn listener_asset_value_for_data_bind<'a>(
        &'a self,
        data_bind_index: usize,
        fallback: &'a RuntimeBindableAssetValue,
    ) -> &'a RuntimeBindableAssetValue {
        self.bindable_assets
            .iter()
            .find(|bindable_asset| bindable_asset.has_data_bind_index(data_bind_index))
            .map(|bindable_asset| &bindable_asset.value)
            .unwrap_or(fallback)
    }
    pub(super) fn set_owned_view_model_context_font_asset_source_for_data_bind(
        &mut self,
        context: &mut RuntimeOwnedViewModelInstance,
        data_bind_index: usize,
        value: &RuntimeFontAssetValue,
    ) -> bool {
        if !self
            .data_bind_graph
            .set_owned_view_model_context_font_asset_source_for_data_bind(
                context,
                data_bind_index,
                value,
            )
        {
            return false;
        }
        // A listener can feed the updated source into another bindable in the
        // same frame. Refresh the full Font payload now; the scalar graph only
        // carries the generated propertyValue index.
        self.sync_bindable_font_assets_from_owned_context(context);
        true
    }
    pub(super) fn set_owned_view_model_context_blob_asset_source_for_data_bind(
        &mut self,
        context: &mut RuntimeOwnedViewModelInstance,
        data_bind_index: usize,
        value: &RuntimeBlobAssetValue,
    ) -> bool {
        self.data_bind_graph
            .set_owned_view_model_context_blob_asset_source_for_data_bind(
                context,
                data_bind_index,
                value,
            )
    }
    pub fn set_bindable_number_for_data_bind(
        &mut self,
        data_bind_index: usize,
        value: f32,
    ) -> bool {
        let Some(bindable_number) = self
            .bindable_numbers
            .iter_mut()
            .find(|bindable_number| bindable_number.has_data_bind_index(data_bind_index))
        else {
            return false;
        };
        if !bindable_number.set_value(value) {
            return false;
        }
        self.data_bind_graph
            .mark_number_target_dirty_for_data_bind(data_bind_index);
        self.apply_direct_bindable_target_change(data_bind_index);
        self.needs_advance = true;
        true
    }
    pub fn set_bindable_boolean_for_data_bind(
        &mut self,
        data_bind_index: usize,
        value: bool,
    ) -> bool {
        let Some(bindable_boolean) = self
            .bindable_booleans
            .iter_mut()
            .find(|bindable_boolean| bindable_boolean.has_data_bind_index(data_bind_index))
        else {
            return false;
        };
        if !bindable_boolean.set_value(value) {
            return false;
        }
        self.data_bind_graph
            .mark_boolean_target_dirty_for_data_bind(data_bind_index);
        self.needs_advance = true;
        true
    }
    pub fn set_bindable_integer_for_data_bind(
        &mut self,
        data_bind_index: usize,
        value: u64,
    ) -> bool {
        let Some(bindable_integer) = self
            .bindable_integers
            .iter_mut()
            .find(|bindable_integer| bindable_integer.has_data_bind_index(data_bind_index))
        else {
            return false;
        };
        if !bindable_integer.set_value(value) {
            return false;
        }
        self.data_bind_graph
            .mark_integer_target_dirty_for_data_bind(data_bind_index);
        self.needs_advance = true;
        true
    }
    pub fn set_bindable_color_for_data_bind(&mut self, data_bind_index: usize, value: u32) -> bool {
        let Some(bindable_color) = self
            .bindable_colors
            .iter_mut()
            .find(|bindable_color| bindable_color.has_data_bind_index(data_bind_index))
        else {
            return false;
        };
        if !bindable_color.set_value(value) {
            return false;
        }
        self.data_bind_graph
            .mark_color_target_dirty_for_data_bind(data_bind_index);
        self.needs_advance = true;
        true
    }
    pub fn set_bindable_string_for_data_bind(
        &mut self,
        data_bind_index: usize,
        value: &[u8],
    ) -> bool {
        let Some(bindable_string) = self
            .bindable_strings
            .iter_mut()
            .find(|bindable_string| bindable_string.has_data_bind_index(data_bind_index))
        else {
            return false;
        };
        if !bindable_string.set_value(value) {
            return false;
        }
        self.data_bind_graph
            .mark_string_target_dirty_for_data_bind(data_bind_index);
        self.needs_advance = true;
        true
    }
    pub fn set_bindable_enum_for_data_bind(&mut self, data_bind_index: usize, value: u64) -> bool {
        let Some(bindable_enum) = self
            .bindable_enums
            .iter_mut()
            .find(|bindable_enum| bindable_enum.has_data_bind_index(data_bind_index))
        else {
            return false;
        };
        if !bindable_enum.set_value(value) {
            return false;
        }
        self.data_bind_graph
            .mark_enum_target_dirty_for_data_bind(data_bind_index);
        self.needs_advance = true;
        true
    }
    pub fn set_bindable_asset_for_data_bind(&mut self, data_bind_index: usize, value: u64) -> bool {
        let Some(bindable_asset) = self
            .bindable_assets
            .iter_mut()
            .find(|bindable_asset| bindable_asset.has_data_bind_index(data_bind_index))
        else {
            return false;
        };
        if !bindable_asset.set_value(value) {
            return false;
        }
        self.data_bind_graph
            .mark_asset_target_dirty_for_data_bind(data_bind_index);
        self.needs_advance = true;
        true
    }
    pub fn set_bindable_artboard_for_data_bind(
        &mut self,
        data_bind_index: usize,
        value: u64,
    ) -> bool {
        let Some(bindable_artboard) = self
            .bindable_artboards
            .iter_mut()
            .find(|bindable_artboard| bindable_artboard.has_data_bind_index(data_bind_index))
        else {
            return false;
        };
        if !bindable_artboard.set_value(value) {
            return false;
        }
        self.data_bind_graph
            .mark_artboard_target_dirty_for_data_bind(data_bind_index);
        self.needs_advance = true;
        true
    }
    pub fn set_bindable_list_for_data_bind(
        &mut self,
        data_bind_index: usize,
        value: usize,
    ) -> bool {
        let Some(bindable_list) = self
            .bindable_lists
            .iter_mut()
            .find(|bindable_list| bindable_list.has_data_bind_index(data_bind_index))
        else {
            return false;
        };
        if !bindable_list.set_value(value) {
            return false;
        }
        self.data_bind_graph
            .mark_list_target_dirty_for_data_bind(data_bind_index);
        self.needs_advance = true;
        true
    }
    pub fn set_bindable_view_model_for_data_bind(
        &mut self,
        data_bind_index: usize,
        instance_index: usize,
    ) -> bool {
        let Some(value) = self
            .data_bind_graph
            .imported_view_model_target_value_for_data_bind(data_bind_index, instance_index)
        else {
            return false;
        };
        let Some(bindable_view_model) = self
            .bindable_view_models
            .iter_mut()
            .find(|bindable_view_model| bindable_view_model.has_data_bind_index(data_bind_index))
        else {
            return false;
        };
        if !bindable_view_model.set_imported_value(value) {
            return false;
        }
        self.data_bind_graph
            .mark_view_model_target_dirty_for_data_bind(data_bind_index);
        self.apply_direct_bindable_target_change(data_bind_index);
        self.needs_advance = true;
        true
    }
    fn apply_direct_bindable_target_change(&mut self, data_bind_index: usize) {
        let Some(data_bind_index) = self
            .data_bind_graph
            .bindable_data_bind_to_source_index_for_data_bind(data_bind_index)
        else {
            // C++ stores only explicitly Direction=ToSource occurrences in
            // `m_bindableDataBindsToSource`. A main-to-target TwoWay bind is
            // capable of reverse flow during `updateDataBinds(true)`, but a
            // direct BindableProperty host edit does not find it through this
            // map and therefore performs no immediate source write
            // (`state_machine_instance.cpp:1788-1805,3201-3210`).
            return;
        };
        // The public number/ViewModel setters mirror the C++ host mutation
        // seam used by the runtime probe: mutate the cloned BindableProperty,
        // immediately call that occurrence's `updateSourceBinding(true)`,
        // then drain source-to-target dirt with `updateDataBinds(false)`.
        // `advancedDataContext()` itself only advances retained ViewModel
        // values and must not own either operation
        // (`state_machine_instance.cpp:2587-2593`;
        // `data_bind.cpp:550-588`).
        let targets = RuntimeDataBindGraphTargetsMut {
            numbers: &mut self.bindable_numbers,
            integers: &mut self.bindable_integers,
            booleans: &mut self.bindable_booleans,
            strings: &mut self.bindable_strings,
            colors: &mut self.bindable_colors,
            enums: &mut self.bindable_enums,
            assets: &mut self.bindable_assets,
            artboards: &mut self.bindable_artboards,
            lists: &mut self.bindable_lists,
            triggers: &mut self.bindable_triggers,
            view_models: &mut self.bindable_view_models,
            transition_durations: &mut self.transition_durations,
            include_view_models: true,
        };
        if let Err(error) = self
            .data_bind_graph
            .apply_direct_bindable_target_to_source_for_data_bind(data_bind_index, &targets)
        {
            self.script_error.get_or_insert(error);
            return;
        }
        if let Err(error) = self
            .data_bind_graph
            .update_all_default_view_model_bindings_false(targets)
        {
            self.script_error.get_or_insert(error);
        }
    }
    pub fn default_view_model_view_model_source_instance_index_for_data_bind(
        &self,
        data_bind_index: usize,
    ) -> Option<usize> {
        self.data_bind_graph
            .default_view_model_view_model_source_instance_index_for_data_bind(data_bind_index)
    }
    pub fn bindable_view_model_instance_index_for_data_bind(
        &self,
        data_bind_index: usize,
    ) -> Option<usize> {
        let global_id = self
            .data_bind_graph
            .view_model_target_global_id_for_data_bind(data_bind_index)?;
        let value = self
            .bindable_view_models
            .iter()
            .find(|bindable_view_model| bindable_view_model.global_id == global_id)
            .map(|bindable_view_model| bindable_view_model.value)?;
        self.data_bind_graph
            .view_model_instance_index_for_data_bind_value(data_bind_index, value)
    }
    pub fn default_view_model_number_source_value_for_data_bind(
        &self,
        data_bind_index: usize,
    ) -> Option<f32> {
        self.data_bind_graph
            .default_view_model_number_source_value_for_data_bind(data_bind_index)
    }
    pub fn bindable_number_value_for_data_bind(&self, data_bind_index: usize) -> Option<f32> {
        if let Some(value) = self
            .data_bind_graph
            .number_target_global_id_for_data_bind(data_bind_index)
            .and_then(|global_id| {
                self.bindable_numbers
                    .iter()
                    .find(|bindable_number| bindable_number.global_id == global_id)
                    .map(|bindable_number| bindable_number.value)
            })
        {
            return Some(value);
        }
        self.bindable_numbers
            .iter()
            .find(|bindable_number| bindable_number.has_data_bind_index(data_bind_index))
            .map(|bindable_number| bindable_number.value)
    }
    pub fn default_view_model_boolean_source_value_for_data_bind(
        &self,
        data_bind_index: usize,
    ) -> Option<bool> {
        self.data_bind_graph
            .default_view_model_boolean_source_value_for_data_bind(data_bind_index)
    }
    pub fn bindable_boolean_value_for_data_bind(&self, data_bind_index: usize) -> Option<bool> {
        if let Some(value) = self
            .data_bind_graph
            .boolean_target_global_id_for_data_bind(data_bind_index)
            .and_then(|global_id| bindable_boolean_value(&self.bindable_booleans, global_id))
        {
            return Some(value);
        }
        self.bindable_booleans
            .iter()
            .find(|bindable_boolean| bindable_boolean.has_data_bind_index(data_bind_index))
            .map(|bindable_boolean| bindable_boolean.value)
    }
    pub fn default_view_model_list_source_item_count_for_data_bind(
        &self,
        data_bind_index: usize,
    ) -> Option<usize> {
        self.data_bind_graph
            .default_view_model_list_source_item_count_for_data_bind(data_bind_index)
    }
    pub fn bindable_list_property_value_for_data_bind(
        &self,
        data_bind_index: usize,
    ) -> Option<usize> {
        let global_id = self
            .data_bind_graph
            .list_target_global_id_for_data_bind(data_bind_index)?;
        self.bindable_lists
            .iter()
            .find(|bindable_list| bindable_list.global_id == global_id)
            .map(|bindable_list| bindable_list.property_value)
    }
    pub fn default_view_model_string_source_value_for_data_bind(
        &self,
        data_bind_index: usize,
    ) -> Option<&[u8]> {
        self.data_bind_graph
            .default_view_model_string_source_value_for_data_bind(data_bind_index)
    }
    pub fn bindable_string_value_for_data_bind(&self, data_bind_index: usize) -> Option<&[u8]> {
        if let Some(value) = self
            .data_bind_graph
            .string_target_global_id_for_data_bind(data_bind_index)
            .and_then(|global_id| bindable_string_value(&self.bindable_strings, global_id))
        {
            return Some(value);
        }
        self.bindable_strings
            .iter()
            .find(|bindable_string| bindable_string.has_data_bind_index(data_bind_index))
            .map(|bindable_string| bindable_string.value.as_slice())
    }
    pub fn default_view_model_color_source_value_for_data_bind(
        &self,
        data_bind_index: usize,
    ) -> Option<u32> {
        self.data_bind_graph
            .default_view_model_color_source_value_for_data_bind(data_bind_index)
    }
    pub fn bindable_color_value_for_data_bind(&self, data_bind_index: usize) -> Option<u32> {
        if let Some(value) = self
            .data_bind_graph
            .color_target_global_id_for_data_bind(data_bind_index)
            .and_then(|global_id| bindable_color_value(&self.bindable_colors, global_id))
        {
            return Some(value);
        }
        self.bindable_colors
            .iter()
            .find(|bindable_color| bindable_color.has_data_bind_index(data_bind_index))
            .map(|bindable_color| bindable_color.value)
    }
    pub fn default_view_model_enum_source_value_for_data_bind(
        &self,
        data_bind_index: usize,
    ) -> Option<u64> {
        self.data_bind_graph
            .default_view_model_enum_source_value_for_data_bind(data_bind_index)
    }
    pub fn bindable_enum_value_for_data_bind(&self, data_bind_index: usize) -> Option<u64> {
        if let Some(value) = self
            .data_bind_graph
            .enum_target_global_id_for_data_bind(data_bind_index)
            .and_then(|global_id| bindable_enum_value(&self.bindable_enums, global_id))
        {
            return Some(value);
        }
        self.bindable_enums
            .iter()
            .find(|bindable_enum| bindable_enum.has_data_bind_index(data_bind_index))
            .map(|bindable_enum| bindable_enum.value)
    }
    pub fn default_view_model_symbol_list_index_source_value_for_data_bind(
        &self,
        data_bind_index: usize,
    ) -> Option<u64> {
        self.data_bind_graph
            .default_view_model_symbol_list_index_source_value_for_data_bind(data_bind_index)
    }
    pub fn bindable_integer_value_for_data_bind(&self, data_bind_index: usize) -> Option<u64> {
        if let Some(value) = self
            .data_bind_graph
            .integer_target_global_id_for_data_bind(data_bind_index)
            .and_then(|global_id| bindable_integer_value(&self.bindable_integers, global_id))
        {
            return Some(value);
        }
        self.bindable_integers
            .iter()
            .find(|bindable_integer| bindable_integer.has_data_bind_index(data_bind_index))
            .map(|bindable_integer| bindable_integer.value)
    }
    pub fn default_view_model_asset_source_value_for_data_bind(
        &self,
        data_bind_index: usize,
    ) -> Option<u64> {
        self.data_bind_graph
            .default_view_model_asset_source_value_for_data_bind(data_bind_index)
    }
    pub fn bindable_asset_value_for_data_bind(&self, data_bind_index: usize) -> Option<u64> {
        if let Some(value) = self
            .data_bind_graph
            .asset_target_global_id_for_data_bind(data_bind_index)
            .and_then(|global_id| bindable_asset_value(&self.bindable_assets, global_id))
        {
            return Some(value);
        }
        self.bindable_assets
            .iter()
            .find(|bindable_asset| bindable_asset.has_data_bind_index(data_bind_index))
            .map(|bindable_asset| bindable_asset.value.asset_index())
    }
    pub fn default_view_model_artboard_source_value_for_data_bind(
        &self,
        data_bind_index: usize,
    ) -> Option<u64> {
        self.data_bind_graph
            .default_view_model_artboard_source_value_for_data_bind(data_bind_index)
    }
    pub fn bindable_artboard_value_for_data_bind(&self, data_bind_index: usize) -> Option<u64> {
        if let Some(value) = self
            .data_bind_graph
            .artboard_target_global_id_for_data_bind(data_bind_index)
            .and_then(|global_id| bindable_artboard_value(&self.bindable_artboards, global_id))
        {
            return Some(value);
        }
        self.bindable_artboards
            .iter()
            .find(|bindable_artboard| bindable_artboard.has_data_bind_index(data_bind_index))
            .map(|bindable_artboard| bindable_artboard.value)
    }
    pub fn default_view_model_trigger_source_value_for_data_bind(
        &self,
        data_bind_index: usize,
    ) -> Option<u64> {
        self.data_bind_graph
            .default_view_model_trigger_source_value_for_data_bind(data_bind_index)
    }
    fn set_key_frame_default_number_source_for_path(&mut self, path: &[u32], value: f32) -> bool {
        self.key_frame_data_bind_graphs
            .iter_mut()
            .flatten()
            .fold(false, |changed, graph| {
                graph.set_default_view_model_number_source_for_path(path, value) || changed
            })
    }
    fn set_key_frame_default_boolean_source_for_path(&mut self, path: &[u32], value: bool) -> bool {
        self.key_frame_data_bind_graphs
            .iter_mut()
            .flatten()
            .fold(false, |changed, graph| {
                graph.set_default_view_model_boolean_source_for_path(path, value) || changed
            })
    }
    fn set_key_frame_default_string_source_for_path(&mut self, path: &[u32], value: &[u8]) -> bool {
        self.key_frame_data_bind_graphs
            .iter_mut()
            .flatten()
            .fold(false, |changed, graph| {
                graph.set_default_view_model_string_source_for_path(path, value) || changed
            })
    }
    fn set_key_frame_default_color_source_for_path(&mut self, path: &[u32], value: u32) -> bool {
        self.key_frame_data_bind_graphs
            .iter_mut()
            .flatten()
            .fold(false, |changed, graph| {
                graph.set_default_view_model_color_source_for_path(path, value) || changed
            })
    }
    fn set_key_frame_active_source_for_path(
        &mut self,
        path: &[u32],
        value: RuntimeDataBindGraphValue,
    ) -> bool {
        self.key_frame_data_bind_graphs
            .iter_mut()
            .flatten()
            .fold(false, |changed, graph| {
                graph.set_active_view_model_source_for_path(path, value.clone()) || changed
            })
    }
    pub fn set_default_view_model_number_source_for_data_bind(
        &mut self,
        data_bind_index: usize,
        value: f32,
    ) -> bool {
        let path = self
            .data_bind_graph
            .source_path_for_data_bind(data_bind_index);
        let changed = self
            .data_bind_graph
            .set_default_view_model_number_source_for_data_bind(data_bind_index, value);
        let key_frame_changed = path
            .is_some_and(|path| self.set_key_frame_default_number_source_for_path(&path, value));
        if !changed && !key_frame_changed {
            return false;
        }
        self.needs_advance = true;
        true
    }
    pub fn set_default_view_model_number_source_by_source_handle(
        &mut self,
        handle: &RuntimeDefaultViewModelNumberSourceHandle,
        value: f32,
    ) -> bool {
        let changed = self
            .data_bind_graph
            .set_default_view_model_number_source_for_path(&handle.path, value);
        let key_frame_changed =
            self.set_key_frame_default_number_source_for_path(&handle.path, value);
        if !changed && !key_frame_changed {
            return false;
        }
        self.needs_advance = true;
        true
    }
    pub fn set_default_view_model_number_source_by_property_name(
        &mut self,
        file: &RuntimeFile,
        property_name: &str,
        value: f32,
    ) -> bool {
        let Some(path) =
            runtime_default_view_model_number_property_path_for_name(file, property_name)
        else {
            return false;
        };
        let changed = self
            .data_bind_graph
            .set_default_view_model_number_source_for_path(&path, value);
        let key_frame_changed = self.set_key_frame_default_number_source_for_path(&path, value);
        if !changed && !key_frame_changed {
            return false;
        }
        self.needs_advance = true;
        true
    }
    pub fn set_default_view_model_boolean_source_by_property_name(
        &mut self,
        file: &RuntimeFile,
        property_name: &str,
        value: bool,
    ) -> bool {
        let Some(path) =
            runtime_default_view_model_boolean_property_path_for_name(file, property_name)
        else {
            return false;
        };
        let changed = self
            .data_bind_graph
            .set_default_view_model_boolean_source_for_path(&path, value);
        let key_frame_changed = self.set_key_frame_default_boolean_source_for_path(&path, value);
        if !changed && !key_frame_changed {
            return false;
        }
        self.needs_advance = true;
        true
    }
    pub fn set_default_view_model_boolean_source_by_source_handle(
        &mut self,
        handle: &RuntimeDefaultViewModelBooleanSourceHandle,
        value: bool,
    ) -> bool {
        let changed = self
            .data_bind_graph
            .set_default_view_model_boolean_source_for_path(&handle.path, value);
        let key_frame_changed =
            self.set_key_frame_default_boolean_source_for_path(&handle.path, value);
        if !changed && !key_frame_changed {
            return false;
        }
        self.needs_advance = true;
        true
    }
    pub fn set_default_view_model_boolean_source_for_data_bind(
        &mut self,
        data_bind_index: usize,
        value: bool,
    ) -> bool {
        let path = self
            .data_bind_graph
            .source_path_for_data_bind(data_bind_index);
        let changed = self
            .data_bind_graph
            .set_default_view_model_boolean_source_for_data_bind(data_bind_index, value);
        let key_frame_changed = path
            .is_some_and(|path| self.set_key_frame_default_boolean_source_for_path(&path, value));
        if !changed && !key_frame_changed {
            return false;
        }
        self.needs_advance = true;
        true
    }
    pub fn set_default_view_model_string_source_by_property_name(
        &mut self,
        file: &RuntimeFile,
        property_name: &str,
        value: &[u8],
    ) -> bool {
        let Some(path) =
            runtime_default_view_model_string_property_path_for_name(file, property_name)
        else {
            return false;
        };
        let changed = self
            .data_bind_graph
            .set_default_view_model_string_source_for_path(&path, value);
        let key_frame_changed = self.set_key_frame_default_string_source_for_path(&path, value);
        if !changed && !key_frame_changed {
            return false;
        }
        self.needs_advance = true;
        true
    }
    pub fn set_default_view_model_string_source_by_source_handle(
        &mut self,
        handle: &RuntimeDefaultViewModelStringSourceHandle,
        value: &[u8],
    ) -> bool {
        let changed = self
            .data_bind_graph
            .set_default_view_model_string_source_for_path(&handle.path, value);
        let key_frame_changed =
            self.set_key_frame_default_string_source_for_path(&handle.path, value);
        if !changed && !key_frame_changed {
            return false;
        }
        self.needs_advance = true;
        true
    }
    pub fn set_default_view_model_string_source_for_data_bind(
        &mut self,
        data_bind_index: usize,
        value: &[u8],
    ) -> bool {
        let path = self
            .data_bind_graph
            .source_path_for_data_bind(data_bind_index);
        let changed = self
            .data_bind_graph
            .set_default_view_model_string_source_for_data_bind(data_bind_index, value);
        let key_frame_changed = path
            .is_some_and(|path| self.set_key_frame_default_string_source_for_path(&path, value));
        if !changed && !key_frame_changed {
            return false;
        }
        self.needs_advance = true;
        true
    }
    pub fn set_default_view_model_color_source_by_property_name(
        &mut self,
        file: &RuntimeFile,
        property_name: &str,
        value: u32,
    ) -> bool {
        let Some(path) =
            runtime_default_view_model_color_property_path_for_name(file, property_name)
        else {
            return false;
        };
        let changed = self
            .data_bind_graph
            .set_default_view_model_color_source_for_path(&path, value);
        let key_frame_changed = self.set_key_frame_default_color_source_for_path(&path, value);
        if !changed && !key_frame_changed {
            return false;
        }
        self.needs_advance = true;
        true
    }
    pub fn set_default_view_model_color_source_by_source_handle(
        &mut self,
        handle: &RuntimeDefaultViewModelColorSourceHandle,
        value: u32,
    ) -> bool {
        let changed = self
            .data_bind_graph
            .set_default_view_model_color_source_for_path(&handle.path, value);
        let key_frame_changed =
            self.set_key_frame_default_color_source_for_path(&handle.path, value);
        if !changed && !key_frame_changed {
            return false;
        }
        self.needs_advance = true;
        true
    }
    pub fn set_default_view_model_color_source_for_data_bind(
        &mut self,
        data_bind_index: usize,
        value: u32,
    ) -> bool {
        let path = self
            .data_bind_graph
            .source_path_for_data_bind(data_bind_index);
        let changed = self
            .data_bind_graph
            .set_default_view_model_color_source_for_data_bind(data_bind_index, value);
        let key_frame_changed =
            path.is_some_and(|path| self.set_key_frame_default_color_source_for_path(&path, value));
        if !changed && !key_frame_changed {
            return false;
        }
        self.needs_advance = true;
        true
    }
    pub fn set_default_view_model_enum_source_by_property_name(
        &mut self,
        file: &RuntimeFile,
        property_name: &str,
        value: u64,
    ) -> bool {
        let Some(path) =
            runtime_default_view_model_enum_property_path_for_name(file, property_name)
        else {
            return false;
        };
        if !self
            .data_bind_graph
            .set_default_view_model_enum_source_for_path(&path, value)
        {
            return false;
        }
        self.needs_advance = true;
        true
    }
    pub fn set_default_view_model_enum_source_by_source_handle(
        &mut self,
        handle: &RuntimeDefaultViewModelEnumSourceHandle,
        value: u64,
    ) -> bool {
        if !self
            .data_bind_graph
            .set_default_view_model_enum_source_for_path(&handle.path, value)
        {
            return false;
        }
        self.needs_advance = true;
        true
    }
    pub fn set_default_view_model_enum_source_for_data_bind(
        &mut self,
        data_bind_index: usize,
        value: u64,
    ) -> bool {
        if !self
            .data_bind_graph
            .set_default_view_model_enum_source_for_data_bind(data_bind_index, value)
        {
            return false;
        }
        self.needs_advance = true;
        true
    }
    pub fn set_default_view_model_symbol_list_index_source_by_property_name(
        &mut self,
        file: &RuntimeFile,
        property_name: &str,
        value: u64,
    ) -> bool {
        let Some(path) = runtime_default_view_model_symbol_list_index_property_path_for_name(
            file,
            property_name,
        ) else {
            return false;
        };
        if !self
            .data_bind_graph
            .set_default_view_model_symbol_list_index_source_for_path(&path, value)
        {
            return false;
        }
        self.needs_advance = true;
        true
    }
    pub fn set_default_view_model_symbol_list_index_source_by_source_handle(
        &mut self,
        handle: &RuntimeDefaultViewModelSymbolListIndexSourceHandle,
        value: u64,
    ) -> bool {
        if !self
            .data_bind_graph
            .set_default_view_model_symbol_list_index_source_for_path(&handle.path, value)
        {
            return false;
        }
        self.needs_advance = true;
        true
    }
    pub fn set_default_view_model_symbol_list_index_source_for_data_bind(
        &mut self,
        data_bind_index: usize,
        value: u64,
    ) -> bool {
        if !self
            .data_bind_graph
            .set_default_view_model_symbol_list_index_source_for_data_bind(data_bind_index, value)
        {
            return false;
        }
        self.needs_advance = true;
        true
    }
    pub fn set_default_view_model_asset_source_by_property_name(
        &mut self,
        file: &RuntimeFile,
        property_name: &str,
        value: u64,
    ) -> bool {
        let Some(path) =
            runtime_default_view_model_asset_property_path_for_name(file, property_name)
        else {
            return false;
        };
        if !self
            .data_bind_graph
            .set_default_view_model_asset_source_for_path(&path, value)
        {
            return false;
        }
        self.needs_advance = true;
        true
    }
    pub fn set_default_view_model_asset_source_by_source_handle(
        &mut self,
        handle: &RuntimeDefaultViewModelAssetSourceHandle,
        value: u64,
    ) -> bool {
        if !self
            .data_bind_graph
            .set_default_view_model_asset_source_for_path(&handle.path, value)
        {
            return false;
        }
        self.needs_advance = true;
        true
    }
    pub fn set_default_view_model_asset_source_for_data_bind(
        &mut self,
        data_bind_index: usize,
        value: u64,
    ) -> bool {
        if !self
            .data_bind_graph
            .set_default_view_model_asset_source_for_data_bind(data_bind_index, value)
        {
            return false;
        }
        self.needs_advance = true;
        true
    }
    pub fn set_default_view_model_artboard_source_for_data_bind(
        &mut self,
        data_bind_index: usize,
        value: u64,
    ) -> bool {
        if !self
            .data_bind_graph
            .set_default_view_model_artboard_source_for_data_bind(data_bind_index, value)
        {
            return false;
        }
        self.needs_advance = true;
        true
    }
    pub fn set_default_view_model_artboard_source_by_property_name(
        &mut self,
        file: &RuntimeFile,
        property_name: &str,
        value: u64,
    ) -> bool {
        let Some(path) =
            runtime_default_view_model_artboard_property_path_for_name(file, property_name)
        else {
            return false;
        };
        if !self
            .data_bind_graph
            .set_default_view_model_artboard_source_for_path(&path, value)
        {
            return false;
        }
        self.needs_advance = true;
        true
    }
    pub fn set_default_view_model_artboard_source_by_source_handle(
        &mut self,
        handle: &RuntimeDefaultViewModelArtboardSourceHandle,
        value: u64,
    ) -> bool {
        if !self
            .data_bind_graph
            .set_default_view_model_artboard_source_for_path(&handle.path, value)
        {
            return false;
        }
        self.needs_advance = true;
        true
    }
    pub fn set_default_view_model_list_source_item_count_for_data_bind(
        &mut self,
        data_bind_index: usize,
        item_count: usize,
    ) -> bool {
        if !self
            .data_bind_graph
            .set_default_view_model_list_source_item_count_for_data_bind(
                data_bind_index,
                item_count,
            )
        {
            return false;
        }
        self.needs_advance = true;
        true
    }
    pub fn set_default_view_model_list_source_item_count_by_property_name(
        &mut self,
        file: &RuntimeFile,
        property_name: &str,
        item_count: usize,
    ) -> bool {
        let Some(path) =
            runtime_default_view_model_list_property_path_for_name(file, property_name)
        else {
            return false;
        };
        if !self
            .data_bind_graph
            .set_default_view_model_list_source_item_count_for_path(&path, item_count)
        {
            return false;
        }
        self.needs_advance = true;
        true
    }
    pub fn set_default_view_model_list_source_item_count_by_source_handle(
        &mut self,
        handle: &RuntimeDefaultViewModelListSourceHandle,
        item_count: usize,
    ) -> bool {
        if !self
            .data_bind_graph
            .set_default_view_model_list_source_item_count_for_path(&handle.path, item_count)
        {
            return false;
        }
        self.needs_advance = true;
        true
    }
    pub fn set_default_view_model_view_model_source_for_data_bind(
        &mut self,
        data_bind_index: usize,
        instance_index: usize,
    ) -> bool {
        if !self
            .data_bind_graph
            .set_default_view_model_view_model_source_for_data_bind(data_bind_index, instance_index)
        {
            return false;
        }
        self.needs_advance = true;
        true
    }
    pub fn relink_default_view_model_view_model_source_for_data_bind(
        &mut self,
        data_bind_index: usize,
        instance_index: usize,
    ) -> bool {
        if !self
            .data_bind_graph
            .relink_default_view_model_view_model_source_for_data_bind(
                data_bind_index,
                instance_index,
            )
        {
            return false;
        }
        self.needs_advance = true;
        true
    }
    pub fn relink_default_view_model_view_model_source_by_property_name(
        &mut self,
        file: &RuntimeFile,
        property_name: &str,
        instance_index: usize,
    ) -> bool {
        let Some(path) =
            runtime_default_view_model_view_model_property_path_for_name(file, property_name)
        else {
            return false;
        };
        if !self
            .data_bind_graph
            .relink_default_view_model_view_model_source_for_path(&path, instance_index)
        {
            return false;
        }
        self.needs_advance = true;
        true
    }
    pub fn relink_default_view_model_view_model_source_by_source_handle(
        &mut self,
        handle: &RuntimeDefaultViewModelViewModelSourceHandle,
        instance_index: usize,
    ) -> bool {
        if !self
            .data_bind_graph
            .relink_default_view_model_view_model_source_for_path(&handle.path, instance_index)
        {
            return false;
        }
        self.needs_advance = true;
        true
    }
    pub fn relink_view_model_instance_view_model_source_for_data_bind(
        &mut self,
        data_bind_index: usize,
        instance_index: usize,
    ) -> bool {
        if !self
            .data_bind_graph
            .relink_view_model_instance_view_model_source_for_data_bind(
                data_bind_index,
                instance_index,
            )
        {
            return false;
        }
        self.needs_advance = true;
        true
    }
    pub fn relink_imported_view_model_context_view_model_source_for_data_bind(
        &mut self,
        context: &mut RuntimeImportedViewModelInstanceContext,
        data_bind_index: usize,
        instance_index: usize,
    ) -> bool {
        if !self
            .data_bind_graph
            .relink_imported_view_model_context_view_model_source_for_data_bind(
                context,
                data_bind_index,
                instance_index,
            )
        {
            return false;
        }
        self.needs_advance = true;
        true
    }
    pub fn set_imported_view_model_context_number_source_for_data_bind(
        &mut self,
        context: &mut RuntimeImportedViewModelInstanceContext,
        data_bind_index: usize,
        value: f32,
    ) -> bool {
        let path = self
            .data_bind_graph
            .source_path_for_data_bind(data_bind_index);
        let changed = self
            .data_bind_graph
            .set_imported_view_model_context_number_source_for_data_bind(
                context,
                data_bind_index,
                value,
            );
        let key_frame_changed = path.is_some_and(|path| {
            self.set_key_frame_active_source_for_path(
                &path,
                RuntimeDataBindGraphValue::Number(value),
            )
        });
        if !changed && !key_frame_changed {
            return false;
        }
        self.needs_advance = true;
        true
    }
    fn owned_context_listener_report_waits_for_nested_relative_relink(
        &self,
        listener_index: usize,
        changed_cell: Option<&RuntimeViewModelCell>,
    ) -> bool {
        let Some(changed_cell) = changed_cell else {
            return false;
        };
        let Some(listener) = self.view_model_listeners.get(listener_index) else {
            return false;
        };
        let Some(file) = self.scripted_listener_runtime_file.as_deref() else {
            return false;
        };
        let Some(manifest) = file.manifest() else {
            return false;
        };
        let definition = &listener.listener_definitions[listener.listener_index];
        listener.property_bindings.iter().any(|binding| {
            let binding_matches_changed_cell = binding
                .cell_binding
                .as_ref()
                .is_some_and(|bound| bound.cell.ptr_eq(changed_cell));
            if !binding_matches_changed_cell {
                return false;
            }
            let path = match binding.source {
                RuntimeViewModelListenerSource::Single => definition.view_model_path.as_ref(),
                RuntimeViewModelListenerSource::Input(input_index) => definition
                    .view_model_input_types
                    .get(input_index)
                    .and_then(|input| input.path()),
            };
            matches!(
                path,
                Some(RuntimeListenerViewModelPath::Relative {
                    resolved_name_ids,
                    ..
                }) if resolved_name_ids.len() > 1
                    && resolved_name_ids
                        .iter()
                        .all(|name_id| manifest.resolve_name(*name_id).is_some())
            )
        })
    }
    pub(super) fn write_owned_view_model_context_with_listener_boundary(
        &mut self,
        context: &mut RuntimeOwnedViewModelInstance,
        data_bind_index: usize,
        write: impl FnOnce(&mut RuntimeDataBindGraph, &mut RuntimeOwnedViewModelInstance, usize) -> bool,
    ) -> bool {
        let changed_cell = self
            .data_bind_graph
            .source_path_for_data_bind(data_bind_index)
            .and_then(|source_path| {
                let (&view_model_index, property_path) = source_path.split_first()?;
                (usize::try_from(view_model_index).ok()? == context.view_model_index)
                    .then_some(property_path)?
                    .iter()
                    .map(|property_index| usize::try_from(*property_index).ok())
                    .collect::<Option<Vec<_>>>()
            })
            .and_then(|property_path| context.cell_by_property_path(&property_path));
        let mut previously_reported = Vec::new();
        self.reported_listener_view_models
            .swap_into(&mut previously_reported);
        let changed = write(&mut self.data_bind_graph, context, data_bind_index);
        let mut reports_from_write = Vec::new();
        self.reported_listener_view_models
            .swap_into(&mut reports_from_write);
        for listener_index in previously_reported {
            self.reported_listener_view_models
                .report_data_bind(listener_index);
        }
        for listener_index in reports_from_write {
            if self.owned_context_listener_report_waits_for_nested_relative_relink(
                listener_index,
                changed_cell.as_ref(),
            ) {
                self.post_apply_listener_view_models.push(listener_index);
            } else {
                self.reported_listener_view_models
                    .report_data_bind(listener_index);
            }
        }
        changed
    }
    pub fn set_owned_view_model_context_number_source_for_data_bind(
        &mut self,
        context: &mut RuntimeOwnedViewModelInstance,
        data_bind_index: usize,
        value: f32,
    ) -> bool {
        let path = self
            .data_bind_graph
            .source_path_for_data_bind(data_bind_index);
        // C++ discovers a fully resolved nested-relative listener report
        // during the later DataBind occurrence pass. Capture only the exact
        // listener/cell reports produced by this external write; flat and
        // unrelated listeners retain their immediate pending boundary.
        let changed = self.write_owned_view_model_context_with_listener_boundary(
            context,
            data_bind_index,
            |graph, context, data_bind_index| {
                graph.set_owned_view_model_context_number_source_for_data_bind(
                    context,
                    data_bind_index,
                    value,
                )
            },
        );
        let key_frame_changed = path.is_some_and(|path| {
            self.set_key_frame_active_source_for_path(
                &path,
                RuntimeDataBindGraphValue::Number(value),
            )
        });
        if !changed && !key_frame_changed {
            return false;
        }
        self.needs_advance = true;
        true
    }
    pub fn set_owned_view_model_context_symbol_list_index_source_for_data_bind(
        &mut self,
        context: &mut RuntimeOwnedViewModelInstance,
        data_bind_index: usize,
        value: u64,
    ) -> bool {
        if !self.write_owned_view_model_context_with_listener_boundary(
            context,
            data_bind_index,
            |graph, context, data_bind_index| {
                graph.set_owned_view_model_context_symbol_list_index_source_for_data_bind(
                    context,
                    data_bind_index,
                    value,
                )
            },
        ) {
            return false;
        }
        self.needs_advance = true;
        true
    }
    pub fn set_owned_view_model_context_boolean_source_for_data_bind(
        &mut self,
        context: &mut RuntimeOwnedViewModelInstance,
        data_bind_index: usize,
        value: bool,
    ) -> bool {
        let path = self
            .data_bind_graph
            .source_path_for_data_bind(data_bind_index);
        let changed = self.write_owned_view_model_context_with_listener_boundary(
            context,
            data_bind_index,
            |graph, context, data_bind_index| {
                graph.set_owned_view_model_context_boolean_source_for_data_bind(
                    context,
                    data_bind_index,
                    value,
                )
            },
        );
        let key_frame_changed = path.is_some_and(|path| {
            self.set_key_frame_active_source_for_path(
                &path,
                RuntimeDataBindGraphValue::Boolean(value),
            )
        });
        if !changed && !key_frame_changed {
            return false;
        }
        self.needs_advance = true;
        true
    }
    pub fn set_owned_view_model_context_enum_source_for_data_bind(
        &mut self,
        context: &mut RuntimeOwnedViewModelInstance,
        data_bind_index: usize,
        value: u64,
    ) -> bool {
        if !self.write_owned_view_model_context_with_listener_boundary(
            context,
            data_bind_index,
            |graph, context, data_bind_index| {
                graph.set_owned_view_model_context_enum_source_for_data_bind(
                    context,
                    data_bind_index,
                    value,
                )
            },
        ) {
            return false;
        }
        self.needs_advance = true;
        true
    }
    pub fn set_owned_view_model_context_color_source_for_data_bind(
        &mut self,
        context: &mut RuntimeOwnedViewModelInstance,
        data_bind_index: usize,
        value: u32,
    ) -> bool {
        let path = self
            .data_bind_graph
            .source_path_for_data_bind(data_bind_index);
        let changed = self.write_owned_view_model_context_with_listener_boundary(
            context,
            data_bind_index,
            |graph, context, data_bind_index| {
                graph.set_owned_view_model_context_color_source_for_data_bind(
                    context,
                    data_bind_index,
                    value,
                )
            },
        );
        let key_frame_changed = path.is_some_and(|path| {
            self.set_key_frame_active_source_for_path(
                &path,
                RuntimeDataBindGraphValue::Color(value),
            )
        });
        if !changed && !key_frame_changed {
            return false;
        }
        self.needs_advance = true;
        true
    }
    pub fn set_owned_view_model_context_string_source_for_data_bind(
        &mut self,
        context: &mut RuntimeOwnedViewModelInstance,
        data_bind_index: usize,
        value: &[u8],
    ) -> bool {
        let path = self
            .data_bind_graph
            .source_path_for_data_bind(data_bind_index);
        let changed = self.write_owned_view_model_context_with_listener_boundary(
            context,
            data_bind_index,
            |graph, context, data_bind_index| {
                graph.set_owned_view_model_context_string_source_for_data_bind(
                    context,
                    data_bind_index,
                    value,
                )
            },
        );
        let key_frame_changed = path.is_some_and(|path| {
            self.set_key_frame_active_source_for_path(
                &path,
                RuntimeDataBindGraphValue::String(value.to_vec()),
            )
        });
        if !changed && !key_frame_changed {
            return false;
        }
        self.needs_advance = true;
        true
    }
    pub fn set_owned_view_model_context_list_source_item_count_for_data_bind(
        &mut self,
        context: &mut RuntimeOwnedViewModelInstance,
        data_bind_index: usize,
        item_count: usize,
    ) -> bool {
        if !self.write_owned_view_model_context_with_listener_boundary(
            context,
            data_bind_index,
            |graph, context, data_bind_index| {
                graph.set_owned_view_model_context_list_source_item_count_for_data_bind(
                    context,
                    data_bind_index,
                    item_count,
                )
            },
        ) {
            return false;
        }
        self.needs_advance = true;
        true
    }
    pub fn set_owned_view_model_context_asset_source_for_data_bind(
        &mut self,
        context: &mut RuntimeOwnedViewModelInstance,
        data_bind_index: usize,
        value: u64,
    ) -> bool {
        if !self.write_owned_view_model_context_with_listener_boundary(
            context,
            data_bind_index,
            |graph, context, data_bind_index| {
                graph.set_owned_view_model_context_asset_source_for_data_bind(
                    context,
                    data_bind_index,
                    value,
                )
            },
        ) {
            return false;
        }
        self.needs_advance = true;
        true
    }
    pub fn set_owned_view_model_context_artboard_source_for_data_bind(
        &mut self,
        context: &mut RuntimeOwnedViewModelInstance,
        data_bind_index: usize,
        value: u64,
    ) -> bool {
        if !self.write_owned_view_model_context_with_listener_boundary(
            context,
            data_bind_index,
            |graph, context, data_bind_index| {
                graph.set_owned_view_model_context_artboard_source_for_data_bind(
                    context,
                    data_bind_index,
                    value,
                )
            },
        ) {
            return false;
        }
        self.needs_advance = true;
        true
    }
    pub fn set_owned_view_model_context_view_model_source_for_data_bind(
        &mut self,
        context: &mut RuntimeOwnedViewModelInstance,
        data_bind_index: usize,
        instance_index: usize,
    ) -> bool {
        if !self.write_owned_view_model_context_with_listener_boundary(
            context,
            data_bind_index,
            |graph, context, data_bind_index| {
                graph.set_owned_view_model_context_view_model_source_for_data_bind(
                    context,
                    data_bind_index,
                    instance_index,
                )
            },
        ) {
            return false;
        }
        self.needs_advance = true;
        true
    }
    pub fn set_imported_view_model_context_boolean_source_for_data_bind(
        &mut self,
        context: &mut RuntimeImportedViewModelInstanceContext,
        data_bind_index: usize,
        value: bool,
    ) -> bool {
        let path = self
            .data_bind_graph
            .source_path_for_data_bind(data_bind_index);
        let changed = self
            .data_bind_graph
            .set_imported_view_model_context_boolean_source_for_data_bind(
                context,
                data_bind_index,
                value,
            );
        let key_frame_changed = path.is_some_and(|path| {
            self.set_key_frame_active_source_for_path(
                &path,
                RuntimeDataBindGraphValue::Boolean(value),
            )
        });
        if !changed && !key_frame_changed {
            return false;
        }
        self.needs_advance = true;
        true
    }
    pub fn set_imported_view_model_context_string_source_for_data_bind(
        &mut self,
        context: &mut RuntimeImportedViewModelInstanceContext,
        data_bind_index: usize,
        value: &[u8],
    ) -> bool {
        let path = self
            .data_bind_graph
            .source_path_for_data_bind(data_bind_index);
        let changed = self
            .data_bind_graph
            .set_imported_view_model_context_string_source_for_data_bind(
                context,
                data_bind_index,
                value,
            );
        let key_frame_changed = path.is_some_and(|path| {
            self.set_key_frame_active_source_for_path(
                &path,
                RuntimeDataBindGraphValue::String(value.to_vec()),
            )
        });
        if !changed && !key_frame_changed {
            return false;
        }
        self.needs_advance = true;
        true
    }
    pub fn set_imported_view_model_context_color_source_for_data_bind(
        &mut self,
        context: &mut RuntimeImportedViewModelInstanceContext,
        data_bind_index: usize,
        value: u32,
    ) -> bool {
        let path = self
            .data_bind_graph
            .source_path_for_data_bind(data_bind_index);
        let changed = self
            .data_bind_graph
            .set_imported_view_model_context_color_source_for_data_bind(
                context,
                data_bind_index,
                value,
            );
        let key_frame_changed = path.is_some_and(|path| {
            self.set_key_frame_active_source_for_path(
                &path,
                RuntimeDataBindGraphValue::Color(value),
            )
        });
        if !changed && !key_frame_changed {
            return false;
        }
        self.needs_advance = true;
        true
    }
    pub fn set_imported_view_model_context_enum_source_for_data_bind(
        &mut self,
        context: &mut RuntimeImportedViewModelInstanceContext,
        data_bind_index: usize,
        value: u64,
    ) -> bool {
        if !self
            .data_bind_graph
            .set_imported_view_model_context_enum_source_for_data_bind(
                context,
                data_bind_index,
                value,
            )
        {
            return false;
        }
        self.needs_advance = true;
        true
    }
    pub fn set_imported_view_model_context_symbol_list_index_source_for_data_bind(
        &mut self,
        context: &mut RuntimeImportedViewModelInstanceContext,
        data_bind_index: usize,
        value: u64,
    ) -> bool {
        if !self
            .data_bind_graph
            .set_imported_view_model_context_symbol_list_index_source_for_data_bind(
                context,
                data_bind_index,
                value,
            )
        {
            return false;
        }
        self.needs_advance = true;
        true
    }
    pub fn set_imported_view_model_context_asset_source_for_data_bind(
        &mut self,
        context: &mut RuntimeImportedViewModelInstanceContext,
        data_bind_index: usize,
        value: u64,
    ) -> bool {
        if !self
            .data_bind_graph
            .set_imported_view_model_context_asset_source_for_data_bind(
                context,
                data_bind_index,
                value,
            )
        {
            return false;
        }
        self.needs_advance = true;
        true
    }
    pub fn set_imported_view_model_context_artboard_source_for_data_bind(
        &mut self,
        context: &mut RuntimeImportedViewModelInstanceContext,
        data_bind_index: usize,
        value: u64,
    ) -> bool {
        if !self
            .data_bind_graph
            .set_imported_view_model_context_artboard_source_for_data_bind(
                context,
                data_bind_index,
                value,
            )
        {
            return false;
        }
        self.needs_advance = true;
        true
    }
    pub fn set_imported_view_model_context_list_source_item_count_for_data_bind(
        &mut self,
        context: &mut RuntimeImportedViewModelInstanceContext,
        data_bind_index: usize,
        item_count: usize,
    ) -> bool {
        if !self
            .data_bind_graph
            .set_imported_view_model_context_list_source_item_count_for_data_bind(
                context,
                data_bind_index,
                item_count,
            )
        {
            return false;
        }
        self.needs_advance = true;
        true
    }
    pub fn set_data_bind_formula_random_values(&mut self, values: &[f32]) {
        self.data_bind_graph.set_formula_random_values(values);
        for graph in self.key_frame_data_bind_graphs.iter_mut().flatten() {
            graph.set_formula_random_values(values);
            graph.mark_default_view_model_bindings_dirty();
        }
    }
    pub fn data_bind_formula_random_call_count(&self) -> usize {
        self.data_bind_graph.formula_random_call_count()
    }
    /// Retained transition-duration DataBind occurrence count, in authored order.
    ///
    /// This is exposed for pinned-C++ differential evidence. Each occurrence
    /// has independent converter/source ownership even when several target the
    /// same transition.
    #[doc(hidden)]
    pub fn transition_duration_binding_count(&self) -> usize {
        self.transition_durations.len()
    }
    /// Current retained value of one authored transition-duration occurrence.
    #[doc(hidden)]
    pub fn transition_duration_binding_value(&self, index: usize) -> Option<f32> {
        self.transition_durations
            .get(index)
            .map(StateMachineTransitionDurationInstance::value)
    }
    fn apply_default_view_model_bindings(
        &mut self,
        include_view_models: bool,
        phase: RuntimeDataBindGraphApplyPhase,
    ) -> Result<(), ScriptError> {
        self.data_bind_graph.apply_default_view_model_bindings(
            RuntimeDataBindGraphTargetsMut {
                numbers: &mut self.bindable_numbers,
                integers: &mut self.bindable_integers,
                booleans: &mut self.bindable_booleans,
                strings: &mut self.bindable_strings,
                colors: &mut self.bindable_colors,
                enums: &mut self.bindable_enums,
                assets: &mut self.bindable_assets,
                artboards: &mut self.bindable_artboards,
                lists: &mut self.bindable_lists,
                triggers: &mut self.bindable_triggers,
                view_models: &mut self.bindable_view_models,
                transition_durations: &mut self.transition_durations,
                include_view_models,
            },
            phase,
        )
    }
    pub(super) fn public_update_default_view_model_binding(
        &mut self,
        data_bind_index: usize,
    ) -> Result<(), ScriptError> {
        self.data_bind_graph
            .public_update_default_view_model_binding(
                data_bind_index,
                RuntimeDataBindGraphTargetsMut {
                    numbers: &mut self.bindable_numbers,
                    integers: &mut self.bindable_integers,
                    booleans: &mut self.bindable_booleans,
                    strings: &mut self.bindable_strings,
                    colors: &mut self.bindable_colors,
                    enums: &mut self.bindable_enums,
                    assets: &mut self.bindable_assets,
                    artboards: &mut self.bindable_artboards,
                    lists: &mut self.bindable_lists,
                    triggers: &mut self.bindable_triggers,
                    view_models: &mut self.bindable_view_models,
                    transition_durations: &mut self.transition_durations,
                    include_view_models: true,
                },
            )
    }
    pub(super) fn update_default_view_model_binding(
        &mut self,
        data_bind_index: usize,
        include_view_models: bool,
        phase: RuntimeDataBindGraphApplyPhase,
    ) -> Result<(), ScriptError> {
        self.data_bind_graph.update_default_view_model_binding(
            data_bind_index,
            RuntimeDataBindGraphTargetsMut {
                numbers: &mut self.bindable_numbers,
                integers: &mut self.bindable_integers,
                booleans: &mut self.bindable_booleans,
                strings: &mut self.bindable_strings,
                colors: &mut self.bindable_colors,
                enums: &mut self.bindable_enums,
                assets: &mut self.bindable_assets,
                artboards: &mut self.bindable_artboards,
                lists: &mut self.bindable_lists,
                triggers: &mut self.bindable_triggers,
                view_models: &mut self.bindable_view_models,
                transition_durations: &mut self.transition_durations,
                include_view_models,
            },
            phase,
        )
    }
}
