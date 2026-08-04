// State-machine instance integration for the C++ `listener_viewmodel_change.cpp` source.
use super::*;

impl RuntimeStateMachineListenerActionExecutor<'_> {
    pub(in crate::state_machine) fn perform_scheduled_view_model_change(
        &mut self,
        artboard: &mut ArtboardInstance,
        bindable_global_id: u32,
        value: &RuntimeListenerViewModelChangeValue,
        mut targets: RuntimeScheduledListenerActionTargetsMut<'_>,
    ) -> bool {
        if !self.data_bind_facilities_ready {
            return false;
        }
        let data_bind_index = self
            .data_bind_graph
            .bindable_data_bind_to_source_index(bindable_global_id);
        let artboard_value = match value {
            RuntimeListenerViewModelChangeValue::Number(value) => {
                RuntimeDataBindGraphValue::Number(*value)
            }
            RuntimeListenerViewModelChangeValue::Integer(value) => {
                RuntimeDataBindGraphValue::SymbolListIndex(*value)
            }
            RuntimeListenerViewModelChangeValue::Color(value) => {
                RuntimeDataBindGraphValue::Color(*value)
            }
            RuntimeListenerViewModelChangeValue::String(value) => {
                RuntimeDataBindGraphValue::String(value.clone())
            }
            RuntimeListenerViewModelChangeValue::Enum(value) => {
                RuntimeDataBindGraphValue::Enum(*value)
            }
            RuntimeListenerViewModelChangeValue::Asset(value) => {
                RuntimeDataBindGraphValue::Asset(value.data_bind_asset_index())
            }
            RuntimeListenerViewModelChangeValue::Artboard(value) => {
                RuntimeDataBindGraphValue::Artboard(*value)
            }
            RuntimeListenerViewModelChangeValue::Trigger(value) => {
                RuntimeDataBindGraphValue::Trigger(*value)
            }
            RuntimeListenerViewModelChangeValue::Boolean(value) => {
                RuntimeDataBindGraphValue::Boolean(*value)
            }
            RuntimeListenerViewModelChangeValue::List(value) => RuntimeDataBindGraphValue::List {
                item_count: usize::try_from(*value).unwrap_or(usize::MAX),
            },
            RuntimeListenerViewModelChangeValue::ViewModel(value) => {
                RuntimeDataBindGraphValue::ViewModel(*value)
            }
        };
        let path = data_bind_index.and_then(|data_bind_index| {
            self.data_bind_graph
                .source_path_for_data_bind(data_bind_index)
        });
        let source_changed = if let Some(data_bind_index) = data_bind_index
            && let Some(context) = self.owned_view_model_context.take()
        {
            let changed = self.perform_owned_view_model_change(
                &mut *context,
                data_bind_index,
                value,
                &mut targets,
            );
            self.owned_view_model_context = Some(context);
            changed
        } else if let Some(data_bind_index) = data_bind_index
            && self.owned_data_context.is_some()
        {
            self.perform_owned_data_context_change(data_bind_index, value, &mut targets)
        } else if let Some(data_bind_index) = data_bind_index {
            self.data_bind_graph
                .set_active_view_model_source_for_data_bind(data_bind_index, artboard_value.clone())
        } else {
            false
        };
        let target_dirtied = self
            .data_bind_graph
            .dirty_bindable_data_bind_to_target(bindable_global_id);
        if !source_changed && !target_dirtied {
            return false;
        }
        if source_changed && let Some(path) = path {
            artboard.set_artboard_data_bind_value_for_path(&path, artboard_value);
        }
        // Pinned `ListenerViewModelChange::perform` updates only the
        // target-to-source bind, then calls `addDirt(Bindings, true)` on the
        // paired source-to-target bind. It does not run `updateDataBinds`
        // inside the action FIFO (`listener_viewmodel_change.cpp:42-80`).
        // Keeping the target dirty until the normal data-bind boundary means
        // a later action in this same FIFO still observes its pre-batch
        // target value.
        true
    }

    fn perform_owned_data_context_change(
        &mut self,
        data_bind_index: usize,
        value: &RuntimeListenerViewModelChangeValue,
        targets: &mut RuntimeScheduledListenerActionTargetsMut<'_>,
    ) -> bool {
        let Some(source_path) = self
            .data_bind_graph
            .source_path_for_data_bind(data_bind_index)
        else {
            return false;
        };
        let Some((context_handle, property_path)) = self
            .owned_data_context
            .as_ref()
            .and_then(|context| context.resolved_property_path(&source_path))
        else {
            return false;
        };

        if let RuntimeListenerViewModelChangeValue::Trigger(value) = value {
            let Some(bindable_trigger) = targets
                .bindable_triggers
                .iter_mut()
                .find(|trigger| trigger.has_data_bind_index(data_bind_index))
            else {
                return false;
            };
            bindable_trigger.set_value(*value);
            let mut context = context_handle.borrow_mut();
            if !self
                .data_bind_graph
                .fire_owned_view_model_context_trigger_source_for_data_bind_at_property_path(
                    &mut context,
                    data_bind_index,
                    *value,
                    &property_path,
                )
            {
                return false;
            }
            return true;
        }

        let asset_value = match value {
            RuntimeListenerViewModelChangeValue::Asset(fallback) => Some(
                targets
                    .bindable_assets
                    .iter()
                    .find(|asset| asset.has_data_bind_index(data_bind_index))
                    .map(|asset| asset.value.clone())
                    .unwrap_or_else(|| fallback.clone()),
            ),
            _ => None,
        };
        let graph_value = match value {
            RuntimeListenerViewModelChangeValue::Number(value) => {
                RuntimeDataBindGraphValue::Number(*value)
            }
            RuntimeListenerViewModelChangeValue::Integer(value) => {
                RuntimeDataBindGraphValue::SymbolListIndex(*value)
            }
            RuntimeListenerViewModelChangeValue::Color(value) => {
                RuntimeDataBindGraphValue::Color(*value)
            }
            RuntimeListenerViewModelChangeValue::String(value) => {
                RuntimeDataBindGraphValue::String(value.clone())
            }
            RuntimeListenerViewModelChangeValue::Enum(value) => {
                RuntimeDataBindGraphValue::Enum(*value)
            }
            RuntimeListenerViewModelChangeValue::Asset(_) => RuntimeDataBindGraphValue::Asset(
                asset_value
                    .as_ref()
                    .map(RuntimeBindableAssetValue::data_bind_asset_index)
                    .unwrap_or_default(),
            ),
            RuntimeListenerViewModelChangeValue::Artboard(value) => {
                RuntimeDataBindGraphValue::Artboard(*value)
            }
            RuntimeListenerViewModelChangeValue::Boolean(value) => {
                RuntimeDataBindGraphValue::Boolean(*value)
            }
            RuntimeListenerViewModelChangeValue::List(value) => RuntimeDataBindGraphValue::List {
                item_count: usize::try_from(*value).unwrap_or(usize::MAX),
            },
            RuntimeListenerViewModelChangeValue::ViewModel(value) => {
                RuntimeDataBindGraphValue::ViewModel(*value)
            }
            RuntimeListenerViewModelChangeValue::Trigger(_) => unreachable!(),
        };
        let mut context = context_handle.borrow_mut();
        let Some(context_changed) =
            StateMachineInstance::apply_listener_view_model_change_at_property_path(
                &mut context,
                &property_path,
                value,
                asset_value.as_ref(),
            )
        else {
            return false;
        };
        let graph_changed = self
            .data_bind_graph
            .set_active_view_model_source_for_data_bind(data_bind_index, graph_value);
        if matches!(value, RuntimeListenerViewModelChangeValue::Number(_)) {
            // The listener wrote the retained cell above. Its owning binds,
            // including converter-operand dependencies, are already dirty;
            // fold that pushed dirt before this frame's data-bind pass.
            self.data_bind_graph.collect_retained_source_dirt();
        }
        context_changed || graph_changed
    }

    fn perform_owned_view_model_change(
        &mut self,
        context: &mut RuntimeOwnedViewModelInstance,
        data_bind_index: usize,
        value: &RuntimeListenerViewModelChangeValue,
        targets: &mut RuntimeScheduledListenerActionTargetsMut<'_>,
    ) -> bool {
        match value {
            RuntimeListenerViewModelChangeValue::Number(value) => self
                .data_bind_graph
                .set_owned_view_model_context_number_source_for_data_bind(
                    context,
                    data_bind_index,
                    *value,
                ),
            RuntimeListenerViewModelChangeValue::Integer(value) => self
                .data_bind_graph
                .set_owned_view_model_context_symbol_list_index_source_for_data_bind(
                    context,
                    data_bind_index,
                    *value,
                ),
            RuntimeListenerViewModelChangeValue::Color(value) => self
                .data_bind_graph
                .set_owned_view_model_context_color_source_for_data_bind(
                    context,
                    data_bind_index,
                    *value,
                ),
            RuntimeListenerViewModelChangeValue::String(value) => self
                .data_bind_graph
                .set_owned_view_model_context_string_source_for_data_bind(
                    context,
                    data_bind_index,
                    value,
                ),
            RuntimeListenerViewModelChangeValue::Enum(value) => self
                .data_bind_graph
                .set_owned_view_model_context_enum_source_for_data_bind(
                    context,
                    data_bind_index,
                    *value,
                ),
            RuntimeListenerViewModelChangeValue::Asset(value) => {
                if let Some(blob_value) = value.blob_data_bind_value() {
                    self.data_bind_graph
                        .set_owned_view_model_context_blob_asset_source_for_data_bind(
                            context,
                            data_bind_index,
                            &blob_value,
                        )
                } else if let Some(font_value) = value.font_data_bind_value() {
                    self.data_bind_graph
                        .set_owned_view_model_context_font_asset_source_for_data_bind(
                            context,
                            data_bind_index,
                            &font_value,
                        )
                } else {
                    self.data_bind_graph
                        .set_owned_view_model_context_asset_source_for_data_bind(
                            context,
                            data_bind_index,
                            value.data_bind_asset_index(),
                        )
                }
            }
            RuntimeListenerViewModelChangeValue::Artboard(value) => self
                .data_bind_graph
                .set_owned_view_model_context_artboard_source_for_data_bind(
                    context,
                    data_bind_index,
                    *value,
                ),
            RuntimeListenerViewModelChangeValue::Boolean(value) => self
                .data_bind_graph
                .set_owned_view_model_context_boolean_source_for_data_bind(
                    context,
                    data_bind_index,
                    *value,
                ),
            RuntimeListenerViewModelChangeValue::Trigger(value) => {
                let Some(bindable_trigger) = targets
                    .bindable_triggers
                    .iter_mut()
                    .find(|trigger| trigger.has_data_bind_index(data_bind_index))
                else {
                    return false;
                };
                bindable_trigger.set_value(*value);
                if !self
                    .data_bind_graph
                    .fire_owned_view_model_context_trigger_source_for_data_bind(
                        context,
                        data_bind_index,
                        *value,
                    )
                {
                    return false;
                }
                true
            }
            RuntimeListenerViewModelChangeValue::List(value) => self
                .data_bind_graph
                .set_active_view_model_source_for_data_bind(
                    data_bind_index,
                    RuntimeDataBindGraphValue::List {
                        item_count: usize::try_from(*value).unwrap_or(usize::MAX),
                    },
                ),
            RuntimeListenerViewModelChangeValue::ViewModel(value) => self
                .data_bind_graph
                .set_active_view_model_source_for_data_bind(
                    data_bind_index,
                    RuntimeDataBindGraphValue::ViewModel(*value),
                ),
        }
    }
}

impl StateMachineInstance {
    pub(super) fn perform_listener_view_model_change(
        &mut self,
        data_bind_index: usize,
        value: &RuntimeListenerViewModelChangeValue,
        owned_context: Option<&mut RuntimeOwnedViewModelInstance>,
    ) -> bool {
        match value {
            RuntimeListenerViewModelChangeValue::Number(value) => match owned_context {
                Some(context) => self.set_owned_view_model_context_number_source_for_data_bind(
                    context,
                    data_bind_index,
                    *value,
                ),
                None => {
                    self.set_default_view_model_number_source_for_data_bind(data_bind_index, *value)
                }
            },
            RuntimeListenerViewModelChangeValue::Integer(value) => match owned_context {
                Some(context) => self
                    .set_owned_view_model_context_symbol_list_index_source_for_data_bind(
                        context,
                        data_bind_index,
                        *value,
                    ),
                None => self.set_default_view_model_symbol_list_index_source_for_data_bind(
                    data_bind_index,
                    *value,
                ),
            },
            RuntimeListenerViewModelChangeValue::Color(value) => match owned_context {
                Some(context) => self.set_owned_view_model_context_color_source_for_data_bind(
                    context,
                    data_bind_index,
                    *value,
                ),
                None => {
                    self.set_default_view_model_color_source_for_data_bind(data_bind_index, *value)
                }
            },
            RuntimeListenerViewModelChangeValue::String(value) => match owned_context {
                Some(context) => self.set_owned_view_model_context_string_source_for_data_bind(
                    context,
                    data_bind_index,
                    value,
                ),
                None => {
                    self.set_default_view_model_string_source_for_data_bind(data_bind_index, value)
                }
            },
            RuntimeListenerViewModelChangeValue::Enum(value) => match owned_context {
                Some(context) => self.set_owned_view_model_context_enum_source_for_data_bind(
                    context,
                    data_bind_index,
                    *value,
                ),
                None => {
                    self.set_default_view_model_enum_source_for_data_bind(data_bind_index, *value)
                }
            },
            RuntimeListenerViewModelChangeValue::Asset(value) => {
                let value = self
                    .listener_asset_value_for_data_bind(data_bind_index, value)
                    .clone();
                let font_value = value.font_data_bind_value();
                let blob_value = value.blob_data_bind_value();
                match (owned_context, font_value.as_ref(), blob_value.as_ref()) {
                    (Some(context), Some(font_value), _) => self
                        .set_owned_view_model_context_font_asset_source_for_data_bind(
                            context,
                            data_bind_index,
                            font_value,
                        ),
                    (Some(context), _, Some(blob_value)) => self
                        .set_owned_view_model_context_blob_asset_source_for_data_bind(
                            context,
                            data_bind_index,
                            blob_value,
                        ),
                    (Some(context), None, None) => self
                        .set_owned_view_model_context_asset_source_for_data_bind(
                            context,
                            data_bind_index,
                            value.asset_index(),
                        ),
                    (None, _, _) => self.set_default_view_model_asset_source_for_data_bind(
                        data_bind_index,
                        value.data_bind_asset_index(),
                    ),
                }
            }
            RuntimeListenerViewModelChangeValue::Artboard(value) => match owned_context {
                Some(context) => self.set_owned_view_model_context_artboard_source_for_data_bind(
                    context,
                    data_bind_index,
                    *value,
                ),
                None => self
                    .set_default_view_model_artboard_source_for_data_bind(data_bind_index, *value),
            },
            RuntimeListenerViewModelChangeValue::Trigger(value) => match owned_context {
                Some(context) => self.fire_owned_view_model_context_trigger_source_for_data_bind(
                    context,
                    data_bind_index,
                    *value,
                ),
                None => self.perform_listener_trigger_view_model_change(data_bind_index, *value),
            },
            RuntimeListenerViewModelChangeValue::Boolean(value) => match owned_context {
                Some(context) => self.set_owned_view_model_context_boolean_source_for_data_bind(
                    context,
                    data_bind_index,
                    *value,
                ),
                None => self
                    .set_default_view_model_boolean_source_for_data_bind(data_bind_index, *value),
            },
            RuntimeListenerViewModelChangeValue::List(value) => {
                let changed = self
                    .data_bind_graph
                    .set_active_view_model_source_for_data_bind(
                        data_bind_index,
                        RuntimeDataBindGraphValue::List {
                            item_count: usize::try_from(*value).unwrap_or(usize::MAX),
                        },
                    );
                self.needs_advance |= changed;
                changed
            }
            RuntimeListenerViewModelChangeValue::ViewModel(value) => {
                let changed = self
                    .data_bind_graph
                    .set_active_view_model_source_for_data_bind(
                        data_bind_index,
                        RuntimeDataBindGraphValue::ViewModel(*value),
                    );
                self.needs_advance |= changed;
                changed
            }
        }
    }

    fn perform_listener_trigger_view_model_change(
        &mut self,
        data_bind_index: usize,
        value: u64,
    ) -> bool {
        let Some(bindable_trigger) = self
            .bindable_triggers
            .iter_mut()
            .find(|bindable_trigger| bindable_trigger.has_data_bind_index(data_bind_index))
        else {
            return false;
        };

        // Mirrors src/animation/listener_viewmodel_change.cpp: listener
        // actions invalidate the target-to-source binding even when the
        // trigger target value itself did not change.
        bindable_trigger.set_value(value);
        if !self
            .data_bind_graph
            .mark_trigger_target_dirty_for_data_bind(data_bind_index)
        {
            return false;
        }
        let applied = self
            .data_bind_graph
            .apply_default_view_model_target_to_source_for_data_bind(
                data_bind_index,
                &RuntimeDataBindGraphTargetsMut {
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
            );
        match applied {
            Ok(true) => {}
            Ok(false) => return false,
            Err(error) => {
                self.script_error.get_or_insert(error);
                return true;
            }
        }
        self.needs_advance = true;
        true
    }
}
