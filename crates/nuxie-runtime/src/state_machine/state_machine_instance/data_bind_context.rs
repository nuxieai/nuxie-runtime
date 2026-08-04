// State-machine instance integration for the C++ `data_bind_context.cpp` source.
use super::*;
impl StateMachineInstance {
    #[doc(hidden)]
    pub fn scripted_listener_data_context_view_models(
        &self,
        file: &RuntimeFile,
        fallback_root: Option<&RuntimeOwnedViewModelHandle>,
    ) -> (Option<ScriptViewModel>, Vec<Option<ScriptViewModel>>) {
        if let Some(data_context) = self.owned_data_context.as_ref() {
            let mut contexts = data_context.main_context_slots(file).into_iter();
            if let Some(main) = contexts.next() {
                let main = main.and_then(|context| {
                    crate::script_view_model_from_owned_context(file, &context)
                });
                let parents = contexts
                    .map(|context| {
                        context.and_then(|context| {
                            crate::script_view_model_from_owned_context(file, &context)
                        })
                    })
                    .collect();
                return (main, parents);
            }
            // An occurrence-owned but empty DataContext is still
            // authoritative. Pinned C++ asks only this ScriptedObject's
            // DataContext; it does not substitute an unrelated facade root
            // when `mainViewModelInstance()` is null
            // (`lua_scripted_context.cpp:129-146`).
            return (None, Vec::new());
        }
        (
            fallback_root.and_then(|root| crate::script_view_model_from_owned(file, root)),
            Vec::new(),
        )
    }
    /// Retain the exact local/global/parent DataContext chain that C++
    /// supplies to a freshly projected ScriptInputArtboard.
    #[doc(hidden)]
    pub fn scripted_listener_artboard_parent_context(
        &self,
        fallback_root: Option<&RuntimeOwnedViewModelContextHandle>,
    ) -> Option<ScriptArtboardParentContext> {
        self.owned_data_context
            .clone()
            .map(ScriptArtboardParentContext::from_runtime)
            .or_else(|| {
                fallback_root.map(|context| {
                    ScriptArtboardParentContext::from_runtime(
                        RuntimeOwnedDataContext::from_context_handle(context),
                    )
                })
            })
    }
    #[doc(hidden)]
    pub fn has_scripted_listener_data_context(&self) -> bool {
        self.owned_data_context.is_some()
    }
    #[doc(hidden)]
    pub fn scripted_listener_bound_view_model(
        &self,
        file: &RuntimeFile,
        path: &crate::ScriptInputViewModelPropertyPath,
        fallback_root: Option<&RuntimeOwnedViewModelContextHandle>,
    ) -> Option<Option<ScriptViewModel>> {
        match self.owned_data_context.as_ref() {
            Some(data_context) => data_context.bound_script_view_model(file, path),
            None => fallback_root.and_then(|context| {
                crate::script_input_viewmodel_property::
                    bound_script_view_model_property_from_owned_path(file, context, path)
            }),
        }
    }
    /// Resolve one ScriptInput DataBind through this concrete occurrence's
    /// cloned converter state.
    ///
    /// This is intentionally keyed by both action and input identity. A
    /// shared file-level converter helper would alias state across
    /// StateMachineInstances, unlike C++ `ScriptedObject::cloneProperties`.
    fn resolve_scripted_listener_input_binding(
        &mut self,
        file: &RuntimeFile,
        context: &RuntimeOwnedViewModelInstance,
        action_global_id: u32,
        input_global_id: u32,
        emit_unchanged: bool,
    ) -> Result<
        Option<super::scripted_listener_action::RuntimeScriptedListenerBoundValue>,
        ScriptError,
    > {
        let occurrence = self
            .scripted_object_bindings
            .iter_mut()
            .find(|occurrence| occurrence.action_global_id() == action_global_id)
            .ok_or_else(|| {
                ScriptError::new(format!(
                    "state machine has no scripted listener binding occurrence global {action_global_id}",
                ))
            })?;
        occurrence.resolve(file, context, input_global_id, emit_unchanged)
    }
    #[doc(hidden)]
    pub fn resolve_scripted_listener_scalar_binding(
        &mut self,
        file: &RuntimeFile,
        context: &RuntimeOwnedViewModelInstance,
        action_global_id: u32,
        input_global_id: u32,
        emit_unchanged: bool,
    ) -> Result<Option<ScriptValue>, ScriptError> {
        match self.resolve_scripted_listener_input_binding(
            file,
            context,
            action_global_id,
            input_global_id,
            emit_unchanged,
        )? {
            Some(super::scripted_listener_action::RuntimeScriptedListenerBoundValue::Value(
                value,
            )) => Ok(Some(value)),
            Some(value) => Err(ScriptError::new(format!(
                "scripted listener scalar input global {input_global_id} received {value:?}",
            ))),
            None => Ok(None),
        }
    }
    #[doc(hidden)]
    pub fn resolve_scripted_listener_artboard_binding(
        &mut self,
        file: &RuntimeFile,
        context: &RuntimeOwnedViewModelInstance,
        action_global_id: u32,
        input_global_id: u32,
        emit_unchanged: bool,
    ) -> Result<Option<u64>, ScriptError> {
        match self.resolve_scripted_listener_input_binding(
            file,
            context,
            action_global_id,
            input_global_id,
            emit_unchanged,
        )? {
            Some(super::scripted_listener_action::RuntimeScriptedListenerBoundValue::Artboard(
                value,
            )) => Ok(Some(value)),
            Some(value) => Err(ScriptError::new(format!(
                "scripted listener artboard input global {input_global_id} received {value:?}",
            ))),
            None => Ok(None),
        }
    }
    #[doc(hidden)]
    pub fn resolve_scripted_listener_trigger_binding(
        &mut self,
        file: &RuntimeFile,
        context: &RuntimeOwnedViewModelInstance,
        action_global_id: u32,
        input_global_id: u32,
        emit_unchanged: bool,
    ) -> Result<Option<u64>, ScriptError> {
        match self.resolve_scripted_listener_input_binding(
            file,
            context,
            action_global_id,
            input_global_id,
            emit_unchanged,
        )? {
            Some(super::scripted_listener_action::RuntimeScriptedListenerBoundValue::Trigger(
                value,
            )) => Ok(Some(value)),
            Some(value) => Err(ScriptError::new(format!(
                "scripted listener trigger input global {input_global_id} received {value:?}",
            ))),
            None => Ok(None),
        }
    }
    pub fn default_view_model_number_source_handle_by_property_name(
        &self,
        file: &RuntimeFile,
        property_name: &str,
    ) -> Option<RuntimeDefaultViewModelNumberSourceHandle> {
        let path = runtime_default_view_model_number_property_path_for_name(file, property_name)?;
        Some(RuntimeDefaultViewModelNumberSourceHandle { path })
    }
    pub fn default_view_model_number_source_handle_by_property_name_path(
        &self,
        file: &RuntimeFile,
        property_path: &str,
    ) -> Option<RuntimeDefaultViewModelNumberSourceHandle> {
        let path =
            runtime_default_view_model_number_property_path_for_name_path(file, property_path)?;
        Some(RuntimeDefaultViewModelNumberSourceHandle { path })
    }
    pub fn default_view_model_boolean_source_handle_by_property_name(
        &self,
        file: &RuntimeFile,
        property_name: &str,
    ) -> Option<RuntimeDefaultViewModelBooleanSourceHandle> {
        let path = runtime_default_view_model_boolean_property_path_for_name(file, property_name)?;
        Some(RuntimeDefaultViewModelBooleanSourceHandle { path })
    }
    pub fn default_view_model_boolean_source_handle_by_property_name_path(
        &self,
        file: &RuntimeFile,
        property_path: &str,
    ) -> Option<RuntimeDefaultViewModelBooleanSourceHandle> {
        let path =
            runtime_default_view_model_boolean_property_path_for_name_path(file, property_path)?;
        Some(RuntimeDefaultViewModelBooleanSourceHandle { path })
    }
    pub fn default_view_model_string_source_handle_by_property_name(
        &self,
        file: &RuntimeFile,
        property_name: &str,
    ) -> Option<RuntimeDefaultViewModelStringSourceHandle> {
        let path = runtime_default_view_model_string_property_path_for_name(file, property_name)?;
        Some(RuntimeDefaultViewModelStringSourceHandle { path })
    }
    pub fn default_view_model_string_source_handle_by_property_name_path(
        &self,
        file: &RuntimeFile,
        property_path: &str,
    ) -> Option<RuntimeDefaultViewModelStringSourceHandle> {
        let path =
            runtime_default_view_model_string_property_path_for_name_path(file, property_path)?;
        Some(RuntimeDefaultViewModelStringSourceHandle { path })
    }
    pub fn default_view_model_color_source_handle_by_property_name(
        &self,
        file: &RuntimeFile,
        property_name: &str,
    ) -> Option<RuntimeDefaultViewModelColorSourceHandle> {
        let path = runtime_default_view_model_color_property_path_for_name(file, property_name)?;
        Some(RuntimeDefaultViewModelColorSourceHandle { path })
    }
    pub fn default_view_model_color_source_handle_by_property_name_path(
        &self,
        file: &RuntimeFile,
        property_path: &str,
    ) -> Option<RuntimeDefaultViewModelColorSourceHandle> {
        let path =
            runtime_default_view_model_color_property_path_for_name_path(file, property_path)?;
        Some(RuntimeDefaultViewModelColorSourceHandle { path })
    }
    pub fn default_view_model_enum_source_handle_by_property_name(
        &self,
        file: &RuntimeFile,
        property_name: &str,
    ) -> Option<RuntimeDefaultViewModelEnumSourceHandle> {
        let path = runtime_default_view_model_enum_property_path_for_name(file, property_name)?;
        Some(RuntimeDefaultViewModelEnumSourceHandle { path })
    }
    pub fn default_view_model_enum_source_handle_by_property_name_path(
        &self,
        file: &RuntimeFile,
        property_path: &str,
    ) -> Option<RuntimeDefaultViewModelEnumSourceHandle> {
        let path =
            runtime_default_view_model_enum_property_path_for_name_path(file, property_path)?;
        Some(RuntimeDefaultViewModelEnumSourceHandle { path })
    }
    pub fn default_view_model_symbol_list_index_source_handle_by_property_name(
        &self,
        file: &RuntimeFile,
        property_name: &str,
    ) -> Option<RuntimeDefaultViewModelSymbolListIndexSourceHandle> {
        let path = runtime_default_view_model_symbol_list_index_property_path_for_name(
            file,
            property_name,
        )?;
        Some(RuntimeDefaultViewModelSymbolListIndexSourceHandle { path })
    }
    pub fn default_view_model_symbol_list_index_source_handle_by_property_name_path(
        &self,
        file: &RuntimeFile,
        property_path: &str,
    ) -> Option<RuntimeDefaultViewModelSymbolListIndexSourceHandle> {
        let path = runtime_default_view_model_symbol_list_index_property_path_for_name_path(
            file,
            property_path,
        )?;
        Some(RuntimeDefaultViewModelSymbolListIndexSourceHandle { path })
    }
    pub fn default_view_model_asset_source_handle_by_property_name(
        &self,
        file: &RuntimeFile,
        property_name: &str,
    ) -> Option<RuntimeDefaultViewModelAssetSourceHandle> {
        let path = runtime_default_view_model_asset_property_path_for_name(file, property_name)?;
        Some(RuntimeDefaultViewModelAssetSourceHandle { path })
    }
    pub fn default_view_model_asset_source_handle_by_property_name_path(
        &self,
        file: &RuntimeFile,
        property_path: &str,
    ) -> Option<RuntimeDefaultViewModelAssetSourceHandle> {
        let path =
            runtime_default_view_model_asset_property_path_for_name_path(file, property_path)?;
        Some(RuntimeDefaultViewModelAssetSourceHandle { path })
    }
    pub fn default_view_model_artboard_source_handle_by_property_name(
        &self,
        file: &RuntimeFile,
        property_name: &str,
    ) -> Option<RuntimeDefaultViewModelArtboardSourceHandle> {
        let path = runtime_default_view_model_artboard_property_path_for_name(file, property_name)?;
        Some(RuntimeDefaultViewModelArtboardSourceHandle { path })
    }
    pub fn default_view_model_artboard_source_handle_by_property_name_path(
        &self,
        file: &RuntimeFile,
        property_path: &str,
    ) -> Option<RuntimeDefaultViewModelArtboardSourceHandle> {
        let path =
            runtime_default_view_model_artboard_property_path_for_name_path(file, property_path)?;
        Some(RuntimeDefaultViewModelArtboardSourceHandle { path })
    }
    pub fn default_view_model_list_source_handle_by_property_name(
        &self,
        file: &RuntimeFile,
        property_name: &str,
    ) -> Option<RuntimeDefaultViewModelListSourceHandle> {
        let path = runtime_default_view_model_list_property_path_for_name(file, property_name)?;
        Some(RuntimeDefaultViewModelListSourceHandle { path })
    }
    pub fn default_view_model_list_source_handle_by_property_name_path(
        &self,
        file: &RuntimeFile,
        property_path: &str,
    ) -> Option<RuntimeDefaultViewModelListSourceHandle> {
        let path =
            runtime_default_view_model_list_property_path_for_name_path(file, property_path)?;
        Some(RuntimeDefaultViewModelListSourceHandle { path })
    }
    pub fn default_view_model_view_model_source_handle_by_property_name(
        &self,
        file: &RuntimeFile,
        property_name: &str,
    ) -> Option<RuntimeDefaultViewModelViewModelSourceHandle> {
        let path =
            runtime_default_view_model_view_model_property_path_for_name(file, property_name)?;
        Some(RuntimeDefaultViewModelViewModelSourceHandle { path })
    }
    pub fn default_view_model_view_model_source_handle_by_property_name_path(
        &self,
        file: &RuntimeFile,
        property_path: &str,
    ) -> Option<RuntimeDefaultViewModelViewModelSourceHandle> {
        let path =
            runtime_default_view_model_view_model_property_path_for_name_path(file, property_path)?;
        Some(RuntimeDefaultViewModelViewModelSourceHandle { path })
    }
    pub fn relink_view_model_instance_view_model_source_by_property_name_path(
        &mut self,
        file: &RuntimeFile,
        property_path: &str,
        instance_index: usize,
    ) -> bool {
        if !self
            .data_bind_graph
            .relink_view_model_instance_view_model_source_by_property_name_path(
                file,
                property_path,
                instance_index,
            )
        {
            return false;
        }
        self.needs_advance = true;
        true
    }
    pub(super) fn apply_listener_view_model_change_at_property_path(
        context: &mut RuntimeOwnedViewModelInstance,
        property_path: &[usize],
        value: &RuntimeListenerViewModelChangeValue,
        asset_value: Option<&RuntimeBindableAssetValue>,
    ) -> Option<bool> {
        match value {
            RuntimeListenerViewModelChangeValue::Number(value) => {
                context.number_value_by_property_path(property_path)?;
                Some(context.set_number_by_property_path(property_path, *value))
            }
            RuntimeListenerViewModelChangeValue::Integer(value) => {
                context.symbol_list_index_value_by_property_path(property_path)?;
                Some(context.set_symbol_list_index_by_property_path(property_path, *value))
            }
            RuntimeListenerViewModelChangeValue::Color(value) => {
                context.color_value_by_property_path(property_path)?;
                Some(context.set_color_by_property_path(property_path, *value))
            }
            RuntimeListenerViewModelChangeValue::String(value) => {
                context.string_value_by_property_path(property_path)?;
                Some(context.set_string_by_property_path(property_path, value))
            }
            RuntimeListenerViewModelChangeValue::Enum(value) => {
                context.enum_value_by_property_path(property_path)?;
                Some(context.set_enum_by_property_path(property_path, *value))
            }
            RuntimeListenerViewModelChangeValue::Asset(_) => {
                let asset_value = asset_value?;
                if context
                    .font_asset_value_by_property_path(property_path)
                    .is_some()
                {
                    let font_value = asset_value.font_data_bind_value().unwrap_or_else(|| {
                        RuntimeFontAssetValue::from_file_asset_index(asset_value.asset_index())
                    });
                    return Some(context.apply_font_asset_data_bind_value_by_property_path(
                        property_path,
                        &font_value,
                    ));
                }
                if context
                    .blob_asset_value_by_property_path(property_path)
                    .is_some()
                {
                    let blob_value = asset_value.blob_data_bind_value().unwrap_or_else(|| {
                        RuntimeBlobAssetValue::from_file_asset_index(asset_value.asset_index())
                    });
                    return Some(context.apply_blob_asset_data_bind_value_by_property_path(
                        property_path,
                        &blob_value,
                    ));
                }
                context.asset_value_by_property_path(property_path)?;
                Some(
                    context.set_asset_by_property_path(
                        property_path,
                        asset_value.data_bind_asset_index(),
                    ),
                )
            }
            RuntimeListenerViewModelChangeValue::Artboard(value) => {
                context.artboard_value_by_property_path(property_path)?;
                Some(context.set_artboard_by_property_path(property_path, *value))
            }
            RuntimeListenerViewModelChangeValue::Trigger(_) => None,
            RuntimeListenerViewModelChangeValue::Boolean(value) => {
                context.boolean_value_by_property_path(property_path)?;
                Some(context.set_boolean_by_property_path(property_path, *value))
            }
            RuntimeListenerViewModelChangeValue::List(value) => {
                Some(context.set_list_item_count_by_property_path(
                    property_path,
                    usize::try_from(*value).ok()?,
                ))
            }
            RuntimeListenerViewModelChangeValue::ViewModel(_) => None,
        }
    }
}
