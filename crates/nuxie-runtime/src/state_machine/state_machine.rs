use super::*;

pub(super) fn next_view_model_trigger_layer_id() -> u64 {
    static NEXT_ID: AtomicU64 = AtomicU64::new(1);
    NEXT_ID.fetch_add(1, Ordering::Relaxed)
}

#[derive(Debug, Clone)]
pub struct RuntimeStateMachine {
    pub global_id: u32,
    pub name: Option<Arc<str>>,
    pub(crate) default_view_model_index: Option<usize>,
    pub inputs: Arc<Vec<Option<RuntimeStateMachineInput>>>,
    pub(crate) listeners: Arc<Vec<RuntimeStateMachineListener>>,
    pub layers: Arc<Vec<RuntimeStateMachineLayer>>,
    pub(crate) bindable_numbers: Arc<Vec<RuntimeBindableNumber>>,
    pub(crate) bindable_integers: Arc<Vec<RuntimeBindableInteger>>,
    pub(crate) bindable_colors: Arc<Vec<RuntimeBindableColor>>,
    pub(crate) bindable_strings: Arc<Vec<RuntimeBindableString>>,
    pub(crate) bindable_enums: Arc<Vec<RuntimeBindableEnum>>,
    pub(crate) bindable_assets: Arc<Vec<RuntimeBindableAsset>>,
    pub(crate) bindable_artboards: Arc<Vec<RuntimeBindableArtboard>>,
    pub(crate) bindable_lists: Arc<Vec<RuntimeBindableList>>,
    pub(crate) bindable_triggers: Arc<Vec<RuntimeBindableTrigger>>,
    pub(crate) bindable_view_models: Arc<Vec<RuntimeBindableViewModel>>,
    pub(crate) bindable_booleans: Arc<Vec<RuntimeBindableBoolean>>,
    pub(crate) view_model_triggers: Arc<Vec<RuntimeViewModelTrigger>>,
    pub(crate) transition_duration_bindings: Arc<Vec<RuntimeTransitionDurationBinding>>,
    pub(crate) data_bind_templates: Arc<Vec<RuntimeStateMachineDataBindTemplate>>,
    /// Every source `StateMachine::scriptedObjects()` occurrence in import
    /// order, including listener actions and scripted transition conditions.
    pub(crate) scripted_objects: Vec<ScriptListenerActionDefinition>,
    pub(crate) scripted_object_bindings: Vec<RuntimeScriptedListenerActionBindingDefinition>,
    pub(crate) scripted_listener_actions: Vec<ScriptListenerActionDefinition>,
    /// Pinned C++ source-StateMachine-owned generated fields for every
    /// ListenerAction and StateMachineFireAction. All concrete SMIs retain
    /// handles into this one definition arena.
    pub(crate) action_owners: RuntimeActionCoreArena,
}

impl RuntimeStateMachine {
    /// Number of authored layer definition occurrences.
    pub fn layer_count(&self) -> usize {
        self.layers.len()
    }

    /// Authored-index layer lookup. Out-of-range indices return `None`.
    pub fn layer_at(&self, index: usize) -> Option<&RuntimeStateMachineLayer> {
        self.layers.get(index)
    }

    /// Exact, case-sensitive, first-authored layer lookup.
    pub fn layer_named(&self, name: &str) -> Option<&RuntimeStateMachineLayer> {
        self.layers
            .iter()
            .find(|layer| layer.name.as_deref() == Some(name))
    }

    /// Number of authored input slots, including unsupported/null slots.
    pub fn input_count(&self) -> usize {
        self.inputs.len()
    }

    /// Authored-index input lookup.
    ///
    /// Both an out-of-range index and an in-range null compatibility slot
    /// return `None`, matching the observable result of C++ `input(size_t)`.
    pub fn input_at(&self, index: usize) -> Option<&RuntimeStateMachineInput> {
        self.inputs.get(index).and_then(Option::as_ref)
    }

    /// Exact, case-sensitive, first-authored input lookup.
    ///
    /// Rust deliberately skips a retained null compatibility slot instead of
    /// reproducing C++'s undefined null dereference in `input(std::string)`.
    pub fn input_named(&self, name: &str) -> Option<&RuntimeStateMachineInput> {
        self.inputs.iter().find_map(|input| {
            input
                .as_ref()
                .filter(|input| input.name.as_deref() == Some(name))
        })
    }

    /// Number of authored listener slots, including inert unbuildable slots.
    pub fn listener_count(&self) -> usize {
        self.listeners.len()
    }

    /// Authored-index listener lookup used by instance construction.
    pub(crate) fn listener_at(&self, index: usize) -> Option<&RuntimeStateMachineListener> {
        self.listeners.get(index)
    }

    /// Number of authored StateMachine DataBind occurrences.
    pub fn data_bind_count(&self) -> usize {
        self.data_bind_templates.len()
    }

    /// Authored-index lookup over the typed immutable DataBind adaptation.
    pub(crate) fn data_bind_at(
        &self,
        index: usize,
    ) -> Option<&RuntimeStateMachineDataBindTemplate> {
        self.data_bind_templates.get(index)
    }

    /// Number of authored scripted-object occurrences.
    pub fn scripted_object_count(&self) -> usize {
        self.scripted_objects.len()
    }

    /// Complete state-machine `ScriptedObject` collection in imported order.
    ///
    /// The immutable slice is the Rust equivalent of C++ returning a copied
    /// pointer vector: callers can copy or clear their own `Vec` without
    /// changing this definition owner.
    #[doc(hidden)]
    pub fn scripted_objects(&self) -> &[ScriptListenerActionDefinition] {
        &self.scripted_objects
    }

    /// Scripted listener tables that must be instantiated for each concrete
    /// [`StateMachineInstance`] occurrence.
    pub fn scripted_listener_actions(&self) -> &[ScriptListenerActionDefinition] {
        &self.scripted_listener_actions
    }

    /// Existing-import-architecture adaptation of
    /// `StateMachine::onAddedDirty`.
    ///
    /// Child owners supply their phase operation. This owner fixes the C++
    /// traversal boundary: inputs, then layers, then listeners, returning the
    /// first failure without rolling back successful earlier callbacks.
    #[allow(dead_code)]
    pub(crate) fn on_added_dirty<E>(
        &self,
        on_input: impl FnMut(usize, Option<&RuntimeStateMachineInput>) -> Result<(), E>,
        on_layer: impl FnMut(usize, &RuntimeStateMachineLayer) -> Result<(), E>,
        on_listener: impl FnMut(usize, &RuntimeStateMachineListener) -> Result<(), E>,
    ) -> Result<(), E> {
        self.visit_added_children(on_input, on_layer, on_listener)
    }

    /// Existing-import-architecture adaptation of
    /// `StateMachine::onAddedClean`; ordering and first-error behavior are
    /// intentionally identical to the dirty phase.
    #[allow(dead_code)]
    pub(crate) fn on_added_clean<E>(
        &self,
        on_input: impl FnMut(usize, Option<&RuntimeStateMachineInput>) -> Result<(), E>,
        on_layer: impl FnMut(usize, &RuntimeStateMachineLayer) -> Result<(), E>,
        on_listener: impl FnMut(usize, &RuntimeStateMachineListener) -> Result<(), E>,
    ) -> Result<(), E> {
        self.visit_added_children(on_input, on_layer, on_listener)
    }

    fn visit_added_children<E>(
        &self,
        mut on_input: impl FnMut(usize, Option<&RuntimeStateMachineInput>) -> Result<(), E>,
        mut on_layer: impl FnMut(usize, &RuntimeStateMachineLayer) -> Result<(), E>,
        mut on_listener: impl FnMut(usize, &RuntimeStateMachineListener) -> Result<(), E>,
    ) -> Result<(), E> {
        for (index, input) in self.inputs.iter().enumerate() {
            on_input(index, input.as_ref())?;
        }
        for (index, layer) in self.layers.iter().enumerate() {
            on_layer(index, layer)?;
        }
        for (index, listener) in self.listeners.iter().enumerate() {
            on_listener(index, listener)?;
        }
        Ok(())
    }
}

/// Retain one inert Rust slot for a C++ listener definition that the current
/// Rust listener adaptation cannot build.
///
/// Generated C++ initializes an absent `targetId` to `u32::MAX`; malformed
/// target/input/action relationships can still make the current Rust lowering
/// return `None`. The source listener owner nevertheless occupies its authored
/// vector index, so dropping it here would renumber every later listener.
fn inert_state_machine_listener(
    listener: &nuxie_binary::RuntimeStateMachineListener<'_>,
) -> RuntimeStateMachineListener {
    RuntimeStateMachineListener {
        name: listener
            .object
            .string_property("name")
            .map(ToOwned::to_owned),
        target_local_id: listener
            .object
            .uint_property("targetId")
            .and_then(|target| usize::try_from(target).ok())
            .unwrap_or(usize::MAX),
        is_single: listener.object.type_name == "StateMachineListenerSingle",
        listener_types: Vec::new(),
        event_local_indices: Vec::new(),
        view_model_path: None,
        view_model_input_types: Vec::new(),
        gamepad_input_types: Vec::new(),
        keyboard_input_types: Vec::new(),
        semantic_input_types: Vec::new(),
        hit_paths: Vec::new(),
        listener_actions: Vec::new(),
    }
}

fn retain_state_machine_listener_slot(
    listener: &nuxie_binary::RuntimeStateMachineListener<'_>,
    built: Option<RuntimeStateMachineListener>,
) -> RuntimeStateMachineListener {
    built.unwrap_or_else(|| inert_state_machine_listener(listener))
}

pub(crate) fn build_state_machines<'a>(
    file: &'a RuntimeFile,
    graph: &ArtboardGraph,
    linear_animations: &[RuntimeLinearAnimation],
    converter_cache: &mut RuntimeDataBindGraphConverterBuildCache<'a>,
) -> Vec<RuntimeStateMachine> {
    let action_catalog = RuntimeFileStateMachineActionCatalog::new(file);
    build_state_machines_with_action_catalog(
        file,
        graph,
        linear_animations,
        converter_cache,
        &action_catalog,
    )
}

pub(crate) fn build_state_machines_with_action_catalog<'a>(
    file: &'a RuntimeFile,
    graph: &ArtboardGraph,
    linear_animations: &[RuntimeLinearAnimation],
    converter_cache: &mut RuntimeDataBindGraphConverterBuildCache<'a>,
    action_catalog: &RuntimeFileStateMachineActionCatalog,
) -> Vec<RuntimeStateMachine> {
    let Some(artboard_index) = artboard_index_for_graph(file, graph) else {
        return Vec::new();
    };
    let animation_index_by_global = linear_animations
        .iter()
        .enumerate()
        .map(|(index, animation)| (animation.global_id, index))
        .collect::<BTreeMap<_, _>>();
    let default_view_model_index = state_machine_default_view_model_index(file, artboard_index);
    let default_instance = default_view_model_index
        .and_then(|view_model_index| file.view_model_default_instance(view_model_index))
        .map(|instance| instance.object);

    file.artboard_state_machine_graphs(artboard_index)
        .into_iter()
        .map(|state_machine| {
            let action_owners = action_catalog
                .arena(state_machine.object.id)
                .expect("file action catalog must contain every accepted state machine");
            let state_machine_data_binds = state_machine.data_binds.clone();
            let bindable_numbers = runtime_bindable_numbers(
                file,
                &state_machine,
                default_instance,
                converter_cache,
            );
            let bindable_integers = runtime_bindable_integers(
                file,
                &state_machine,
                default_instance,
                converter_cache,
            );
            let bindable_colors = runtime_bindable_colors(file, &state_machine, default_instance);
            let bindable_strings = runtime_bindable_strings(
                file,
                &state_machine,
                default_instance,
                converter_cache,
            );
            let bindable_enums = runtime_bindable_enums(file, &state_machine, default_instance);
            let bindable_assets = runtime_bindable_assets(file, &state_machine, default_instance);
            let bindable_artboards =
                runtime_bindable_artboards(file, &state_machine, default_instance);
            let bindable_lists = runtime_bindable_lists(
                file,
                &state_machine,
                default_instance,
                converter_cache,
            );
            let bindable_triggers = runtime_bindable_triggers(
                file,
                &state_machine,
                default_instance,
                converter_cache,
            );
            let bindable_view_models = runtime_bindable_view_models(
                file,
                &state_machine,
                default_instance,
                converter_cache,
            );
            let bindable_booleans = runtime_bindable_booleans(
                file,
                &state_machine,
                default_instance,
                converter_cache,
            );
            let view_model_triggers =
                runtime_default_view_model_triggers(file, default_view_model_index);
            let transition_duration_bindings =
                runtime_transition_duration_bindings(file, &state_machine, default_instance);
            let data_bind_templates = runtime_state_machine_data_bind_templates(
                file,
                &state_machine,
                default_instance,
                &transition_duration_bindings,
                converter_cache,
            );
            let scripted_listener_actions = state_machine
                .scripted_objects
                .iter()
                .map(|scripted| {
                    (
                        runtime_scripted_object_definition(
                            file,
                            scripted.object,
                            &scripted.inputs,
                        )
                        .expect(
                            "binary StateMachine scripted-object inventory contains supported kinds",
                        ),
                        runtime_scripted_object_binding_definition(
                            file,
                            scripted.object,
                            &scripted.inputs,
                        )
                        .expect(
                            "every supported StateMachine scripted object has a binding recipe",
                        ),
                    )
                })
                .collect::<Vec<_>>();
            let scripted_object_bindings = scripted_listener_actions
                .iter()
                .map(|(_, binding)| binding.clone())
                .collect();
            let scripted_objects = scripted_listener_actions
                .into_iter()
                .map(|(definition, _)| definition)
                .collect::<Vec<_>>();
            let scripted_listener_actions = scripted_objects
                .iter()
                .filter(|definition| {
                    definition.scripted_object_kind()
                        == crate::ScriptedStateMachineObjectKind::ListenerAction
                })
                .cloned()
                .collect();
            RuntimeStateMachine {
                global_id: state_machine.object.id,
                name: state_machine
                    .object
                    .string_property("name")
                    .map(Arc::<str>::from),
                default_view_model_index,
                inputs: Arc::new(
                    state_machine
                        .inputs
                        .iter()
                        .map(|input| input.and_then(runtime_state_machine_input))
                        .collect(),
                ),
                listeners: Arc::new(
                    state_machine
                        .listeners
                        .iter()
                        .map(|listener| {
                            retain_state_machine_listener_slot(
                                listener,
                                runtime_state_machine_listener(
                                    file,
                                    graph,
                                    &state_machine.inputs,
                                    &state_machine_data_binds,
                                    listener,
                                    &action_owners,
                                ),
                            )
                        })
                        .collect(),
                ),
                bindable_numbers: Arc::new(bindable_numbers),
                bindable_integers: Arc::new(bindable_integers),
                bindable_colors: Arc::new(bindable_colors),
                bindable_strings: Arc::new(bindable_strings),
                bindable_enums: Arc::new(bindable_enums),
                bindable_assets: Arc::new(bindable_assets),
                bindable_artboards: Arc::new(bindable_artboards),
                bindable_lists: Arc::new(bindable_lists),
                bindable_triggers: Arc::new(bindable_triggers),
                bindable_view_models: Arc::new(bindable_view_models),
                bindable_booleans: Arc::new(bindable_booleans),
                view_model_triggers: Arc::new(view_model_triggers),
                transition_duration_bindings: Arc::new(transition_duration_bindings),
                data_bind_templates: Arc::new(data_bind_templates),
                scripted_objects,
                scripted_object_bindings,
                scripted_listener_actions,
                action_owners: action_owners.clone(),
                layers: Arc::new(
                    state_machine
                    .layers
                    .into_iter()
                    .map(|layer| {
                        let states = layer
                            .states
                            .into_iter()
                            .map(|state| {
                                let animation = state
                                    .object
                                    .filter(|object| object.type_name == "AnimationState")
                                    .map(|_| {
                                        state
                                            .animation
                                            .and_then(|animation| {
                                                animation_index_by_global
                                                    .get(&animation.id)
                                                    .copied()
                                            })
                                            .map(RuntimeLinearAnimationHandle::new)
                                            .unwrap_or_else(RuntimeLinearAnimationHandle::empty)
                                    });
                                let blend_state_1d = RuntimeBlendState1D::from_imported(
                                    file,
                                    &state,
                                    &animation_index_by_global,
                                );
                                let blend_state_direct = RuntimeBlendStateDirect::from_imported(
                                    file,
                                    &state,
                                    &animation_index_by_global,
                                );
                                RuntimeLayerState {
                                    global_id: state.object.map(|object| object.id),
                                    // C++ retains a concrete LayerState for
                                    // the base/no-op state and reports its
                                    // core type. The importer represents that
                                    // occurrence with no concrete object.
                                    type_name: Some(
                                        state
                                            .object
                                            .map_or("LayerState", |object| object.type_name),
                                    ),
                                    animation,
                                    blend_state_1d,
                                    blend_state_direct,
                                    speed: state
                                        .object
                                        .and_then(|object| object.double_property("speed"))
                                        .unwrap_or(1.0),
                                    flags: state
                                        .object
                                        .and_then(|object| object.uint_property("flags"))
                                        .unwrap_or(0),
                                    fire_actions: state
                                        .fire_actions
                                        .iter()
                                        .map(|action| {
                                            RuntimeStateMachineFireAction::from_imported(
                                                file,
                                                action,
                                                action_owners
                                                    .handle(action.object.id)
                                                    .expect("accepted fire action has an owner"),
                                            )
                                        })
                                        .collect(),
                                    listener_actions: state
                                        .listener_actions
                                        .iter()
                                        .map(|action| {
                                            RuntimeScheduledListenerAction::from_imported(
                                                file,
                                                graph,
                                                &state_machine.inputs,
                                                &state_machine_data_binds,
                                                action,
                                                action_owners
                                                    .handle(action.object.id)
                                                    .expect(
                                                        "accepted listener action has an owner",
                                                    ),
                                            )
                                        })
                                        .collect(),
                                    transitions: state
                                        .transitions
                                        .into_iter()
                                        .map(|transition| {
                                            let interpolator = transition.interpolator.and_then(
                                                RuntimeTransitionInterpolator::from_object,
                                            );
                                            let conditions = transition
                                                .conditions
                                                .iter()
                                                .filter_map(|condition| {
                                                    RuntimeTransitionCondition::from_object(
                                                        file,
                                                        graph,
                                                        &state_machine.inputs,
                                                        condition,
                                                    )
                                                })
                                                .collect::<Vec<_>>();
                                            let direct_input_conditions_only = conditions
                                                .iter()
                                                .all(RuntimeTransitionCondition::is_direct_input);
                                            RuntimeStateTransition {
                                                global_id: transition.object.id,
                                                state_to_index: transition.state_to_index,
                                                exit_blend_animation_index: transition
                                                    .exit_blend_animation_index,
                                                duration: transition
                                                    .object
                                                    .uint_property("duration")
                                                    .unwrap_or(0),
                                                exit_time: transition
                                                    .object
                                                    .uint_property("exitTime")
                                                    .unwrap_or(0),
                                                flags: transition
                                                    .object
                                                    .uint_property("flags")
                                                    .unwrap_or(0),
                                                random_weight: transition
                                                    .object
                                                    .uint_property("randomWeight")
                                                    .unwrap_or(1)
                                                    as u32,
                                                conditions,
                                                direct_input_conditions_only,
                                                fire_actions: transition
                                                    .fire_actions
                                                    .iter()
                                                    .map(|action| {
                                                        RuntimeStateMachineFireAction::from_imported(
                                                            file,
                                                            action,
                                                            action_owners
                                                                .handle(action.object.id)
                                                                .expect(
                                                                    "accepted fire action has an owner",
                                                                ),
                                                        )
                                                    })
                                                    .collect(),
                                                listener_actions: transition
                                                    .listener_actions
                                                    .iter()
                                                    .map(|action| {
                                                        RuntimeScheduledListenerAction::from_imported(
                                                            file,
                                                            graph,
                                                            &state_machine.inputs,
                                                            &state_machine_data_binds,
                                                            action,
                                                            action_owners
                                                                .handle(action.object.id)
                                                                .expect(
                                                                    "accepted listener action has an owner",
                                                                ),
                                                        )
                                                    })
                                                    .collect(),
                                                interpolator,
                                                has_unsupported_interpolator: transition
                                                    .interpolator
                                                    .is_some()
                                                    && interpolator.is_none(),
                                            }
                                        })
                                        .collect(),
                                }
                            })
                            .collect::<Vec<_>>();
                        let (entry_state_index, any_state_index, exit_state_index) =
                            RuntimeStateMachineLayer::resolve_system_state_indices(&states);
                        RuntimeStateMachineLayer {
                            global_id: layer.object.id,
                            name: layer.object.string_property("name").map(ToOwned::to_owned),
                            states,
                            entry_state_index,
                            any_state_index,
                            exit_state_index,
                        }
                    })
                    .collect(),
                ),
            }
        })
        .collect()
}

fn state_machine_default_view_model_index(
    file: &RuntimeFile,
    artboard_index: usize,
) -> Option<usize> {
    file.resolved_view_model_for_artboard(artboard_index)
        .map(|view_model| view_model.view_model_index)
        .or_else(|| file.view_model(0).map(|_| 0))
}

pub(crate) struct TransitionEvaluationContext<'a> {
    pub(super) bindable_numbers: &'a [StateMachineBindableNumberInstance],
    pub(super) bindable_integers: &'a [StateMachineBindableIntegerInstance],
    pub(super) bindable_colors: &'a [StateMachineBindableColorInstance],
    pub(super) bindable_strings: &'a [StateMachineBindableStringInstance],
    pub(super) bindable_enums: &'a [StateMachineBindableEnumInstance],
    pub(super) bindable_assets: &'a [StateMachineBindableAssetInstance],
    pub(super) bindable_artboards: &'a [StateMachineBindableArtboardInstance],
    pub(super) bindable_triggers: &'a [StateMachineBindableTriggerInstance],
    pub(super) bindable_view_models: &'a [StateMachineBindableViewModelInstance],
    pub(super) bindable_booleans: &'a [StateMachineBindableBooleanInstance],
    pub(super) data_context_present: bool,
    pub(super) layer_index: usize,
    pub(super) view_model_trigger_layer_id: u64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum RuntimeTransitionInterpolator {
    CubicEase {
        x1: f32,
        y1: f32,
        x2: f32,
        y2: f32,
    },
    Elastic {
        amplitude: f32,
        period: f32,
        easing_value: u64,
    },
}

impl RuntimeTransitionInterpolator {
    pub(crate) fn from_object(object: &RuntimeObject) -> Option<Self> {
        match object.type_name {
            "CubicEaseInterpolator" => Some(Self::CubicEase {
                x1: object.double_property("x1").unwrap_or(0.42),
                y1: object.double_property("y1").unwrap_or(0.0),
                x2: object.double_property("x2").unwrap_or(0.58),
                y2: object.double_property("y2").unwrap_or(1.0),
            }),
            "ElasticInterpolator" => Some(Self::Elastic {
                amplitude: object.double_property("amplitude").unwrap_or(1.0),
                period: object.double_property("period").unwrap_or(1.0),
                easing_value: object.uint_property("easingValue").unwrap_or(1),
            }),
            _ => None,
        }
    }

    pub(crate) fn transform(self, factor: f32) -> f32 {
        match self {
            Self::CubicEase { x1, y1, x2, y2 } => {
                RuntimeInterpolator::CubicEase { x1, y1, x2, y2 }.transform(factor)
            }
            Self::Elastic {
                amplitude,
                period,
                easing_value,
            } => RuntimeInterpolator::Elastic {
                amplitude,
                period,
                easing_value,
            }
            .transform(factor),
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct RuntimeBlendState1D {
    pub(crate) source: RuntimeBlendState1DSource,
    pub(crate) animations: Vec<RuntimeBlendAnimation1D>,
}

impl RuntimeBlendState1D {
    pub(crate) fn from_imported(
        file: &RuntimeFile,
        state: &nuxie_binary::RuntimeLayerState<'_>,
        animation_index_by_global: &BTreeMap<u32, usize>,
    ) -> Option<Self> {
        let object = state.object?;
        let source = match object.type_name {
            "BlendState1DInput" => RuntimeBlendState1DSource::Input {
                input_index: object
                    .uint_property("inputId")
                    .filter(|input_id| *input_id != u64::from(u32::MAX))
                    .and_then(|input_id| usize::try_from(input_id).ok()),
            },
            "BlendState1DViewModel" => RuntimeBlendState1DSource::BindableProperty {
                global_id: file
                    .latest_bindable_property_for_object(object)
                    .map(|property| property.id as u32),
            },
            _ => return None,
        };
        let animations = state
            .blend_animations
            .iter()
            .filter_map(|animation| {
                if animation.object.type_name != "BlendAnimation1D" {
                    return None;
                }
                let definition = animation
                    .animation
                    .and_then(|animation| animation_index_by_global.get(&animation.id).copied())
                    .map(RuntimeLinearAnimationHandle::new)
                    .unwrap_or_else(RuntimeLinearAnimationHandle::empty);
                Some(RuntimeBlendAnimation1D {
                    animation: definition,
                    value: animation.object.double_property("value").unwrap_or(0.0),
                })
            })
            .collect::<Vec<_>>();
        Some(Self { source, animations })
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum RuntimeBlendState1DSource {
    Input { input_index: Option<usize> },
    BindableProperty { global_id: Option<u32> },
}

#[derive(Debug, Clone)]
pub(crate) struct RuntimeBlendAnimation1D {
    pub(crate) animation: RuntimeLinearAnimationHandle,
    pub(crate) value: f32,
}

#[derive(Debug, Clone)]
pub(crate) struct RuntimeBlendStateDirect {
    pub(crate) animations: Vec<RuntimeBlendAnimationDirect>,
}

impl RuntimeBlendStateDirect {
    pub(crate) fn from_imported(
        file: &RuntimeFile,
        state: &nuxie_binary::RuntimeLayerState<'_>,
        animation_index_by_global: &BTreeMap<u32, usize>,
    ) -> Option<Self> {
        let object = state.object?;
        if object.type_name != "BlendStateDirect" {
            return None;
        }
        let animations = state
            .blend_animations
            .iter()
            .filter_map(|animation| {
                if animation.object.type_name != "BlendAnimationDirect" {
                    return None;
                }
                let definition = animation
                    .animation
                    .and_then(|animation| animation_index_by_global.get(&animation.id).copied())
                    .map(RuntimeLinearAnimationHandle::new)
                    .unwrap_or_else(RuntimeLinearAnimationHandle::empty);
                Some(RuntimeBlendAnimationDirect {
                    animation: definition,
                    source: RuntimeDirectBlendSource::from_object(file, animation.object),
                })
            })
            .collect::<Vec<_>>();
        Some(Self { animations })
    }
}

#[derive(Debug, Clone)]
pub(crate) struct RuntimeBlendAnimationDirect {
    pub(crate) animation: RuntimeLinearAnimationHandle,
    pub(crate) source: RuntimeDirectBlendSource,
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum RuntimeDirectBlendSource {
    Input { input_index: usize },
    MixValue { value: f32 },
    BindableProperty { global_id: Option<u32> },
}

impl RuntimeDirectBlendSource {
    fn from_object(file: &RuntimeFile, object: &RuntimeObject) -> Self {
        match object.uint_property("blendSource").unwrap_or(0) {
            1 => Self::MixValue {
                value: object.double_property("mixValue").unwrap_or(100.0),
            },
            2 => Self::BindableProperty {
                global_id: file
                    .latest_bindable_property_for_object(object)
                    .map(|property| property.id as u32),
            },
            _ => Self::Input {
                input_index: object
                    .uint_property("inputId")
                    .and_then(|input_id| usize::try_from(input_id).ok())
                    .unwrap_or(usize::MAX),
            },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct RuntimeBlendAnimationHandle(usize);

impl RuntimeBlendAnimationHandle {
    pub(super) fn new(index: usize) -> Self {
        Self(index)
    }

    pub(super) fn index(self) -> usize {
        self.0
    }
}

#[derive(Debug, Clone)]
pub(crate) struct BlendState1DInstance {
    animations: Vec<BlendAnimation1DInstance>,
    from: Option<RuntimeBlendAnimationHandle>,
    to: Option<RuntimeBlendAnimationHandle>,
    animation_reset: Option<AnimationReset>,
}

impl BlendState1DInstance {
    pub(crate) fn new(
        blend_state: &RuntimeBlendState1D,
        artboard: &ArtboardInstance,
        animation_definitions: &Arc<Vec<RuntimeLinearAnimation>>,
        empty_animation_definition: &Arc<RuntimeLinearAnimation>,
        reset_blend_values: bool,
    ) -> Self {
        let animations: Vec<BlendAnimation1DInstance> = blend_state
            .animations
            .iter()
            .enumerate()
            .filter_map(|(definition_index, animation)| {
                Some(BlendAnimation1DInstance {
                    definition: RuntimeBlendAnimationHandle::new(definition_index),
                    animation: LinearAnimationInstance::new(
                        animation.animation,
                        Arc::clone(animation_definitions),
                        Arc::clone(empty_animation_definition),
                        1.0,
                    )?,
                    mix: 0.0,
                })
            })
            .collect();
        let animation_reset = if reset_blend_values {
            Some(AnimationResetFactory::from_animation_instances(
                artboard,
                animations.iter().map(|animation| &animation.animation),
                true,
            ))
        } else {
            None
        };

        Self {
            animations,
            from: None,
            to: None,
            animation_reset,
        }
    }

    pub(crate) fn advance(
        &mut self,
        blend_state: &RuntimeBlendState1D,
        artboard: &ArtboardInstance,
        inputs: &[StateMachineInputInstance],
        bindable_numbers: &[StateMachineBindableNumberInstance],
        elapsed_seconds: f32,
    ) -> bool {
        for animation in &mut self.animations {
            if artboard.linear_animation_instance_keep_going(&animation.animation) {
                artboard
                    .advance_linear_animation_instance(&mut animation.animation, elapsed_seconds);
            }
        }

        self.update_mix_values(blend_state, inputs, bindable_numbers);
        true
    }

    pub(crate) fn advance_with_events(
        &mut self,
        blend_state: &RuntimeBlendState1D,
        artboard: &mut ArtboardInstance,
        inputs: &[StateMachineInputInstance],
        bindable_numbers: &[StateMachineBindableNumberInstance],
        elapsed_seconds: f32,
        reported_events: &mut Vec<StateMachineReportedEvent>,
    ) -> bool {
        self.advance_and_report(
            artboard,
            blend_state,
            inputs,
            bindable_numbers,
            elapsed_seconds,
            Some(reported_events),
        )
    }

    fn advance_and_report(
        &mut self,
        artboard: &mut ArtboardInstance,
        blend_state: &RuntimeBlendState1D,
        inputs: &[StateMachineInputInstance],
        bindable_numbers: &[StateMachineBindableNumberInstance],
        elapsed_seconds: f32,
        mut reported_events: Option<&mut Vec<StateMachineReportedEvent>>,
    ) -> bool {
        for animation in &mut self.animations {
            if artboard.linear_animation_instance_keep_going(&animation.animation) {
                if let Some(events) = reported_events.as_mut() {
                    artboard.advance_linear_animation_instance_with_events(
                        &mut animation.animation,
                        elapsed_seconds,
                        *events,
                    );
                } else {
                    artboard.advance_linear_animation_instance(
                        &mut animation.animation,
                        elapsed_seconds,
                    );
                }
            }
        }

        self.update_mix_values(blend_state, inputs, bindable_numbers);
        true
    }

    fn update_mix_values(
        &mut self,
        blend_state: &RuntimeBlendState1D,
        inputs: &[StateMachineInputInstance],
        bindable_numbers: &[StateMachineBindableNumberInstance],
    ) {
        if self.animations.is_empty() {
            return;
        }

        let value = match blend_state.source {
            RuntimeBlendState1DSource::Input { input_index } => input_index
                .and_then(|input_index| inputs.get(input_index))
                .and_then(StateMachineInputInstance::number_value)
                .unwrap_or(0.0),
            RuntimeBlendState1DSource::BindableProperty { global_id } => global_id
                .and_then(|global_id| bindable_number_value(bindable_numbers, global_id))
                .unwrap_or(0.0),
        };

        let to_index = self.animation_index(blend_state, value);
        self.to = (to_index < self.animations.len()).then(|| self.animations[to_index].definition);
        self.from = to_index
            .checked_sub(1)
            .and_then(|index| self.animations.get(index))
            .map(|animation| animation.definition);
        let to_value = self
            .to
            .and_then(|handle| blend_state.animations.get(handle.index()))
            .map(|animation| animation.value)
            .unwrap_or(0.0);
        let from_value = self
            .from
            .and_then(|handle| blend_state.animations.get(handle.index()))
            .map(|animation| animation.value)
            .unwrap_or(0.0);
        let (mix, mix_from) = if self.to.is_none() || self.from.is_none() || to_value == from_value
        {
            (1.0, 1.0)
        } else {
            let mix = (value - from_value) / (to_value - from_value);
            (mix, 1.0 - mix)
        };

        for animation in &mut self.animations {
            let animation_value = blend_state
                .animations
                .get(animation.definition.index())
                .map(|definition| definition.value)
                .unwrap_or(0.0);
            if self.to.is_some() && animation_value == to_value {
                animation.mix = mix;
            } else if self.from.is_some() && animation_value == from_value {
                animation.mix = mix_from;
            } else {
                animation.mix = 0.0;
            }
        }
    }

    fn animation_index(&self, blend_state: &RuntimeBlendState1D, value: f32) -> usize {
        let mut index = 0_usize;
        let mut start = 0_isize;
        let mut end = self.animations.len() as isize - 1;

        while start <= end {
            let mid = (start + end) >> 1;
            let closest_value = self
                .animations
                .get(mid as usize)
                .and_then(|animation| blend_state.animations.get(animation.definition.index()))
                .map(|animation| animation.value)
                .unwrap_or(0.0);
            if closest_value < value {
                start = mid + 1;
            } else if closest_value > value {
                end = mid - 1;
            } else {
                index = mid as usize;
                break;
            }

            index = start as usize;
        }

        index
    }

    pub(crate) fn animation_instance(&self, index: usize) -> Option<&LinearAnimationInstance> {
        self.animations
            .iter()
            .find(|animation| animation.definition.index() == index)
            .map(|animation| &animation.animation)
    }

    pub(super) fn for_each_animation_instance_mut(
        &mut self,
        mut callback: impl FnMut(&mut LinearAnimationInstance),
    ) {
        for animation in &mut self.animations {
            callback(&mut animation.animation);
        }
    }

    pub(crate) fn apply(&mut self, artboard: &mut ArtboardInstance, mix: f32) -> bool {
        let mut changed = false;
        if let Some(reset) = self.animation_reset.as_ref() {
            changed |= reset.apply(artboard);
        }
        for animation in &self.animations {
            let animation_mix = mix * animation.mix;
            if animation_mix == 0.0 {
                continue;
            }
            changed |= animation.animation.apply(artboard, animation_mix);
        }
        changed
    }
}

#[derive(Debug, Clone)]
struct BlendAnimation1DInstance {
    definition: RuntimeBlendAnimationHandle,
    animation: LinearAnimationInstance,
    mix: f32,
}

#[cfg(test)]
mod animation_tests {
    use super::*;
    use crate::view_model::RuntimeFontAssetValue;

    #[test]
    fn listener_asset_clone_retains_live_font_payload() {
        let live: Arc<[u8]> = vec![1, 3, 5, 7].into();
        let mut font = RuntimeFontAssetValue::default();
        assert!(font.set_live_font_bytes(Some(Arc::clone(&live))));
        let action = RuntimeScheduledListenerAction::ViewModelChange(
            listener_viewmodel_change::RuntimeListenerViewModelChange::for_test(
                0,
                Some(4),
                Some(RuntimeListenerViewModelChangeValue::Asset(
                    RuntimeBindableAssetValue::from_font_value(font),
                )),
            ),
        );

        let RuntimeScheduledListenerAction::ViewModelChange(
            listener_viewmodel_change::RuntimeListenerViewModelChange {
                value: Some(RuntimeListenerViewModelChangeValue::Asset(value)),
                ..
            },
        ) = action.clone()
        else {
            panic!("listener action lost its asset value");
        };
        assert_eq!(
            value.asset_index(),
            RuntimeFontAssetValue::MISSING_FILE_ASSET_INDEX
        );
        assert!(
            value
                .font_value()
                .and_then(RuntimeFontAssetValue::live_font_bytes_arc)
                .is_some_and(|value| Arc::ptr_eq(value, &live)),
            "cloning a scheduled listener must retain the same live font"
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data_bind_graph::{RuntimeDataBindGraphTarget, RuntimeDataBindGraphValue};
    use crate::data_converter::RuntimeDataConverterDataBindDefinition;
    use crate::properties::property_key_for_name;
    use nuxie_binary::{
        FixtureProperty, FixtureRecord, FixtureValue, RuntimeFile, read_runtime_file,
    };
    use nuxie_graph::GraphFile;
    use std::cell::RefCell;
    use std::path::PathBuf;

    fn rive_runtime_fixture(name: &str) -> PathBuf {
        PathBuf::from(
            std::env::var_os("RIVE_RUNTIME_DIR")
                .unwrap_or_else(|| "/Users/levi/dev/oss/rive-runtime".into()),
        )
        .join("tests/unit_tests/assets")
        .join(name)
    }

    fn fl_c5_empty_listener(target_local_id: usize) -> RuntimeStateMachineListener {
        RuntimeStateMachineListener {
            name: None,
            target_local_id,
            is_single: false,
            listener_types: Vec::new(),
            event_local_indices: Vec::new(),
            view_model_path: None,
            view_model_input_types: Vec::new(),
            gamepad_input_types: Vec::new(),
            keyboard_input_types: Vec::new(),
            semantic_input_types: Vec::new(),
            hit_paths: Vec::new(),
            listener_actions: Vec::new(),
        }
    }

    fn fl_c5_definition_machine(
        inputs: Vec<Option<RuntimeStateMachineInput>>,
        listeners: Vec<RuntimeStateMachineListener>,
        layers: Vec<RuntimeStateMachineLayer>,
        scripted_objects: Vec<ScriptListenerActionDefinition>,
    ) -> RuntimeStateMachine {
        RuntimeStateMachine {
            global_id: 1,
            name: Some(Arc::from("definition")),
            default_view_model_index: None,
            inputs: Arc::new(inputs),
            listeners: Arc::new(listeners),
            layers: Arc::new(layers),
            bindable_numbers: Arc::new(Vec::new()),
            bindable_integers: Arc::new(Vec::new()),
            bindable_colors: Arc::new(Vec::new()),
            bindable_strings: Arc::new(Vec::new()),
            bindable_enums: Arc::new(Vec::new()),
            bindable_assets: Arc::new(Vec::new()),
            bindable_artboards: Arc::new(Vec::new()),
            bindable_lists: Arc::new(Vec::new()),
            bindable_triggers: Arc::new(Vec::new()),
            bindable_view_models: Arc::new(Vec::new()),
            bindable_booleans: Arc::new(Vec::new()),
            view_model_triggers: Arc::new(Vec::new()),
            transition_duration_bindings: Arc::new(Vec::new()),
            data_bind_templates: Arc::new(Vec::new()),
            scripted_object_bindings: Vec::new(),
            scripted_listener_actions: scripted_objects
                .iter()
                .filter(|definition| {
                    definition.scripted_object_kind()
                        == crate::ScriptedStateMachineObjectKind::ListenerAction
                })
                .cloned()
                .collect(),
            scripted_objects,
            action_owners: RuntimeActionCoreArena::empty(),
        }
    }

    fn fl_c5_layer(global_id: u32, name: &str) -> RuntimeStateMachineLayer {
        RuntimeStateMachineLayer {
            global_id,
            name: Some(name.to_owned()),
            states: Vec::new(),
            entry_state_index: None,
            any_state_index: None,
            exit_state_index: None,
        }
    }

    #[test]
    fn fl_c5_definition_empty_machine_count_and_index_views() {
        let machine = fl_c5_definition_machine(Vec::new(), Vec::new(), Vec::new(), Vec::new());

        assert_eq!(machine.layer_count(), 0);
        assert_eq!(machine.input_count(), 0);
        assert_eq!(machine.listener_count(), 0);
        assert_eq!(machine.data_bind_count(), 0);
        assert_eq!(machine.scripted_object_count(), 0);
        assert!(machine.layer_at(0).is_none());
        assert!(machine.layer_at(usize::MAX).is_none());
        assert!(machine.input_at(0).is_none());
        assert!(machine.input_at(usize::MAX).is_none());
        assert!(machine.listener_at(0).is_none());
        assert!(machine.listener_at(usize::MAX).is_none());
        assert!(machine.data_bind_at(0).is_none());
        assert!(machine.data_bind_at(usize::MAX).is_none());
        assert!(machine.input_named("missing").is_none());
        assert!(machine.layer_named("missing").is_none());
    }

    #[test]
    fn fl_c5_definition_authored_order_duplicates_names_and_null_slots() {
        let scripted = ScriptListenerActionDefinition::with_inputs_and_kind(
            70,
            crate::ScriptedStateMachineObjectKind::ListenerAction,
            usize::MAX,
            String::new(),
            false,
            0,
            Vec::new(),
        );
        let mut machine = fl_c5_definition_machine(
            vec![
                None,
                Some(RuntimeStateMachineInput::new_bool(
                    10,
                    Some("duplicate".to_owned()),
                    false,
                )),
                Some(RuntimeStateMachineInput::new_number(
                    11,
                    Some("duplicate".to_owned()),
                    0.0,
                )),
                Some(RuntimeStateMachineInput::new_trigger(
                    12,
                    Some("Case".to_owned()),
                )),
            ],
            vec![fl_c5_empty_listener(40), fl_c5_empty_listener(40)],
            vec![
                fl_c5_layer(20, "duplicate"),
                fl_c5_layer(21, "duplicate"),
                fl_c5_layer(22, "Case"),
            ],
            vec![scripted.clone(), scripted],
        );
        let duplicate_bind = |data_bind_index| RuntimeStateMachineDataBindTemplate {
            data_bind_index,
            authored_path: vec![1, 2],
            data_bind_path: {
                let mut referencer =
                    crate::data_bind_path_referencer::RuntimeDataBindPathReferencer::default();
                assert!(referencer.claim_imported_path(
                    crate::data_bind_path::RuntimeDataBindPath::resolved(vec![1, 2], None)
                ));
                referencer
            },
            name_based: false,
            context_bindable: true,
            flags: 0,
            converter: None,
            converter_data_binds: RuntimeDataConverterDataBindDefinition::default(),
            target: RuntimeDataBindGraphTarget::Number { global_id: 90 },
            source_seed: RuntimeDataBindGraphValue::Untyped,
            source_bound: false,
            view_model_instance_ids: Vec::new(),
        };
        machine.data_bind_templates = Arc::new(vec![duplicate_bind(0), duplicate_bind(1)]);

        assert_eq!(machine.input_count(), 4);
        assert!(machine.input_at(0).is_none());
        assert_eq!(machine.input_at(1).map(|input| input.global_id), Some(10));
        assert_eq!(
            machine
                .input_named("duplicate")
                .map(|input| input.global_id),
            Some(10),
            "the first authored duplicate wins"
        );
        assert!(machine.input_named("DUPLICATE").is_none());
        assert_eq!(machine.layer_count(), 3);
        assert_eq!(
            machine
                .layer_named("duplicate")
                .map(|layer| layer.global_id),
            Some(20),
            "the first authored duplicate wins"
        );
        assert!(machine.layer_named("case").is_none());
        assert_eq!(machine.listener_count(), 2);
        assert_eq!(
            machine
                .listener_at(1)
                .map(|listener| listener.target_local_id),
            Some(40)
        );
        assert_eq!(machine.data_bind_count(), 2);
        assert_eq!(
            machine
                .data_bind_at(1)
                .map(|data_bind| data_bind.data_bind_index),
            Some(1),
            "duplicate-target DataBind occurrences retain authored indices"
        );
        assert_eq!(machine.scripted_object_count(), 2);
        assert_eq!(
            machine
                .scripted_objects()
                .iter()
                .map(ScriptListenerActionDefinition::scripted_object_global_id)
                .collect::<Vec<_>>(),
            [70, 70]
        );
        let mut returned = machine.scripted_objects().to_vec();
        returned.clear();
        assert_eq!(
            machine.scripted_object_count(),
            2,
            "caller mutation of a copied view cannot mutate the definition"
        );
    }

    #[test]
    fn fl_c5_definition_malformed_listener_retains_authored_slot() {
        let type_key = |name: &str| {
            nuxie_schema::definition_by_name(name)
                .unwrap_or_else(|| panic!("missing schema definition {name}"))
                .type_key
                .int
        };
        let mut file = RuntimeFile::from_fixture_records(vec![
            FixtureRecord {
                type_key: type_key("Backboard"),
                properties: Vec::new(),
            },
            FixtureRecord {
                type_key: type_key("Artboard"),
                properties: Vec::new(),
            },
            FixtureRecord {
                type_key: type_key("StateMachine"),
                properties: Vec::new(),
            },
            FixtureRecord {
                type_key: type_key("StateMachineListener"),
                properties: vec![FixtureProperty {
                    key: property_key_for_name("StateMachineListener", "targetId")
                        .expect("targetId property"),
                    value: FixtureValue::Uint(7),
                }],
            },
            FixtureRecord {
                type_key: type_key("StateMachineListenerSingle"),
                properties: vec![FixtureProperty {
                    key: property_key_for_name("StateMachineListenerSingle", "targetId")
                        .expect("targetId property"),
                    value: FixtureValue::Uint(0),
                }],
            },
        ])
        .expect("import listener-slot fixture");
        let malformed = file
            .objects
            .iter_mut()
            .flatten()
            .find(|object| object.type_name == "StateMachineListener")
            .expect("malformed listener object");
        malformed
            .properties
            .iter_mut()
            .find(|property| property.name == "targetId")
            .expect("explicit targetId")
            .value = nuxie_binary::FieldValue::Bool(false);
        assert!(
            malformed.uint_property("targetId").is_none(),
            "fixture must exercise the unbuildable-listener branch"
        );
        let graph = GraphFile::from_runtime_file(&file).expect("build listener-slot graph");
        let mut converter_cache = RuntimeDataBindGraphConverterBuildCache::default();
        let machines = build_state_machines(
            &file,
            graph.artboards.first().expect("fixture artboard"),
            &[],
            &mut converter_cache,
        );
        let machine = machines.first().expect("fixture state machine");

        assert_eq!(machine.listener_count(), 2);
        let inert = machine.listener_at(0).expect("retained inert listener");
        assert_eq!(inert.target_local_id, usize::MAX);
        assert!(inert.listener_types.is_empty());
        assert!(inert.listener_actions.is_empty());
        assert_eq!(
            machine
                .listener_at(1)
                .map(|listener| listener.target_local_id),
            Some(0),
            "the later valid listener keeps authored index one"
        );
    }

    #[test]
    fn fl_c5_definition_added_phases_keep_order_first_error_and_no_rollback() {
        let machine = fl_c5_definition_machine(
            vec![
                Some(RuntimeStateMachineInput::new_trigger(10, None)),
                Some(RuntimeStateMachineInput::new_trigger(11, None)),
            ],
            vec![fl_c5_empty_listener(30), fl_c5_empty_listener(31)],
            vec![fl_c5_layer(20, "first"), fl_c5_layer(21, "second")],
            Vec::new(),
        );
        let dirty_log = RefCell::new(Vec::new());
        let dirty = machine.on_added_dirty(
            |index, _| {
                dirty_log.borrow_mut().push(("input", index));
                Ok::<_, &'static str>(())
            },
            |index, _| {
                dirty_log.borrow_mut().push(("layer", index));
                if index == 1 {
                    return Err("dirty layer failure");
                }
                Ok(())
            },
            |index, _| {
                dirty_log.borrow_mut().push(("listener", index));
                Ok(())
            },
        );
        assert_eq!(dirty, Err("dirty layer failure"));
        assert_eq!(
            dirty_log.into_inner(),
            [("input", 0), ("input", 1), ("layer", 0), ("layer", 1)],
            "earlier callbacks stay observable and later collections do not run"
        );

        let clean_log = RefCell::new(Vec::new());
        let clean = machine.on_added_clean(
            |index, _| {
                clean_log.borrow_mut().push(("input", index));
                Ok::<_, &'static str>(())
            },
            |index, _| {
                clean_log.borrow_mut().push(("layer", index));
                Ok(())
            },
            |index, _| {
                clean_log.borrow_mut().push(("listener", index));
                if index == 0 {
                    return Err("clean listener failure");
                }
                Ok(())
            },
        );
        assert_eq!(clean, Err("clean listener failure"));
        assert_eq!(
            clean_log.into_inner(),
            [
                ("input", 0),
                ("input", 1),
                ("layer", 0),
                ("layer", 1),
                ("listener", 0)
            ]
        );
    }

    #[test]
    fn fl_c5_definition_missing_importer_does_not_attach() {
        let type_key = |name: &str| {
            nuxie_schema::definition_by_name(name)
                .unwrap_or_else(|| panic!("missing schema definition {name}"))
                .type_key
                .int
        };
        let error = RuntimeFile::from_fixture_records(vec![
            FixtureRecord {
                type_key: type_key("Backboard"),
                properties: Vec::new(),
            },
            FixtureRecord {
                type_key: type_key("StateMachine"),
                properties: Vec::new(),
            },
            FixtureRecord {
                type_key: type_key("Artboard"),
                properties: Vec::new(),
            },
        ])
        .expect_err("a StateMachine without an ArtboardImporter must fail import");
        assert!(
            error.to_string().contains("MissingObject"),
            "missing importer status was not retained: {error:#}"
        );
    }

    #[test]
    fn blend_occurrences_retain_definition_handles_and_shared_empty_animation() {
        let file = read_runtime_file(
            &std::fs::read(rive_runtime_fixture("animation_reset_cases.riv"))
                .expect("read animation fixture"),
        )
        .expect("import animation fixture");
        let graph = GraphFile::from_runtime_file(&file).expect("build animation graph");
        let mut artboard = ArtboardInstance::from_graph_with_artboards(
            &file,
            graph.artboards.first().expect("fixture artboard"),
            &graph.artboards,
        )
        .expect("instantiate animation artboard");
        assert!(!artboard.linear_animations().is_empty());

        let blend_state = RuntimeBlendState1D {
            source: RuntimeBlendState1DSource::Input {
                input_index: Some(0),
            },
            animations: vec![
                RuntimeBlendAnimation1D {
                    animation: RuntimeLinearAnimationHandle::new(0),
                    value: 0.0,
                },
                RuntimeBlendAnimation1D {
                    animation: RuntimeLinearAnimationHandle::empty(),
                    value: 100.0,
                },
            ],
        };
        let input_definitions = Arc::new(vec![Some(RuntimeStateMachineInput::new_number(
            1,
            Some("blend".to_owned()),
            25.0,
        ))]);
        let inputs = vec![StateMachineInputInstance::new(0, input_definitions)];
        let mut occurrence = BlendState1DInstance::new(
            &blend_state,
            &artboard,
            &artboard.linear_animations,
            &artboard.empty_linear_animation,
            false,
        );

        assert_eq!(occurrence.animations.len(), blend_state.animations.len());
        assert_eq!(
            occurrence.animations[0].definition,
            RuntimeBlendAnimationHandle::new(0)
        );
        assert_eq!(
            occurrence.animations[1].definition,
            RuntimeBlendAnimationHandle::new(1)
        );
        let empty = artboard
            .linear_animation_instance_definition(&occurrence.animations[1].animation)
            .expect("shared empty definition");
        assert!(std::ptr::eq(
            empty,
            artboard.empty_linear_animation.as_ref()
        ));

        occurrence.advance(&blend_state, &artboard, &inputs, &[], 0.0);
        assert_eq!(occurrence.from, Some(RuntimeBlendAnimationHandle::new(0)));
        assert_eq!(occurrence.to, Some(RuntimeBlendAnimationHandle::new(1)));
        assert_eq!(occurrence.animations[0].mix, 0.75);
        assert_eq!(occurrence.animations[1].mix, 0.25);

        let mut direct_state = RuntimeBlendStateDirect {
            animations: vec![RuntimeBlendAnimationDirect {
                animation: RuntimeLinearAnimationHandle::empty(),
                source: RuntimeDirectBlendSource::MixValue { value: 200.0 },
            }],
        };
        let mut direct_occurrence = BlendStateDirectInstance::new(
            &direct_state,
            &artboard.linear_animations,
            &artboard.empty_linear_animation,
        );
        direct_state.animations[0].source = RuntimeDirectBlendSource::MixValue { value: 40.0 };
        direct_occurrence.advance(&direct_state, &artboard, &[], &[], 0.0);
        assert_eq!(
            direct_occurrence.animations[0].definition,
            RuntimeBlendAnimationHandle::new(0)
        );
        assert_eq!(direct_occurrence.animations[0].mix, 0.4);

        let empty_state = RuntimeLayerState {
            global_id: Some(2),
            type_name: Some("AnimationState"),
            animation: Some(RuntimeLinearAnimationHandle::empty()),
            blend_state_1d: None,
            blend_state_direct: None,
            speed: 1.0,
            flags: 0,
            fire_actions: Vec::new(),
            listener_actions: Vec::new(),
            transitions: Vec::new(),
        };
        let layer = RuntimeStateMachineLayer {
            global_id: 3,
            name: None,
            states: vec![empty_state],
            entry_state_index: Some(0),
            any_state_index: None,
            exit_state_index: None,
        };
        let layer_occurrence = StateMachineLayerInstance::new(&layer, "", &artboard, &[], &[], &[]);
        let empty_state_animation = layer_occurrence
            .current_animation()
            .expect("AnimationState always creates an animation occurrence");
        assert!(std::ptr::eq(
            artboard
                .linear_animation_instance_definition(empty_state_animation)
                .expect("AnimationState empty definition"),
            artboard.empty_linear_animation.as_ref()
        ));
        assert!(!empty_state_animation.apply(&mut artboard, 1.0));
    }

    #[test]
    fn scripted_listener_action_resolves_non_module_script_asset_by_file_ordinal() {
        let file = read_runtime_file(
            &std::fs::read(rive_runtime_fixture("scripted_listener_action.riv"))
                .expect("read scripted listener fixture"),
        )
        .expect("import scripted listener fixture");
        let graph = GraphFile::from_runtime_file(&file).expect("build fixture graph");
        let artboard = graph.artboards.first().expect("fixture artboard");
        let mut converter_cache = RuntimeDataBindGraphConverterBuildCache::default();
        let state_machines = build_state_machines(&file, artboard, &[], &mut converter_cache);
        let action = state_machines
            .first()
            .expect("fixture state machine")
            .scripted_listener_actions()
            .first()
            .expect("scripted listener action");

        assert_eq!(action.action_global_id(), 55);
        assert_eq!(action.asset_ordinal(), 0);
        assert_eq!(action.asset_name(), "ListenerActionAppend");
    }

    #[test]
    fn scripted_listener_action_retains_module_asset_as_inert() {
        let mut file = read_runtime_file(
            &std::fs::read(rive_runtime_fixture("scripted_listener_action.riv"))
                .expect("read scripted listener fixture"),
        )
        .expect("import scripted listener fixture");
        file.objects
            .get_mut(1)
            .and_then(Option::as_mut)
            .expect("first ScriptAsset")
            .properties
            .push(nuxie_binary::RuntimeProperty {
                key: 914,
                name: "isModule",
                owner: "ScriptAsset",
                value: nuxie_binary::FieldValue::Bool(true),
            });
        let graph = GraphFile::from_runtime_file(&file).expect("build fixture graph");
        let mut converter_cache = RuntimeDataBindGraphConverterBuildCache::default();
        let state_machines = build_state_machines(
            &file,
            graph.artboards.first().expect("fixture artboard"),
            &[],
            &mut converter_cache,
        );

        let actions = state_machines
            .first()
            .expect("fixture state machine")
            .scripted_listener_actions();
        assert_eq!(actions.len(), 1);
        assert_eq!(actions[0].action_global_id(), 55);
        assert_eq!(actions[0].asset_ordinal(), 0);
        assert_eq!(actions[0].asset_name(), "ListenerActionAppend");
        assert!(
            !actions[0].has_protocol_asset(),
            "C++ retains the action and its inputs but module ScriptAssets have no protocol generator"
        );
    }

    #[test]
    fn scripted_listener_action_retains_missing_out_of_range_and_wrong_assets() {
        fn actions(file: &RuntimeFile) -> Vec<ScriptListenerActionDefinition> {
            let graph = GraphFile::from_runtime_file(file).expect("build fixture graph");
            let mut converter_cache = RuntimeDataBindGraphConverterBuildCache::default();
            build_state_machines(
                file,
                graph.artboards.first().expect("fixture artboard"),
                &[],
                &mut converter_cache,
            )
            .first()
            .expect("fixture state machine")
            .scripted_listener_actions()
            .to_vec()
        }

        fn action_mut(file: &mut RuntimeFile) -> &mut RuntimeObject {
            file.objects
                .iter_mut()
                .flatten()
                .find(|object| object.type_name == "ScriptedListenerAction")
                .expect("fixture ScriptedListenerAction")
        }

        let bytes = std::fs::read(rive_runtime_fixture("scripted_listener_action.riv"))
            .expect("read scripted listener fixture");
        let baseline_file = read_runtime_file(&bytes).expect("import baseline fixture");
        let baseline = actions(&baseline_file);
        assert_eq!(baseline.len(), 1);
        let baseline_input_ids = baseline[0]
            .inputs()
            .iter()
            .map(|input| input.input_global_id())
            .collect::<Vec<_>>();

        for case in ["missing", "out-of-range", "wrong-type"] {
            let mut file = read_runtime_file(&bytes).expect("import mutated fixture");
            match case {
                "missing" => {
                    action_mut(&mut file)
                        .properties
                        .retain(|property| property.name != "scriptAssetId");
                }
                "out-of-range" => {
                    let property = action_mut(&mut file)
                        .properties
                        .iter_mut()
                        .find(|property| property.name == "scriptAssetId")
                        .expect("fixture scriptAssetId");
                    property.value = nuxie_binary::FieldValue::Uint(999);
                }
                "wrong-type" => {
                    file.objects
                        .iter_mut()
                        .flatten()
                        .find(|object| object.type_name == "ScriptAsset")
                        .expect("fixture ScriptAsset")
                        .type_name = "ImageAsset";
                }
                _ => unreachable!(),
            }

            let retained = actions(&file);
            assert_eq!(retained.len(), 1, "{case}");
            assert_eq!(retained[0].action_global_id(), 55, "{case}");
            assert!(!retained[0].has_protocol_asset(), "{case}");
            assert_eq!(
                retained[0]
                    .inputs()
                    .iter()
                    .map(|input| input.input_global_id())
                    .collect::<Vec<_>>(),
                baseline_input_ids,
                "{case}: the authored input occurrence list must survive unchanged"
            );
        }
    }

    #[test]
    fn scheduled_listener_batch_keeps_scripted_actions_in_authored_order() {
        struct RecordingExecutor {
            reported_event_counts: Vec<usize>,
            fail: bool,
        }

        impl RuntimeScheduledListenerActionExecutor for RecordingExecutor {
            fn perform_instance_action(
                &mut self,
                _artboard: &mut ArtboardInstance,
                action: &RuntimeScheduledListenerAction,
                targets: RuntimeScheduledListenerActionTargetsMut<'_>,
            ) -> Result<bool, ScriptError> {
                assert!(matches!(
                    action,
                    RuntimeScheduledListenerAction::Scripted { .. }
                ));
                self.reported_event_counts
                    .push(targets.reported_events.len());
                if self.fail {
                    return Err(ScriptError::new("scheduled listener failed"));
                }
                Ok(true)
            }
        }

        let type_key = |name: &str| {
            nuxie_schema::definition_by_name(name)
                .unwrap_or_else(|| panic!("missing schema definition {name}"))
                .type_key
                .int
        };
        let parent = |owner: &str, value: u64| FixtureProperty {
            key: property_key_for_name(owner, "parentId").expect("parentId property"),
            value: FixtureValue::Uint(value),
        };
        let file = RuntimeFile::from_fixture_records(vec![
            FixtureRecord {
                type_key: type_key("Backboard"),
                properties: Vec::new(),
            },
            FixtureRecord {
                type_key: type_key("Artboard"),
                properties: Vec::new(),
            },
            FixtureRecord {
                type_key: type_key("Event"),
                properties: vec![parent("Event", 0)],
            },
            FixtureRecord {
                type_key: type_key("Event"),
                properties: vec![parent("Event", 0)],
            },
        ])
        .expect("import two live event occurrences");
        let actions = vec![
            RuntimeScheduledListenerAction::FireEvent(
                listener_fire_event::RuntimeListenerFireEvent::for_test(
                    StateMachineFireOccurrence::AtStart.value(),
                    Some(1),
                ),
            ),
            RuntimeScheduledListenerAction::scripted_for_test(
                StateMachineFireOccurrence::AtStart.value(),
                Some(ScriptListenerActionDefinition::new(
                    44,
                    2,
                    "action".to_owned(),
                )),
            ),
            RuntimeScheduledListenerAction::FireEvent(
                listener_fire_event::RuntimeListenerFireEvent::for_test(
                    StateMachineFireOccurrence::AtStart.value(),
                    Some(2),
                ),
            ),
        ];
        let graph = GraphFile::from_runtime_file(&file).expect("build fixture graph");
        let mut artboard = ArtboardInstance::from_graph_with_artboards(
            &file,
            graph.artboards.first().expect("fixture artboard"),
            &graph.artboards,
        )
        .expect("instantiate listener artboard");
        let mut reported_events = Vec::new();
        let mut executor = RecordingExecutor {
            reported_event_counts: Vec::new(),
            fail: false,
        };

        assert!(
            perform_scheduled_listener_actions(
                &actions,
                StateMachineFireOccurrence::AtStart,
                &mut artboard,
                RuntimeScheduledListenerActionTargetsMut {
                    inputs: &mut [],
                    reported_events: &mut reported_events,
                    bindable_numbers: &mut [],
                    bindable_integers: &mut [],
                    bindable_colors: &mut [],
                    bindable_strings: &mut [],
                    bindable_enums: &mut [],
                    bindable_assets: &mut [],
                    bindable_artboards: &mut [],
                    bindable_lists: &mut [],
                    bindable_triggers: &mut [],
                    bindable_view_models: &mut [],
                    bindable_booleans: &mut [],
                    transition_durations: &mut [],
                },
                &mut executor,
            )
            .expect("execute scheduled listener actions")
        );
        assert_eq!(executor.reported_event_counts, [1]);
        assert_eq!(
            reported_events
                .iter()
                .map(StateMachineReportedEvent::event_local_index)
                .collect::<Vec<_>>(),
            [1, 2]
        );

        reported_events.clear();
        executor.fail = true;
        assert!(
            perform_scheduled_listener_actions(
                &actions,
                StateMachineFireOccurrence::AtStart,
                &mut artboard,
                RuntimeScheduledListenerActionTargetsMut {
                    inputs: &mut [],
                    reported_events: &mut reported_events,
                    bindable_numbers: &mut [],
                    bindable_integers: &mut [],
                    bindable_colors: &mut [],
                    bindable_strings: &mut [],
                    bindable_enums: &mut [],
                    bindable_assets: &mut [],
                    bindable_artboards: &mut [],
                    bindable_lists: &mut [],
                    bindable_triggers: &mut [],
                    bindable_view_models: &mut [],
                    bindable_booleans: &mut [],
                    transition_durations: &mut [],
                },
                &mut executor,
            )
            .expect("script failure is consumed and the authored action tail continues")
        );
        assert_eq!(
            reported_events
                .iter()
                .map(StateMachineReportedEvent::event_local_index)
                .collect::<Vec<_>>(),
            [1, 2]
        );
    }

    #[test]
    fn blend_1d_retained_arena_identities_survive_rust_clone_and_remount() {
        let file = read_runtime_file(
            &std::fs::read(rive_runtime_fixture("animation_reset_cases.riv"))
                .expect("read animation fixture"),
        )
        .expect("import animation fixture");
        let graph = GraphFile::from_runtime_file(&file).expect("build animation graph");
        let artboard = ArtboardInstance::from_graph_with_artboards(
            &file,
            graph.artboards.first().expect("fixture artboard"),
            &graph.artboards,
        )
        .expect("instantiate animation artboard");
        let blend_state = RuntimeBlendState1D {
            source: RuntimeBlendState1DSource::Input {
                input_index: Some(0),
            },
            animations: vec![
                RuntimeBlendAnimation1D {
                    animation: RuntimeLinearAnimationHandle::new(0),
                    value: 0.0,
                },
                RuntimeBlendAnimation1D {
                    animation: RuntimeLinearAnimationHandle::empty(),
                    value: 100.0,
                },
            ],
        };
        let input_definitions = Arc::new(vec![Some(RuntimeStateMachineInput::new_number(
            1,
            Some("blend".to_owned()),
            25.0,
        ))]);
        let inputs = vec![StateMachineInputInstance::new(0, input_definitions)];
        let mut occurrence = BlendState1DInstance::new(
            &blend_state,
            &artboard,
            &artboard.linear_animations,
            &artboard.empty_linear_animation,
            false,
        );
        occurrence.advance(&blend_state, &artboard, &inputs, &[], 0.0);
        let retained_from = occurrence.from;
        let retained_to = occurrence.to;
        assert!(retained_from.is_some(), "the lower occurrence is selected");
        assert!(retained_to.is_some(), "the upper occurrence is selected");
        assert_ne!(
            retained_from, retained_to,
            "the retained blend endpoints are distinct arena handles",
        );
        let retained_definitions = occurrence
            .animations
            .iter()
            .map(|animation| animation.definition)
            .collect::<Vec<_>>();

        let mut cloned = occurrence.clone();
        let remounted = ArtboardInstance::from_graph_with_artboards(
            &file,
            graph.artboards.first().expect("fixture artboard"),
            &graph.artboards,
        )
        .expect("remount animation artboard");
        cloned.advance(&blend_state, &remounted, &inputs, &[], 0.0);

        // FL-B4 requires `from`/`to` to remain handles into the retained
        // occurrence arena. Clone/remount must preserve those identities
        // without reset replay or any other behavioral compensation.
        assert_eq!(cloned.from, retained_from);
        assert_eq!(cloned.to, retained_to);
        assert_eq!(
            cloned
                .animations
                .iter()
                .map(|animation| animation.definition)
                .collect::<Vec<_>>(),
            retained_definitions,
        );
    }

    #[test]
    fn blend_direct_retained_definition_handles_survive_rust_clone_and_remount() {
        let file = read_runtime_file(
            &std::fs::read(rive_runtime_fixture("animation_reset_cases.riv"))
                .expect("read animation fixture"),
        )
        .expect("import animation fixture");
        let graph = GraphFile::from_runtime_file(&file).expect("build animation graph");
        let artboard = ArtboardInstance::from_graph_with_artboards(
            &file,
            graph.artboards.first().expect("fixture artboard"),
            &graph.artboards,
        )
        .expect("instantiate animation artboard");
        let blend_state = RuntimeBlendStateDirect {
            animations: vec![
                RuntimeBlendAnimationDirect {
                    animation: RuntimeLinearAnimationHandle::new(0),
                    source: RuntimeDirectBlendSource::MixValue { value: 25.0 },
                },
                RuntimeBlendAnimationDirect {
                    animation: RuntimeLinearAnimationHandle::empty(),
                    source: RuntimeDirectBlendSource::MixValue { value: 75.0 },
                },
            ],
        };
        let mut occurrence = BlendStateDirectInstance::new(
            &blend_state,
            &artboard.linear_animations,
            &artboard.empty_linear_animation,
        );
        occurrence.advance(&blend_state, &artboard, &[], &[], 0.0);
        let retained_definitions = occurrence
            .animations
            .iter()
            .map(|animation| animation.definition)
            .collect::<Vec<_>>();
        assert_eq!(
            retained_definitions,
            [
                RuntimeBlendAnimationHandle::new(0),
                RuntimeBlendAnimationHandle::new(1),
            ],
            "each direct-blend occurrence retains its exact definition handle",
        );

        let mut cloned = occurrence.clone();
        let remounted = ArtboardInstance::from_graph_with_artboards(
            &file,
            graph.artboards.first().expect("fixture artboard"),
            &graph.artboards,
        )
        .expect("remount animation artboard");
        cloned.advance(&blend_state, &remounted, &[], &[], 0.0);

        assert_eq!(
            cloned
                .animations
                .iter()
                .map(|animation| animation.definition)
                .collect::<Vec<_>>(),
            retained_definitions,
            "Rust-only clone/remount preserves BlendDirect arena identity",
        );
    }

    #[test]
    fn reported_event_metadata_preserves_open_url_values_and_ordinary_absence() {
        assert_eq!(open_url_target(0), "_blank");
        assert_eq!(open_url_target(1), "_parent");
        assert_eq!(open_url_target(2), "_self");
        assert_eq!(open_url_target(3), "_top");
        assert_eq!(open_url_target(4), "");
        assert_eq!(open_url_target(u64::MAX), "");

        let fixture = rive_runtime_fixture("event_on_listener.riv");
        let file = read_runtime_file(&std::fs::read(fixture).expect("read event fixture"))
            .expect("import event fixture");
        let open_url = file
            .objects
            .iter()
            .flatten()
            .find(|object| {
                object.type_name == "OpenUrlEvent"
                    && object.string_property("url") == Some("http://rive.app/delete-me")
            })
            .expect("authored OpenURL event");
        let open_url = StateMachineReportedEvent::from_runtime_event(7, open_url);
        assert_eq!(open_url.url(), Some("http://rive.app/delete-me"));
        assert_eq!(open_url.target(), Some("_blank"));

        let ordinary = file
            .objects
            .iter()
            .flatten()
            .find(|object| object.type_name == "Event")
            .expect("ordinary event");
        let ordinary = StateMachineReportedEvent::from_runtime_event(8, ordinary);
        assert_eq!(ordinary.url(), None);
        assert_eq!(ordinary.target(), None);
    }

    #[test]
    fn animation_reset_retains_first_seen_owner_order_and_shares_one_pool_lease() {
        let file = read_runtime_file(
            &std::fs::read(rive_runtime_fixture("animation_reset_cases.riv"))
                .expect("read animation-reset fixture"),
        )
        .expect("import animation-reset fixture");
        let graph = GraphFile::from_runtime_file(&file).expect("build animation-reset graph");
        let mut artboard = ArtboardInstance::from_graph_with_artboards(
            &file,
            graph.artboards.first().expect("fixture artboard"),
            &graph.artboards,
        )
        .expect("instantiate animation-reset artboard");
        let animation_instances = (0..artboard.linear_animations().len())
            .filter_map(|index| artboard.linear_animation_instance(index))
            .collect::<Vec<_>>();

        let reset =
            AnimationResetFactory::from_animation_instances(&artboard, &animation_instances, false);
        let cloned = reset.clone();
        assert!(
            Arc::ptr_eq(&reset.storage, &cloned.storage),
            "Rust snapshot clones must share one factory-owned reset lease"
        );

        let actual = reset
            .storage
            .entries
            .iter()
            .map(|entry| match entry {
                AnimationResetEntry::Double {
                    local_id,
                    property_key,
                    ..
                }
                | AnimationResetEntry::Color {
                    local_id,
                    property_key,
                    ..
                } => (*local_id, *property_key),
            })
            .collect::<Vec<_>>();
        let mut expected_objects = Vec::<(usize, BTreeSet<u16>, Vec<u16>)>::new();
        for animation in artboard.linear_animations() {
            for keyed_object in animation.keyed_objects.iter() {
                let object_index = expected_objects
                    .iter()
                    .position(|(local_id, _, _)| *local_id == keyed_object.target_local_id)
                    .unwrap_or_else(|| {
                        expected_objects.push((
                            keyed_object.target_local_id,
                            BTreeSet::new(),
                            Vec::new(),
                        ));
                        expected_objects.len() - 1
                    });
                let (_, seen, properties) = &mut expected_objects[object_index];
                for keyed_property in &keyed_object.keyed_properties {
                    if matches!(
                        &keyed_property.target,
                        RuntimeKeyedPropertyTarget::Double { .. }
                            | RuntimeKeyedPropertyTarget::Color { .. }
                    ) && seen.insert(keyed_property.property_key)
                    {
                        properties.push(keyed_property.property_key);
                    }
                }
            }
        }
        let expected = expected_objects
            .into_iter()
            .flat_map(|(local_id, _, properties)| {
                properties
                    .into_iter()
                    .map(move |property_key| (local_id, property_key))
            })
            .collect::<Vec<_>>();
        assert!(!actual.is_empty());
        assert_eq!(actual, expected);

        reset.apply(&mut artboard);
        let empty =
            AnimationResetFactory::from_animation_instances(&artboard, std::iter::empty(), false);
        assert!(
            empty.storage.entries.is_empty(),
            "the factory must return an owned empty reset, not null"
        );
    }

    #[test]
    fn animation_reset_color_uses_cpp_signed_float_round_trip() {
        assert_eq!(
            AnimationResetColorValue::from_color(0x011d_1d1d).replay(),
            0x011d_1d1c,
            "pinned animation_reset_factory.cpp:126-168 stores color int bits as float"
        );
        assert_eq!(
            AnimationResetColorValue::from_color(0xff1d_1d1d).replay(),
            0xff1d_1d1d,
            "negative signed colors also round-trip through the C++ float representation"
        );
        assert_eq!(
            AnimationResetColorValue::from_color(0x7fff_ffff),
            AnimationResetColorValue::SaturatingFloatToInt(2_147_483_648.0),
            "2^31 cannot be converted back to C++ int with defined behavior"
        );
        assert_eq!(
            AnimationResetColorValue::from_color(0x7fff_ffff).replay(),
            0x7fff_ffff,
            "project divergence D2 saturates the otherwise-undefined conversion"
        );
    }
}
