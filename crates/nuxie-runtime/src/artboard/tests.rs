    use super::*;
    use crate::Mat2D;
    use crate::animation::{
        RuntimeKeyFrame, RuntimeKeyFrameCallback, RuntimeKeyFrameDouble, RuntimeKeyedObject,
        RuntimeKeyedProperty, RuntimeKeyedPropertyTarget,
    };
    use crate::components::{
        DataBindHandle, RuntimeComponentCapabilities, SoloMappingWork, TransformRuntimeState,
        reset_solo_mapping_work, solo_mapping_work,
    };
    use crate::data_bind_graph::{
        RuntimeDataBindGraphConverter, runtime_data_bind_graph_reverse_convert_value,
    };
    use crate::properties::property_key_for_name;
    use crate::state_machine::{
        RuntimeBlendState1D, RuntimeBlendState1DSource, RuntimeLayerState,
        RuntimeListenerBoolChange, RuntimeListenerInputTarget, RuntimeListenerNumberChange,
        RuntimeListenerTriggerChange, RuntimeListenerType, RuntimeNestedEventChainPhase,
        RuntimeNestedEventChainTrace, RuntimeNestedNotifyBatchTrace,
        RuntimeScheduledListenerAction, RuntimeStateMachineInput, RuntimeStateMachineLayer,
        RuntimeStateMachineListener, ScriptGamepadMappingKind, ScriptGamepadSnapshot,
        ScriptListenerInvocation, StateMachineInputInstance,
    };
    use nuxie_binary::{
        AuthoringProperty, AuthoringRecord, AuthoringValue, BytesValue, FieldValue, RuntimeObject,
        RuntimeProperty, StringValue, read_runtime_file,
    };
    use nuxie_graph::{DependencyNodeKind, GraphFile};
    use nuxie_render_api::RecordingFactory;
    use nuxie_schema::definition_by_name;
    use std::cell::{Cell, RefCell};
    use std::path::PathBuf;
    use std::rc::Rc;
    use std::sync::atomic::{AtomicBool, Ordering};

    #[test]
    fn profiler_component_list_path_is_root_to_leaf_with_logical_index() {
        let parent = vec![
            crate::ProfilePathSegment::nested_artboard("Outer"),
            crate::ProfilePathSegment::component_list("Cards", 2),
        ];

        let path = component_list_profile_path(&parent, "Inner", "Rows", 7);

        assert_eq!(
            path,
            vec![
                crate::ProfilePathSegment::nested_artboard("Outer"),
                crate::ProfilePathSegment::component_list("Cards", 2),
                crate::ProfilePathSegment::nested_artboard("Inner"),
                crate::ProfilePathSegment::component_list("Rows", 7),
            ]
        );
    }

    struct ArtboardPollTask {
        state: crate::WorkTaskState,
        completed: AtomicBool,
        callback_thread: std::sync::Mutex<Option<std::thread::ThreadId>>,
    }

    impl crate::WorkTask for ArtboardPollTask {
        fn state(&self) -> &crate::WorkTaskState {
            &self.state
        }

        fn execute(&self) -> bool {
            true
        }

        fn on_complete(&self) {
            *self
                .callback_thread
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner()) =
                Some(std::thread::current().id());
            self.completed.store(true, Ordering::Release);
        }
    }

    #[test]
    fn root_artboard_advance_polls_global_async_work_before_advancing() {
        // The pool behind the advance seam is process-global (upstream
        // assumes one main thread polls it; `src/artboard.cpp` pollAsyncWork
        // at d788e8ec), but this harness runs tests concurrently and every
        // root advance drains that pool. Two interleavings can therefore
        // separate one submit from one advance without saying anything about
        // the seam under test:
        //   - a concurrently advancing test pops this task and delivers its
        //     callback on a foreign thread before our own advance runs
        //     (reproducible: ~half of parallel `advance`-filter runs with
        //     `threading` enabled), and
        //   - the worker publishes `Completed` status before it enqueues the
        //     task in the completed queue, so our advance can poll an empty
        //     queue even under `--test-threads=1`; the task is then drained
        //     by the next trial's advance.
        // Each trial that failed to observe its own delivery is discarded
        // and rerun with a fresh task. A pass still requires the full
        // authored property — the callback delivered on the advancing
        // thread — and a regressed advance (not polling, or dispatching
        // off-thread) can never produce a pass verdict.
        let polling_thread = std::thread::current().id();
        let mut artboard = synthetic_instance(Vec::new(), Vec::new());
        for _ in 0..32 {
            let task = std::sync::Arc::new(ArtboardPollTask {
                state: crate::WorkTaskState::default(),
                completed: AtomicBool::new(false),
                callback_thread: std::sync::Mutex::new(None),
            });
            crate::with_global_work_pool(|pool| {
                pool.submit(Some(task.clone()));
            });

            #[cfg(feature = "threading")]
            while matches!(
                task.state.status(),
                crate::WorkStatus::Pending | crate::WorkStatus::Running
            ) {
                std::thread::yield_now();
            }

            let _ = artboard.advance(0.0).expect("advance empty artboard");

            // A foreign poller pops the task before it delivers the
            // callback, so give an in-flight steal a bounded window to
            // publish its thread id before judging the trial. (A trial lost
            // to the status-before-enqueue gap exhausts this window with
            // `completed` still false and retries.)
            let mut spins = 0u32;
            while !task.completed.load(Ordering::Acquire) && spins < 1 << 16 {
                spins += 1;
                std::thread::yield_now();
            }
            if *task
                .callback_thread
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                == Some(polling_thread)
            {
                return;
            }
        }
        panic!(
            "async-work callback was never delivered on the advancing \
             thread: advance() stopped polling the global pool before \
             advancing, or callbacks are dispatched off-thread"
        );
    }

    struct UpdateScriptInstance {
        inits: Rc<Cell<usize>>,
        updates: Rc<Cell<usize>>,
    }

    #[test]
    fn upstream_runtime_nested_inputs_fixture_aliases_share_live_occurrences() {
        let fixture = PathBuf::from(
            std::env::var_os("RIVE_RUNTIME_DIR")
                .unwrap_or_else(|| "/Users/levi/dev/oss/rive-runtime".into()),
        )
        .join("tests/unit_tests/assets/runtime_nested_inputs.riv");
        let file = read_runtime_file(&std::fs::read(&fixture).expect("read nested-input fixture"))
            .expect("import nested-input fixture");
        let graph = GraphFile::from_runtime_file(&file).expect("build nested-input graph");
        let main_index = graph
            .artboards
            .iter()
            .position(|artboard| artboard.name.as_deref() == Some("MainArtboard"))
            .expect("MainArtboard");
        let main_graph = &graph.artboards[main_index];
        let mut main =
            ArtboardInstance::from_graph_with_artboards(&file, main_graph, &graph.artboards)
                .expect("instantiate MainArtboard");
        assert_eq!(main_graph.state_machines.len(), 1);

        let local_named = |artboard: &ArtboardGraph, type_name: &str, name: &str| {
            artboard
                .local_objects
                .iter()
                .find(|object| {
                    object.type_name == Some(type_name) && object.name.as_deref() == Some(name)
                })
                .map(|object| object.local_id)
                .unwrap_or_else(|| panic!("{type_name} {name}"))
        };
        let nested_input_named =
            |instance: &ArtboardInstance, artboard: &ArtboardGraph, type_name: &str, name: &str| {
                artboard
                    .local_objects
                    .iter()
                    .filter(|object| object.type_name == Some(type_name))
                    .find(|object| {
                        instance
                            .nested_input_target(object.local_id)
                            .and_then(|(machine_local, input_id)| {
                                instance
                                    .nested_state_machine(machine_local)
                                    .and_then(|machine| machine.input(input_id))
                            })
                            .and_then(StateMachineInputInstance::name)
                            == Some(name)
                    })
                    .map(|object| object.local_id)
                    .unwrap_or_else(|| panic!("{type_name} forwarding {name}"))
            };
        let assert_bool_aliases = |artboard: &ArtboardInstance, local_id: usize, expected: bool| {
            let (machine_local, input_id) = artboard
                .nested_input_target(local_id)
                .expect("nested bool target");
            assert_eq!(artboard.nested_bool_value(local_id), Some(expected));
            assert_eq!(
                artboard
                    .nested_state_machine(machine_local)
                    .and_then(|machine| machine.input(input_id))
                    .and_then(StateMachineInputInstance::bool_value),
                Some(expected)
            );
            assert_eq!(
                artboard.nested_bool_value(local_id),
                Some(expected),
                "NestedBool virtual alias"
            );
        };
        let assert_number_aliases =
            |artboard: &ArtboardInstance, local_id: usize, expected: f32| {
                let (machine_local, input_id) = artboard
                    .nested_input_target(local_id)
                    .expect("nested number target");
                assert_eq!(artboard.nested_number_value(local_id), Some(expected));
                assert_eq!(
                    artboard
                        .nested_state_machine(machine_local)
                        .and_then(|machine| machine.input(input_id))
                        .and_then(StateMachineInputInstance::number_value),
                    Some(expected)
                );
                assert_eq!(
                    artboard.nested_number_value(local_id),
                    Some(expected),
                    "NestedNumber virtual alias"
                );
            };

        let outer_bool = nested_input_named(&main, main_graph, "NestedBool", "CircleOuterState");
        assert_bool_aliases(&main, outer_bool, false);
        assert!(main.set_nested_bool_value(outer_bool, true));
        assert_bool_aliases(&main, outer_bool, true);
        let (outer_bool_machine, outer_bool_input) = main
            .nested_input_target(outer_bool)
            .expect("outer bool target");
        assert!(
            main.nested_state_machine_mut(outer_bool_machine)
                .expect("outer state machine")
                .set_bool(outer_bool_input, false)
        );
        assert_bool_aliases(&main, outer_bool, false);
        assert!(main.set_nested_bool_value(outer_bool, true));
        assert_bool_aliases(&main, outer_bool, true);

        let outer_number =
            nested_input_named(&main, main_graph, "NestedNumber", "CircleOuterNumber");
        assert_number_aliases(&main, outer_number, 0.0);
        assert!(main.set_nested_number_value(outer_number, 10.0));
        assert_number_aliases(&main, outer_number, 10.0);
        let (outer_number_machine, outer_number_input) = main
            .nested_input_target(outer_number)
            .expect("outer number target");
        assert!(
            main.nested_state_machine_mut(outer_number_machine)
                .expect("outer state machine")
                .set_number(outer_number_input, 5.0)
        );
        assert_number_aliases(&main, outer_number, 5.0);
        assert!(main.set_nested_number_value(outer_number, 99.0));
        assert_number_aliases(&main, outer_number, 99.0);

        let outer_trigger =
            nested_input_named(&main, main_graph, "NestedTrigger", "CircleOuterTrigger");
        let (trigger_machine, trigger_input) = main
            .nested_input_target(outer_trigger)
            .expect("outer trigger target");
        for _ in 0..3 {
            assert_eq!(
                main.nested_state_machine(trigger_machine)
                    .and_then(|machine| machine.input(trigger_input))
                    .and_then(StateMachineInputInstance::trigger_fired),
                Some(false)
            );
        }
        assert!(main.fire_nested_trigger_input(outer_trigger));
        for _ in 0..3 {
            assert_eq!(
                main.nested_state_machine(trigger_machine)
                    .and_then(|machine| machine.input(trigger_input))
                    .and_then(StateMachineInputInstance::trigger_fired),
                Some(true)
            );
        }

        let outer_host = local_named(main_graph, "NestedArtboard", "CircleOuter");
        let inner_graph = graph
            .artboards
            .iter()
            .find(|artboard| {
                artboard
                    .local_objects
                    .iter()
                    .any(|object| object.type_name == Some("NestedBool"))
                    && artboard.global_id != main_graph.global_id
            })
            .expect("outer-circle artboard graph");
        let outer_child = main
            .nested_artboards
            .get_mut(&outer_host)
            .expect("CircleOuter occurrence")
            .child
            .as_mut();
        let inner_bool =
            nested_input_named(outer_child, inner_graph, "NestedBool", "CircleInnerState");
        assert_bool_aliases(outer_child, inner_bool, false);
        assert!(outer_child.set_nested_bool_value(inner_bool, true));
        assert_bool_aliases(outer_child, inner_bool, true);
        let (inner_machine, inner_input) = outer_child
            .nested_input_target(inner_bool)
            .expect("inner bool target");
        assert!(
            outer_child
                .nested_state_machine_mut(inner_machine)
                .expect("inner state machine")
                .set_bool(inner_input, false)
        );
        assert_bool_aliases(outer_child, inner_bool, false);
        assert!(outer_child.set_nested_bool_value(inner_bool, true));
        assert_bool_aliases(outer_child, inner_bool, true);
    }

    struct AdvanceScriptInstance {
        advances: Rc<Cell<usize>>,
    }

    struct RecordingAdvanceScriptInstance {
        seconds: Rc<RefCell<Vec<f32>>>,
    }

    struct OrderedAdvanceScriptInstance {
        label: u32,
        calls: Rc<RefCell<Vec<u32>>>,
    }

    struct OrderedUpdateScriptInstance {
        label: u32,
        calls: Rc<RefCell<Vec<u32>>>,
        fail_once: Option<Rc<Cell<bool>>>,
    }

    struct AdvanceAndUpdateScriptInstance {
        advances: Rc<Cell<usize>>,
        updates: Rc<Cell<usize>>,
    }

    struct FailOnceAdvanceScriptInstance {
        attempts: Rc<RefCell<Vec<f32>>>,
        should_fail: Rc<Cell<bool>>,
    }

    impl ScriptInstance for AdvanceScriptInstance {
        fn has_method(&self, method: ScriptMethod) -> Result<bool, ScriptError> {
            Ok(method == ScriptMethod::Advance)
        }

        fn call_method(
            &mut self,
            method: ScriptMethod,
            args: &[ScriptValue],
            _host: &mut dyn crate::ScriptHost,
        ) -> Result<ScriptValue, ScriptError> {
            assert_eq!(method, ScriptMethod::Advance);
            assert_eq!(args.len(), 1);
            assert_eq!(args[0].as_number().map(|value| value as f32), Some(0.1));
            let count = self.advances.get() + 1;
            self.advances.set(count);
            Ok(ScriptValue::Bool(count != 2))
        }

        fn get_input(&self, _name: &str) -> Result<ScriptValue, ScriptError> {
            Ok(ScriptValue::Nil)
        }

        fn set_input(&mut self, _name: &str, _value: ScriptValue) -> Result<(), ScriptError> {
            Ok(())
        }
    }

    impl ScriptInstance for RecordingAdvanceScriptInstance {
        fn has_method(&self, method: ScriptMethod) -> Result<bool, ScriptError> {
            Ok(method == ScriptMethod::Advance)
        }

        fn call_method(
            &mut self,
            method: ScriptMethod,
            args: &[ScriptValue],
            _host: &mut dyn crate::ScriptHost,
        ) -> Result<ScriptValue, ScriptError> {
            assert_eq!(method, ScriptMethod::Advance);
            let seconds = args
                .first()
                .and_then(ScriptValue::as_number)
                .map(|value| value as f32)
                .expect("advance receives seconds");
            self.seconds.borrow_mut().push(seconds);
            Ok(ScriptValue::Bool(true))
        }

        fn get_input(&self, _name: &str) -> Result<ScriptValue, ScriptError> {
            Ok(ScriptValue::Nil)
        }

        fn set_input(&mut self, _name: &str, _value: ScriptValue) -> Result<(), ScriptError> {
            Ok(())
        }
    }

    impl ScriptInstance for OrderedAdvanceScriptInstance {
        fn has_method(&self, method: ScriptMethod) -> Result<bool, ScriptError> {
            Ok(method == ScriptMethod::Advance)
        }

        fn call_method(
            &mut self,
            method: ScriptMethod,
            _args: &[ScriptValue],
            _host: &mut dyn crate::ScriptHost,
        ) -> Result<ScriptValue, ScriptError> {
            assert_eq!(method, ScriptMethod::Advance);
            self.calls.borrow_mut().push(self.label);
            Ok(ScriptValue::Bool(true))
        }

        fn get_input(&self, _name: &str) -> Result<ScriptValue, ScriptError> {
            Ok(ScriptValue::Nil)
        }

        fn set_input(&mut self, _name: &str, _value: ScriptValue) -> Result<(), ScriptError> {
            Ok(())
        }
    }

    impl ScriptInstance for OrderedUpdateScriptInstance {
        fn has_method(&self, method: ScriptMethod) -> Result<bool, ScriptError> {
            Ok(method == ScriptMethod::Update)
        }

        fn call_method(
            &mut self,
            method: ScriptMethod,
            _args: &[ScriptValue],
            _host: &mut dyn crate::ScriptHost,
        ) -> Result<ScriptValue, ScriptError> {
            assert_eq!(method, ScriptMethod::Update);
            self.calls.borrow_mut().push(self.label);
            if self
                .fail_once
                .as_ref()
                .is_some_and(|fail_once| fail_once.replace(false))
            {
                return Err(ScriptError::new("fail once"));
            }
            Ok(ScriptValue::Nil)
        }

        fn get_input(&self, _name: &str) -> Result<ScriptValue, ScriptError> {
            Ok(ScriptValue::Nil)
        }

        fn set_input(&mut self, _name: &str, _value: ScriptValue) -> Result<(), ScriptError> {
            Ok(())
        }
    }

    impl ScriptInstance for AdvanceAndUpdateScriptInstance {
        fn has_method(&self, method: ScriptMethod) -> Result<bool, ScriptError> {
            Ok(matches!(
                method,
                ScriptMethod::Advance | ScriptMethod::Update
            ))
        }

        fn call_method(
            &mut self,
            method: ScriptMethod,
            _args: &[ScriptValue],
            _host: &mut dyn crate::ScriptHost,
        ) -> Result<ScriptValue, ScriptError> {
            match method {
                ScriptMethod::Advance => {
                    self.advances.set(self.advances.get() + 1);
                    Ok(ScriptValue::Bool(true))
                }
                ScriptMethod::Update => {
                    self.updates.set(self.updates.get() + 1);
                    Ok(ScriptValue::Nil)
                }
                _ => unreachable!("only declared script methods are called"),
            }
        }

        fn get_input(&self, _name: &str) -> Result<ScriptValue, ScriptError> {
            Ok(ScriptValue::Nil)
        }

        fn set_input(&mut self, _name: &str, _value: ScriptValue) -> Result<(), ScriptError> {
            Ok(())
        }
    }

    impl ScriptInstance for FailOnceAdvanceScriptInstance {
        fn has_method(&self, method: ScriptMethod) -> Result<bool, ScriptError> {
            Ok(method == ScriptMethod::Advance)
        }

        fn call_method(
            &mut self,
            method: ScriptMethod,
            args: &[ScriptValue],
            _host: &mut dyn crate::ScriptHost,
        ) -> Result<ScriptValue, ScriptError> {
            assert_eq!(method, ScriptMethod::Advance);
            let seconds = args
                .first()
                .and_then(ScriptValue::as_number)
                .map(|value| value as f32)
                .expect("advance receives seconds");
            self.attempts.borrow_mut().push(seconds);
            if self.should_fail.replace(false) {
                return Err(ScriptError::new("fail once"));
            }
            Ok(ScriptValue::Bool(true))
        }

        fn get_input(&self, _name: &str) -> Result<ScriptValue, ScriptError> {
            Ok(ScriptValue::Nil)
        }

        fn set_input(&mut self, _name: &str, _value: ScriptValue) -> Result<(), ScriptError> {
            Ok(())
        }
    }

    impl ScriptInstance for UpdateScriptInstance {
        fn has_method(&self, method: ScriptMethod) -> Result<bool, ScriptError> {
            Ok(matches!(method, ScriptMethod::Init | ScriptMethod::Update))
        }

        fn call_method(
            &mut self,
            method: ScriptMethod,
            _args: &[ScriptValue],
            _host: &mut dyn crate::ScriptHost,
        ) -> Result<ScriptValue, ScriptError> {
            match method {
                ScriptMethod::Init => self.inits.set(self.inits.get() + 1),
                ScriptMethod::Update => self.updates.set(self.updates.get() + 1),
                _ => {}
            }
            Ok(ScriptValue::Nil)
        }

        fn get_input(&self, _name: &str) -> Result<ScriptValue, ScriptError> {
            Ok(ScriptValue::Nil)
        }

        fn set_input(&mut self, _name: &str, _value: ScriptValue) -> Result<(), ScriptError> {
            Ok(())
        }
    }

    fn synthetic_instance(
        components: Vec<RuntimeComponent>,
        update_order: Vec<usize>,
    ) -> ArtboardInstance {
        let slots = components
            .iter()
            .map(|component| InstanceSlot {
                local_id: component.local_id,
                source_global_id: component.global_id,
                type_name: Some(component.type_name),
                name: None,
            })
            .collect::<Vec<_>>();
        let mut runtime_objects = vec![None; slots.len()];
        for component in &components {
            if component.local_id >= runtime_objects.len() {
                runtime_objects.resize(component.local_id + 1, None);
            }
            runtime_objects[component.local_id] = Some(synthetic_runtime_object(
                component.global_id,
                component.type_name,
                Vec::new(),
            ));
        }
        let mut objects = InstanceObjectArena::from_runtime_objects(runtime_objects);
        for component in components {
            let local_id = component.local_id;
            objects
                .attach_component(local_id, component)
                .expect("synthetic component occurrence must exist once");
        }
        let dependency_handles = update_order
            .iter()
            .filter_map(|local_id| objects.component_handle(*local_id))
            .collect();
        objects.set_dependency_order(dependency_handles);
        if let Some(root) = objects.root()
            && let Some(component) = objects.component_mut(root)
        {
            // Synthetic Artboards model the inherited C++ root Component:
            // Components is loop-control dirt on that same owner, alongside
            // whatever concrete dirt the test supplied.
            component.dirt |= ComponentDirt::COMPONENTS;
        }
        let advancing_components = slots
            .iter()
            .filter_map(|slot| {
                let kind = match slot.type_name? {
                    "Artboard" => AdvancingComponentKind::Artboard,
                    "NestedArtboard" | "NestedArtboardLeaf" | "NestedArtboardLayout" => {
                        AdvancingComponentKind::NestedArtboard
                    }
                    "LayoutComponent" => AdvancingComponentKind::LayoutComponent,
                    "ArtboardComponentList" => AdvancingComponentKind::ArtboardComponentList,
                    "ScrollConstraint" => AdvancingComponentKind::ScrollConstraint,
                    "TextInput" => AdvancingComponentKind::TextInput,
                    "ScriptedDataConverter" => AdvancingComponentKind::ScriptedDataConverter,
                    "ScriptedDrawable" => AdvancingComponentKind::ScriptedDrawable,
                    "ScriptedLayout" => AdvancingComponentKind::ScriptedLayout,
                    "ScriptedPathEffect" => AdvancingComponentKind::ScriptedPathEffect,
                    _ => return None,
                };
                Some(RuntimeAdvancingComponent {
                    local_id: slot.local_id,
                    object: objects.object_handle(slot.local_id)?,
                    component: objects.component_handle(slot.local_id),
                    kind,
                })
            })
            .collect();
        let resetting_components = slots
            .iter()
            .filter_map(|slot| {
                let kind = match slot.type_name? {
                    "NestedArtboard" | "NestedArtboardLeaf" | "NestedArtboardLayout" => {
                        ResettingComponentKind::NestedArtboard
                    }
                    "ArtboardComponentList" => ResettingComponentKind::ArtboardComponentList,
                    "CustomPropertyTrigger" => ResettingComponentKind::CustomPropertyTrigger,
                    _ => return None,
                };
                Some(RuntimeResettingComponent {
                    local_id: slot.local_id,
                    component: objects.component_handle(slot.local_id)?,
                    kind,
                })
            })
            .collect();
        let component_lists = objects
            .component_handles()
            .iter()
            .copied()
            .filter(|handle| {
                objects
                    .component(*handle)
                    .is_some_and(|component| component.concrete.constrainable_list.is_some())
            })
            .collect();

        let text_affecting_locals = build_text_affecting_locals(&slots, &objects);
        let solid_color_paint_revisions = vec![
            1;
            slots
                .iter()
                .map(|slot| slot.local_id)
                .max()
                .map_or(0, |local_id| local_id.saturating_add(1))
        ];
        let instance_identity = RuntimeArtboardInstanceIdentity::next();
        ArtboardInstance {
            audio_event_playback: RuntimeAudioEventPlayback::empty(crate::AudioArtboardId(
                instance_identity.0,
            )),
            instance_identity,
            audio_lifecycle_armed: true,
            width: 0.0,
            height: 0.0,
            origin_x: 0.0,
            origin_y: 0.0,
            clip: true,
            host_opacity: 1.0,
            frame_origin: Cell::new(true),
            frame_id: Cell::new(0),
            slots,
            objects,
            joysticks: Vec::new(),
            advancing_components,
            #[cfg(test)]
            persistent_dirt_component_fixture: None,
            #[cfg(test)]
            update_pass_data_bind_call_count: 0,
            resetting_components,
            component_lists,
            component_list_resource_pools: RuntimeComponentListResourcePools::default(),
            joysticks_apply_before_update: true,
            linear_animations: Arc::new(Vec::new()),
            shared_scripted_interpolators: RefCell::new(RuntimeScriptedInterpolatorState::default()),
            scripted_interpolator_factories: BTreeMap::new(),
            empty_linear_animation: Arc::new(RuntimeLinearAnimation::empty()),
            state_machines: Arc::new(Vec::new()),
            script_instances_by_global: RuntimeScriptState::default(),
            script_attachment_generation: 0,
            scripted_data_converter_instances_by_global: RuntimeScriptState::default(),
            has_scripted_drawables: false,
            nested_script_owned_contexts: BTreeMap::new(),
            script_update_error: None,
            external_focus_domain: None,
            nested_artboards: RuntimeNestedArtboards::default(),
            active_nested_state_machines: BTreeMap::new(),
            nested_artboard_locals: Vec::new(),
            newly_uncollapsed_nested_artboards: BTreeSet::new(),
            graph_global_id: 0,
            profile_name: String::new(),
            profile_path: Vec::new(),
            build_context: None,
            nested_context_source_tree_cache: Cell::new(None),
            nested_layout_bounds: None,
            artboard_data_bind_values: BTreeMap::new(),
            artboard_formula_random_source: RuntimeDataBindGraphFormulaRandomSource::default(),
            artboard_owned_view_model_context: None,
            artboard_owned_data_context: None,
            artboard_owned_view_model_handle: None,
            artboard_authored_data_bind_states: RuntimeArtboardAuthoredDataBindStates::default(),
            artboard_owned_view_model_rebind_sink: crate::view_model_cell::RuntimeCellDirtSink::new(
            ),
            artboard_property_bindings: Vec::new(),
            artboard_image_asset_bindings: Vec::new(),
            artboard_data_bind_target_queues: RuntimeArtboardDataBindTargetQueues::default(),
            artboard_data_bind_source_queues: RuntimeArtboardDataBindSourceQueues::default(),
            artboard_retained_subordinate_converter_operands: Vec::new(),
            artboard_custom_property_bindings: Vec::new(),
            artboard_layout_computed_bindings: Vec::new(),
            artboard_numeric_source_bindings: Vec::new(),
            artboard_formula_token_bindings: RuntimeArtboardFormulaTokenBindingStates::default(),
            artboard_converter_property_bindings: Vec::new(),
            artboard_solo_bindings: Vec::new(),
            artboard_solo_source_bindings: Vec::new(),
            artboard_nested_host_bindings: Vec::new(),
            artboard_list_bindings: Vec::new(),
            artboard_text_list_bindings: Vec::new(),
            runtime_list_paths: RefCell::new(Vec::new()),
            artboard_context_source_values_scratch: Vec::new(),
            artboard_nested_child_context_updates_scratch: Vec::new(),
            stateful_nested_view_model_contexts_dirty: true,
            stateful_nested_view_model_dirty_locals: BTreeSet::new(),
            image_asset_overrides: BTreeMap::new(),
            image_render_overrides: BTreeMap::new(),
            text_style_font_overrides: BTreeMap::new(),
            text_style_feature_options: RefCell::new(BTreeMap::new()),
            text_variation_modifier_tags: RefCell::new(BTreeMap::new()),
            runtime_images: crate::draw::image::RuntimeImageList::default(),
            external_font_assets: Arc::new(BTreeMap::new()),
            runtime_font_assets: Arc::new(crate::RuntimeFontAssetOwners::default()),
            runtime_font_asset_snapshots: BTreeMap::new(),
            runtime_font_asset_referencer: Rc::new(Default::default()),
            runtime_image_assets: RefCell::new(None),
            runtime_image_asset_referencer: Rc::new(Default::default()),
            render_resources: RefCell::new(crate::draw::RuntimeOccurrenceRenderResources::default()),
            geometry_state: RefCell::new(crate::draw::RuntimeGeometryState::default()),
            dirt_depth: 0,
            cache_epoch: 1,
            prepared_epoch: 1,
            path_epoch: 1,
            layout_revision: 1,
            text_shape_revision: 1,
            text_affecting_locals,
            solid_color_paint_revisions,
            runtime_drawables: RuntimeDrawableList::default(),
            runtime_shapes: RuntimeShapeList::default(),
            runtime_clipping_shapes: RuntimeClippingShapeList::default(),
            runtime_meshes: crate::draw::RuntimeMeshList::default(),
            did_change: Cell::new(true),
            semantic_bounds_dirty_locals: BTreeSet::new(),
            layout_constraint_bounds_enabled: false,
            layout_constraint_bounds: None,
            solved_layout_bounds: None,
        }
    }

    fn synthetic_nested_artboard_instance(graph_global_id: u32) -> RuntimeNestedArtboardInstance {
        let mut child = synthetic_instance(Vec::new(), Vec::new());
        child.graph_global_id = graph_global_id;
        RuntimeNestedArtboardInstance {
            child: Box::new(child),
            render_cache_revision: 0,
            render_resources: RefCell::new(crate::draw::RuntimeOccurrenceRenderResources::default()),
            initial_layout_paint_frame: RefCell::new(None),
            layout_data_transferred: false,
            layout_data_transfer_key: None,
            data_bind_path_ids: None,
            data_bind_path_is_relative: false,
            stateful_view_model_instance_local: None,
            stateful_view_model_instance_locals_by_id: BTreeMap::new(),
            stateful_view_model_context: None,
            stateful_global_view_model_contexts: BTreeMap::new(),
            data_bind_property_source_locals: Vec::new(),
            data_bind_image_source_locals: Vec::new(),
            data_bind_context_source_locals_by_path: BTreeMap::new(),
            animations: Vec::new(),
            is_paused: false,
            speed: 1.0,
            quantize: -1.0,
            cumulated_seconds: 0.0,
        }
    }

    #[test]
    fn layout_fit_leaf_resizes_its_mounted_artboard_from_the_parent_layout_frame() {
        let mut root = synthetic_component_for_type(0, "Artboard");
        root.dirt = ComponentDirt::COMPONENTS;
        let mut layout = synthetic_component_for_type(1, "LayoutComponent");
        layout.dirt = ComponentDirt::COMPONENTS;
        let mut leaf = synthetic_component_for_type(2, "NestedArtboardLeaf");
        leaf.dirt = ComponentDirt::WORLD_TRANSFORM;
        let mut instance = synthetic_instance(vec![root, layout, leaf], vec![0, 1, 2]);
        synthetic_link_parent(&mut instance, 1, 0);
        synthetic_link_parent(&mut instance, 2, 1);

        let bounds = Arc::new(BTreeMap::from([(
            1,
            RuntimeLayoutBounds {
                x: 11.0,
                y: 13.0,
                width: 120.0,
                height: 80.0,
            },
        )]));
        instance.layout_constraint_bounds_enabled = true;
        instance.layout_constraint_bounds = Some(Arc::clone(&bounds));
        instance.solved_layout_bounds = Some(bounds);

        let mut child =
            synthetic_instance(vec![synthetic_component_for_type(0, "Artboard")], vec![0]);
        child.set_artboard_dimensions(10.0, 20.0);
        child.update_pass_data_bind_call_count = 0;
        let mut nested = synthetic_nested_artboard_instance(7);
        nested.child = Box::new(child);
        instance.nested_artboards.insert(2, nested);
        instance.nested_artboard_locals.push(2);

        let fit_key = property_key_for_name("NestedArtboardLeaf", "fit").unwrap();
        assert!(instance.set_uint_property(2, fit_key, 7));
        assert!(
            instance
                .component(2)
                .is_some_and(|component| component.dirt.contains(ComponentDirt::WORLD_TRANSFORM))
        );

        instance.update_pass();

        let child = &instance.nested_artboards[&2].child;
        assert_eq!(child.artboard_dimensions(), (120.0, 80.0));
        assert_eq!(
            child.update_pass_data_bind_call_count, 2,
            "NestedArtboard::update performs the ordinary mounted-child pass, then the resized \
             layout-fit leaf performs exactly one same-frame reflow pass"
        );
    }

    #[test]
    fn nested_artboards_preserve_sorted_iteration_and_sparse_lookup_after_edits() {
        let mut nested_artboards = RuntimeNestedArtboards::default();
        nested_artboards.insert(9, synthetic_nested_artboard_instance(90));
        nested_artboards.insert(2, synthetic_nested_artboard_instance(20));
        nested_artboards.insert(5, synthetic_nested_artboard_instance(50));

        assert_eq!(
            nested_artboards.keys().copied().collect::<Vec<_>>(),
            [2, 5, 9]
        );
        assert_eq!(nested_artboards.get(&5).unwrap().child.graph_global_id, 50);

        let replaced = nested_artboards
            .insert(5, synthetic_nested_artboard_instance(51))
            .expect("existing local is replaced");
        assert_eq!(replaced.child.graph_global_id, 50);
        assert_eq!(nested_artboards.get(&5).unwrap().child.graph_global_id, 51);

        let removed = nested_artboards.remove(&2).expect("local is removed");
        assert_eq!(removed.child.graph_global_id, 20);
        assert!(nested_artboards.get(&2).is_none());
        assert_eq!(nested_artboards.keys().copied().collect::<Vec<_>>(), [5, 9]);
        assert_eq!(nested_artboards.get(&9).unwrap().child.graph_global_id, 90);
    }

    #[test]
    fn artboard_audio_engine_and_volume_propagate_to_nested_occurrences() {
        let mut root = synthetic_instance(Vec::new(), Vec::new());
        root.nested_artboards
            .insert(2, synthetic_nested_artboard_instance(20));
        let engine = crate::AudioEngine::new(2, 44_100).expect("headless engine");

        root.set_audio_engine(Some(engine));
        root.set_volume(0.125);

        let child = &root.nested_artboards.get(&2).expect("nested child").child;
        assert_eq!(root.volume(), 0.125);
        assert_eq!(child.volume(), 0.125);
        assert_eq!(
            child.audio_engine().expect("propagated engine").channels(),
            2
        );
    }

    fn authoring_record(type_name: &str, properties: Vec<AuthoringProperty>) -> AuthoringRecord {
        AuthoringRecord {
            type_key: definition_by_name(type_name)
                .unwrap_or_else(|| panic!("missing schema definition {type_name}"))
                .type_key
                .int,
            properties,
        }
    }

    fn authoring_property(
        type_name: &str,
        property_name: &str,
        value: AuthoringValue,
    ) -> AuthoringProperty {
        AuthoringProperty {
            key: property_key_for_name(type_name, property_name)
                .unwrap_or_else(|| panic!("missing property {type_name}.{property_name}")),
            value,
        }
    }

    #[test]
    fn component_list_context_match_requires_the_same_shared_graph() {
        let bytes = synthetic_riv(9621, |bytes| {
            push_synthetic_object(bytes, "ViewModel", &[]);
            push_synthetic_object(bytes, "Backboard", &[]);
        });
        let file = read_runtime_file(&bytes).expect("synthetic view model should import");
        let instance = RuntimeOwnedViewModelInstance::new(&file, 0)
            .expect("synthetic view model should instantiate");
        let retained = RuntimeOwnedViewModelHandle::new(instance);
        let same_graph = retained.clone();
        let forked_graph = RuntimeOwnedViewModelHandle::new(retained.borrow().clone());
        assert_eq!(
            retained.borrow().instance_identity(),
            forked_graph.borrow().instance_identity(),
            "the payload clone deliberately preserves logical instance identity"
        );

        let row = RuntimeComponentListItemInstance {
            child: Box::new(synthetic_instance(Vec::new(), Vec::new())),
            render_resources: RefCell::new(crate::draw::RuntimeOccurrenceRenderResources::default()),
            state_machines: Vec::new(),
            context_rebind_sink: {
                let sink = crate::view_model_cell::RuntimeCellDirtSink::new();
                retained.add_rebind_dependent(&sink);
                sink
            },
            draw_index_sink: None,
            context: retained,
            occurrence_identity: 1,
            logical_index: 0,
            settled_layout_size: Cell::new(None),
            transform: Mat2D::IDENTITY,
            render_cache_revision: 1,
        };

        assert!(component_list_contexts_retain_same_handles(
            std::slice::from_ref(&row),
            std::slice::from_ref(&same_graph),
        ));
        assert!(!component_list_contexts_retain_same_handles(
            std::slice::from_ref(&row),
            std::slice::from_ref(&forked_graph),
        ));
    }

    fn empty_state_machine(global_id: u32) -> RuntimeStateMachine {
        RuntimeStateMachine {
            global_id,
            name: None,
            default_view_model_index: None,
            inputs: Arc::new(Vec::new()),
            listeners: Arc::new(Vec::new()),
            layers: Arc::new(Vec::new()),
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
            scripted_objects: Vec::new(),
            scripted_listener_actions: Vec::new(),
            scripted_object_bindings: Vec::new(),
            action_owners: crate::state_machine::RuntimeActionCoreArena::empty(),
        }
    }

    fn nested_audio_event(local_index: usize) -> (StateMachineReportedEvent, u32) {
        let file = RuntimeFile::from_authoring_records(vec![
            authoring_record("Backboard", Vec::new()),
            authoring_record("Artboard", Vec::new()),
            authoring_record(
                "AudioEvent",
                vec![authoring_property(
                    "AudioEvent",
                    "parentId",
                    AuthoringValue::Uint(0),
                )],
            ),
        ])
        .expect("import nested AudioEvent fixture");
        let event = file
            .objects
            .iter()
            .flatten()
            .find(|object| object.type_name == "AudioEvent")
            .expect("AudioEvent fixture object");
        (
            StateMachineReportedEvent::from_runtime_event(local_index, event),
            u32::from(event.type_key),
        )
    }

    fn nested_event_source(
        graph_global_id: u32,
        event: StateMachineReportedEvent,
        local_phase: &'static str,
        audio_phase: &'static str,
        total_order: &Rc<RefCell<Vec<&'static str>>>,
    ) -> RuntimeNestedArtboardInstance {
        let mut child =
            synthetic_instance(vec![synthetic_component_for_type(0, "Artboard")], vec![0]);
        child.graph_global_id = graph_global_id;
        child.state_machines = Arc::new(vec![empty_state_machine(graph_global_id)]);
        let definition = Arc::clone(&child.state_machines);
        let mut machine = StateMachineInstance::new(0, &definition[0], &mut child);
        machine.configure_nested_event_source_test(
            local_phase,
            audio_phase,
            Rc::clone(total_order),
            event,
        );
        let mut nested = synthetic_nested_artboard_instance(graph_global_id);
        nested.child = Box::new(child);
        nested
            .animations
            .push(RuntimeNestedAnimationInstance::StateMachine(
                RuntimeNestedStateMachineInstance::new(0, machine, Vec::new()),
            ));
        nested
    }

    fn append_nested_event_source(
        nested: &mut RuntimeNestedArtboardInstance,
        local_id: usize,
        event: StateMachineReportedEvent,
        local_phase: &'static str,
        audio_phase: &'static str,
        total_order: &Rc<RefCell<Vec<&'static str>>>,
    ) {
        let definition = Arc::clone(&nested.child.state_machines);
        let mut machine = StateMachineInstance::new(0, &definition[0], &mut nested.child);
        machine.configure_nested_event_source_test(
            local_phase,
            audio_phase,
            Rc::clone(total_order),
            event,
        );
        nested
            .animations
            .push(RuntimeNestedAnimationInstance::StateMachine(
                RuntimeNestedStateMachineInstance::new(local_id, machine, Vec::new()),
            ));
    }

    #[test]
    fn production_advance_and_apply_dispatches_each_same_host_reporter_chain_atomically() {
        let total_order = Rc::new(RefCell::new(Vec::new()));
        let (event, event_core_type) = nested_audio_event(7);
        let mut nested = synthetic_nested_artboard_instance(101);
        nested.child.state_machines = Arc::new(vec![empty_state_machine(101)]);
        append_nested_event_source(
            &mut nested,
            2,
            event.clone(),
            "A-local",
            "A-audio",
            &total_order,
        );
        append_nested_event_source(&mut nested, 4, event, "B-local", "B-audio", &total_order);

        let mut root = synthetic_instance(
            vec![
                synthetic_component_for_type(0, "Artboard"),
                synthetic_component_for_type(1, "NestedArtboard"),
            ],
            vec![0, 1],
        );
        root.nested_artboards.insert(1, nested);
        root.state_machines = Arc::new(vec![empty_state_machine(100)]);
        let definition = Arc::clone(&root.state_machines);
        let mut root_machine = StateMachineInstance::new(0, &definition[0], &mut root);
        root_machine.configure_nested_event_root_test(
            "root-local",
            "root-audio",
            Rc::clone(&total_order),
            [1],
        );

        root_machine
            .advance_and_apply(&mut root, 0.25)
            .expect("production advance_and_apply frame");

        assert_eq!(
            total_order.borrow().as_slice(),
            [
                "A-local",
                "root-local",
                "root-audio",
                "A-audio",
                "B-local",
                "root-local",
                "root-audio",
                "B-audio",
            ],
            "each animation on one host completes local dispatch, ancestor dispatch, and audio unwind before the next animation advances",
        );
        assert_eq!(
            root_machine.audio_event_seam_receipt(),
            (2, Some((7, event_core_type))),
        );
    }

    #[test]
    fn production_advance_and_apply_dispatches_each_sibling_reporter_chain_atomically() {
        let total_order = Rc::new(RefCell::new(Vec::new()));
        let (event, event_core_type) = nested_audio_event(7);
        let mut root = synthetic_instance(
            vec![
                synthetic_component_for_type(0, "Artboard"),
                synthetic_component_for_type(1, "NestedArtboard"),
                synthetic_component_for_type(2, "NestedArtboard"),
            ],
            vec![0, 1, 2],
        );
        root.nested_artboards.insert(
            1,
            nested_event_source(101, event.clone(), "A-local", "A-audio", &total_order),
        );
        root.nested_artboards.insert(
            2,
            nested_event_source(102, event, "B-local", "B-audio", &total_order),
        );
        root.state_machines = Arc::new(vec![empty_state_machine(100)]);
        let definition = Arc::clone(&root.state_machines);
        let mut root_machine = StateMachineInstance::new(0, &definition[0], &mut root);
        root_machine.configure_nested_event_root_test(
            "root-local",
            "root-audio",
            Rc::clone(&total_order),
            [1, 2],
        );
        root_machine
            .configure_nested_event_settlement_test("root-settled", Rc::clone(&total_order));

        root_machine
            .advance_and_apply(&mut root, 0.25)
            .expect("production advance_and_apply frame");

        assert_eq!(
            total_order.borrow().as_slice(),
            [
                "A-local",
                "root-local",
                "root-audio",
                "A-audio",
                "B-local",
                "root-local",
                "root-audio",
                "B-audio",
            ],
            "each source completes local dispatch, ancestor dispatch, and audio unwind before the next authored component advances",
        );
        assert!(
            !total_order.borrow().contains(&"root-settled"),
            "nested notify performs only updateDataBinds(false), never a full zero-time advance",
        );
        assert_eq!(
            root_machine.audio_event_seam_receipt(),
            (2, Some((7, event_core_type))),
        );
    }

    #[test]
    fn production_ancestor_listener_can_mutate_the_reporting_source_input() {
        let total_order = Rc::new(RefCell::new(Vec::new()));
        let (event, _) = nested_audio_event(7);
        let mut nested = synthetic_nested_artboard_instance(101);
        let mut source_definition = empty_state_machine(101);
        source_definition.inputs = Arc::new(vec![Some(RuntimeStateMachineInput::new_number(
            1,
            Some("source-value".to_owned()),
            0.0,
        ))]);
        nested.child.state_machines = Arc::new(vec![source_definition]);
        append_nested_event_source(
            &mut nested,
            2,
            event,
            "source-local",
            "source-audio",
            &total_order,
        );

        let mut root = synthetic_instance(
            vec![
                synthetic_component_for_type(0, "Artboard"),
                synthetic_component_for_type(1, "NestedArtboard"),
                synthetic_component_for_type(3, "NestedNumber"),
            ],
            vec![0, 1, 3],
        );
        let parent_id_key =
            property_key_for_name("Component", "parentId").expect("Component.parentId");
        let input_id_key =
            property_key_for_name("NestedInput", "inputId").expect("NestedInput.inputId");
        assert!(root.objects.set_uint_property(3, parent_id_key, 2));
        assert!(root.objects.set_uint_property(3, input_id_key, 0));
        root.nested_artboards.insert(1, nested);
        let mut root_definition = empty_state_machine(100);
        root_definition.listeners = Arc::new(vec![RuntimeStateMachineListener {
            name: None,
            target_local_id: 1,
            is_single: true,
            listener_types: vec![RuntimeListenerType::Event],
            event_local_indices: vec![7],
            view_model_path: None,
            view_model_input_types: Vec::new(),
            gamepad_input_types: Vec::new(),
            keyboard_input_types: Vec::new(),
            semantic_input_types: Vec::new(),
            hit_paths: Vec::new(),
            listener_actions: vec![RuntimeScheduledListenerAction::NumberChange(
                RuntimeListenerNumberChange::for_test(
                    0,
                    RuntimeListenerInputTarget {
                        direct_input_index: None,
                        nested_input_local_id: Some(3),
                    },
                    9.0,
                ),
            )],
        }]);
        root.state_machines = Arc::new(vec![root_definition]);
        let definitions = Arc::clone(&root.state_machines);
        let mut root_machine = StateMachineInstance::new(0, &definitions[0], &mut root);
        root_machine.configure_nested_event_root_test("root-local", "root-audio", total_order, [1]);

        root_machine
            .advance_and_apply(&mut root, 0.25)
            .expect("source-addressability frame");

        assert_eq!(
            root.nested_state_machine(2)
                .and_then(|machine| machine.input(0))
                .and_then(StateMachineInputInstance::number_value),
            Some(9.0),
            "the reporting source stays addressable throughout synchronous ancestor delivery",
        );
    }

    #[test]
    fn production_advance_and_apply_flushes_prior_source_audio_before_later_script_error() {
        let total_order = Rc::new(RefCell::new(Vec::new()));
        let (event, event_core_type) = nested_audio_event(7);
        let mut root = synthetic_instance(
            vec![
                synthetic_component_for_type(0, "Artboard"),
                synthetic_component_for_type(1, "NestedArtboard"),
                synthetic_component_for_type(2, "ScriptedDrawable"),
            ],
            vec![0, 1, 2],
        );
        root.nested_artboards.insert(
            1,
            nested_event_source(101, event, "A-local", "A-audio", &total_order),
        );
        root.set_script_instance_for_global(
            2,
            Box::new(FailOnceAdvanceScriptInstance {
                attempts: Rc::new(RefCell::new(Vec::new())),
                should_fail: Rc::new(Cell::new(true)),
            }),
        );
        root.state_machines = Arc::new(vec![empty_state_machine(100)]);
        let definition = Arc::clone(&root.state_machines);
        let mut root_machine = StateMachineInstance::new(0, &definition[0], &mut root);
        root_machine.configure_nested_event_root_test(
            "root-local",
            "root-audio",
            Rc::clone(&total_order),
            [1],
        );

        root_machine
            .advance_and_apply(&mut root, 0.25)
            .expect_err("later scripted component fails");

        assert_eq!(
            total_order.borrow().as_slice(),
            ["A-local", "root-local", "root-audio", "A-audio"],
            "the earlier reporter's deferred audio is flushed before propagating the later ScriptError",
        );
        let nested_machine = root
            .nested_state_machine(0)
            .expect("nested source state machine");
        assert_eq!(
            nested_machine.audio_event_seam_receipt(),
            (1, Some((7, event_core_type))),
        );
    }

    fn deep_nested_event_topology(
        fail_after_leaf: bool,
    ) -> (
        ArtboardInstance,
        StateMachineInstance,
        Rc<RefCell<Vec<&'static str>>>,
        u32,
    ) {
        let total_order = Rc::new(RefCell::new(Vec::new()));
        let (event, event_core_type) = nested_audio_event(7);

        let mut middle_components = vec![
            synthetic_component_for_type(0, "Artboard"),
            synthetic_component_for_type(1, "NestedArtboard"),
        ];
        let mut middle_order = vec![0, 1];
        if fail_after_leaf {
            middle_components.push(synthetic_component_for_type(2, "ScriptedDrawable"));
            middle_order.push(2);
        }
        let mut middle = synthetic_instance(middle_components, middle_order);
        middle.nested_artboards.insert(
            1,
            nested_event_source(102, event.clone(), "leaf-local", "leaf-audio", &total_order),
        );
        if fail_after_leaf {
            middle.set_script_instance_for_global(
                2,
                Box::new(FailOnceAdvanceScriptInstance {
                    attempts: Rc::new(RefCell::new(Vec::new())),
                    should_fail: Rc::new(Cell::new(true)),
                }),
            );
        }
        middle.state_machines = Arc::new(vec![empty_state_machine(101)]);
        let middle_definitions = Arc::clone(&middle.state_machines);
        let mut middle_owner = StateMachineInstance::new(0, &middle_definitions[0], &mut middle);
        middle_owner.configure_nested_event_forwarder_test(
            "middle-local",
            "middle-audio",
            Rc::clone(&total_order),
            1,
            event,
        );

        let mut outer = synthetic_nested_artboard_instance(101);
        outer.child = Box::new(middle);
        outer
            .animations
            .push(RuntimeNestedAnimationInstance::StateMachine(
                RuntimeNestedStateMachineInstance::new(2, middle_owner, Vec::new()),
            ));

        let mut root = synthetic_instance(
            vec![
                synthetic_component_for_type(0, "Artboard"),
                synthetic_component_for_type(1, "NestedArtboard"),
            ],
            vec![0, 1],
        );
        root.nested_artboards.insert(1, outer);
        root.state_machines = Arc::new(vec![empty_state_machine(100)]);
        let root_definitions = Arc::clone(&root.state_machines);
        let mut root_machine = StateMachineInstance::new(0, &root_definitions[0], &mut root);
        root_machine.configure_nested_event_root_test(
            "root-local",
            "root-audio",
            Rc::clone(&total_order),
            [1],
        );
        (root, root_machine, total_order, event_core_type)
    }

    #[test]
    fn production_deep_nested_owner_chain_reaches_root_before_subtree_continuation() {
        let (mut root, mut root_machine, total_order, event_core_type) =
            deep_nested_event_topology(false);

        root_machine
            .advance_and_apply(&mut root, 0.25)
            .expect("three-level nested event frame");

        assert_eq!(
            total_order.borrow().as_slice(),
            [
                "leaf-local",
                "middle-local",
                "root-local",
                "root-audio",
                "middle-audio",
                "leaf-audio",
            ],
            "the intermediate owner completes the report to the root and audio unwinds before subtree continuation",
        );
        assert_eq!(
            root_machine.audio_event_seam_receipt(),
            (1, Some((7, event_core_type))),
        );
    }

    #[test]
    fn production_deep_nested_owner_chain_survives_later_subtree_script_error() {
        let (mut root, mut root_machine, total_order, event_core_type) =
            deep_nested_event_topology(true);

        root_machine
            .advance_and_apply(&mut root, 0.25)
            .expect_err("the scripted sibling after the leaf fails");

        assert_eq!(
            total_order.borrow().as_slice(),
            [
                "leaf-local",
                "middle-local",
                "root-local",
                "root-audio",
                "middle-audio",
                "leaf-audio",
            ],
            "the completed deep chain remains delivered through every audio tail before the later error propagates",
        );
        assert_eq!(
            root_machine.audio_event_seam_receipt(),
            (1, Some((7, event_core_type))),
        );
        assert!(
            root.nested_artboards.contains_key(&1),
            "the active nested occurrence is restored before ScriptError propagation",
        );
    }

    #[test]
    fn state_machine_instance_retains_the_cpp_authored_definition_owner() {
        let mut artboard = synthetic_instance(Vec::new(), Vec::new());
        artboard.state_machines = Arc::new(vec![empty_state_machine(11)]);

        let instance = artboard.state_machine_instance(0).expect("state machine");
        let retained_owner = instance
            .retained_state_machine_definitions()
            .expect("artboard-created instances retain the definition");
        let retained = retained_owner.get(0).expect("retained definition");
        let authored = artboard.state_machine(0).expect("authored definition");

        // C++ stores this exact authored `StateMachine*` on the instance
        // (`state_machine_instance.hpp:123,386`;
        // `state_machine_instance.cpp:1707-1711`).
        assert!(std::ptr::eq(retained, authored));
    }

    #[test]
    fn each_state_machine_instance_dirties_each_hit_shape_once() {
        let shape = synthetic_component_for_type(0, "Shape");
        let mut artboard = synthetic_instance(vec![shape], vec![0]);
        let listener = RuntimeStateMachineListener {
            name: None,
            target_local_id: 0,
            is_single: false,
            listener_types: vec![RuntimeListenerType::Down],
            event_local_indices: Vec::new(),
            view_model_path: None,
            view_model_input_types: Vec::new(),
            gamepad_input_types: Vec::new(),
            keyboard_input_types: Vec::new(),
            semantic_input_types: Vec::new(),
            hit_paths: Vec::new(),
            listener_actions: Vec::new(),
        };
        let mut definition = empty_state_machine(12);
        definition.listeners = Arc::new(vec![listener.clone(), listener]);
        artboard.state_machines = Arc::new(vec![definition]);
        assert!(artboard.update_components().did_update);

        let first_cache_epoch = artboard.cache_epoch();
        artboard.state_machine_instance(0).expect("first instance");
        assert!(
            artboard
                .component(0)
                .expect("shape")
                .dirt
                .contains(ComponentDirt::PATH),
            "HitExpandable construction immediately executes Shape::addDirt(Path,true), while its instance-local hitLookup deduplicates duplicate Shape listeners (`state_machine_instance.cpp:1651-1666`)"
        );
        assert!(
            artboard
                .component(0)
                .and_then(|component| component.concrete.shape.as_ref())
                .expect("shape state")
                .is_flagged(crate::components::RuntimeShapeState::NEVER_DEFER_UPDATE),
            "listener initialization sets neverDeferUpdate with the same HitExpandable owner before returning the instance (`state_machine_instance.cpp:1651-1661,1827-1831`)"
        );
        assert!(artboard.update_components().did_update);
        assert!(artboard.cache_epoch() > first_cache_epoch);

        let second_cache_epoch = artboard.cache_epoch();
        artboard.state_machine_instance(0).expect("second instance");
        assert!(
            artboard
                .component(0)
                .expect("shape")
                .dirt
                .contains(ComponentDirt::PATH),
            "a fresh C++ StateMachineInstance owns a fresh hitLookup and dirties the already-flagged Shape again"
        );
        assert!(artboard.update_components().did_update);
        assert!(
            artboard.cache_epoch() > second_cache_epoch,
            "every StateMachineInstance creation executes Shape::addDirt(Path,true) (`state_machine_instance.cpp:1651-1666,1827-1831`)"
        );
    }

    fn direct_input_blend_state_machine(global_id: u32) -> RuntimeStateMachine {
        let mut state_machine = empty_state_machine(global_id);
        state_machine.inputs = Arc::new(vec![Some(RuntimeStateMachineInput::new_number(
            1,
            Some("blend".to_owned()),
            0.0,
        ))]);
        state_machine.layers = Arc::new(vec![RuntimeStateMachineLayer {
            global_id: 2,
            name: None,
            states: vec![RuntimeLayerState {
                global_id: Some(3),
                type_name: Some("BlendState1DInput"),
                animation: None,
                blend_state_1d: Some(RuntimeBlendState1D {
                    source: RuntimeBlendState1DSource::Input {
                        input_index: Some(0),
                    },
                    animations: Vec::new(),
                }),
                blend_state_direct: None,
                speed: 1.0,
                flags: 0,
                fire_actions: Vec::new(),
                listener_actions: Vec::new(),
                transitions: Vec::new(),
            }],
            entry_state_index: Some(0),
            any_state_index: None,
            exit_state_index: None,
        }]);
        state_machine
    }

    #[test]
    fn ordinary_direct_input_blend_accepts_unconditional_outer_state_probes() {
        let definition = direct_input_blend_state_machine(11);
        let mut artboard = synthetic_instance(Vec::new(), Vec::new());
        let mut state_machine = StateMachineInstance::new(0, &definition, &mut artboard);
        artboard.state_machines = Arc::new(vec![definition]);

        assert!(artboard.advance_state_machine_instance(&mut state_machine, 0.0));
        assert!(state_machine.needs_advance());
        assert!(!artboard.try_change_state_machine_instance(&mut state_machine));
        assert!(!artboard.try_change_state_machine_instance(&mut state_machine));
    }

    #[test]
    fn nested_host_input_write_is_visible_to_unconditional_outer_state_probe() {
        let mut definition = empty_state_machine(11);
        definition.inputs = Arc::new(vec![
            Some(RuntimeStateMachineInput::new_bool(
                1,
                Some("enabled".to_owned()),
                false,
            )),
            Some(RuntimeStateMachineInput::new_number(
                2,
                Some("amount".to_owned()),
                0.0,
            )),
            Some(RuntimeStateMachineInput::new_trigger(
                3,
                Some("fire".to_owned()),
            )),
        ]);
        let mut nested = synthetic_nested_artboard_instance(22);
        let bool_state_machine = StateMachineInstance::new(0, &definition, &mut nested.child);
        let number_state_machine = StateMachineInstance::new(0, &definition, &mut nested.child);
        let trigger_state_machine = StateMachineInstance::new(0, &definition, &mut nested.child);
        nested.child.state_machines = Arc::new(vec![definition]);
        nested.animations.extend([
            RuntimeNestedAnimationInstance::StateMachine(RuntimeNestedStateMachineInstance::new(
                7,
                bool_state_machine,
                Vec::new(),
            )),
            RuntimeNestedAnimationInstance::StateMachine(RuntimeNestedStateMachineInstance::new(
                8,
                number_state_machine,
                Vec::new(),
            )),
            RuntimeNestedAnimationInstance::StateMachine(RuntimeNestedStateMachineInstance::new(
                9,
                trigger_state_machine,
                Vec::new(),
            )),
        ]);
        let nested_bool = synthetic_component_for_type(0, "NestedBool");
        let nested_number = synthetic_component_for_type(1, "NestedNumber");
        let nested_trigger = synthetic_component_for_type(2, "NestedTrigger");
        let mut parent =
            synthetic_instance(vec![nested_bool, nested_number, nested_trigger], Vec::new());
        let parent_id_key =
            property_key_for_name("Component", "parentId").expect("Component.parentId");
        let input_id_key =
            property_key_for_name("NestedInput", "inputId").expect("NestedInput.inputId");
        for (local_id, state_machine_local_id, input_id) in [(0, 7, 0), (1, 8, 1), (2, 9, 2)] {
            assert!(parent.objects.set_uint_property(
                local_id,
                parent_id_key,
                state_machine_local_id
            ));
            assert!(
                parent
                    .objects
                    .set_uint_property(local_id, input_id_key, input_id)
            );
        }
        parent.nested_artboards.insert(3, nested);

        assert!(parent.set_nested_state_machine_bool(7, 0, true));
        assert!(parent.set_nested_state_machine_number(8, 1, 1.0));
        assert!(
            parent.fire_nested_trigger_input(2),
            "NestedTrigger::fire is a callback and must reach the nested occurrence without a stored-property write"
        );
        assert_eq!(
            parent
                .nested_state_machine_mut(9)
                .expect("mounted nested state machine")
                .input(2)
                .and_then(|input| input.trigger_fired()),
            Some(true)
        );

        let direct_definitions = Arc::new(vec![
            Some(RuntimeStateMachineInput::new_bool(
                101,
                Some("direct bool".to_owned()),
                false,
            )),
            Some(RuntimeStateMachineInput::new_number(
                102,
                Some("direct number".to_owned()),
                0.0,
            )),
            Some(RuntimeStateMachineInput::new_trigger(
                103,
                Some("direct trigger".to_owned()),
            )),
        ]);
        let mut direct_inputs = (0..direct_definitions.len())
            .map(|index| StateMachineInputInstance::new(index, Arc::clone(&direct_definitions)))
            .collect::<Vec<_>>();

        let nested_bool_action = RuntimeListenerBoolChange::for_test(
            0,
            RuntimeListenerInputTarget {
                direct_input_index: Some(0),
                nested_input_local_id: Some(0),
            },
            1,
        );
        let nested_bool_value_key =
            property_key_for_name("NestedBool", "nestedValue").expect("NestedBool.nestedValue");
        let parent_cache_epoch = parent.cache_epoch();
        assert!(
            !nested_bool_action.perform(&mut parent, &mut direct_inputs),
            "the child was independently set true above, so setting true again is an exact no-op"
        );
        assert_eq!(direct_inputs[0].bool_value(), Some(false));
        assert_eq!(
            parent.objects.bool_property(0, nested_bool_value_key),
            Some(false),
            "NestedBoolBase storage is construction-only and is not rewritten by the live setter"
        );
        assert_eq!(
            parent.bool_property(0, nested_bool_value_key),
            Some(true),
            "the virtual getter reads the child SMIBool rather than stale authored storage"
        );
        assert_eq!(
            parent.cache_epoch(),
            parent_cache_epoch,
            "a nested-input listener action dirties only the child state-machine occurrence"
        );
        assert_eq!(
            parent
                .nested_state_machine_mut(7)
                .and_then(|machine| machine.input(0))
                .and_then(StateMachineInputInstance::bool_value),
            Some(true)
        );
        assert!(
            RuntimeListenerBoolChange::for_test(
                0,
                RuntimeListenerInputTarget {
                    direct_input_index: Some(0),
                    nested_input_local_id: Some(0),
                },
                2,
            )
            .perform(&mut parent, &mut direct_inputs)
        );
        assert_eq!(
            parent.objects.bool_property(0, nested_bool_value_key),
            Some(false),
            "toggling the live child must not mutate parent property storage"
        );
        assert_eq!(
            parent
                .nested_state_machine_mut(7)
                .and_then(|machine| machine.input(0))
                .and_then(StateMachineInputInstance::bool_value),
            Some(false)
        );
        assert!(
            !RuntimeListenerBoolChange::for_test(
                0,
                RuntimeListenerInputTarget {
                    direct_input_index: Some(0),
                    nested_input_local_id: Some(99),
                },
                1,
            )
            .perform(&mut parent, &mut direct_inputs)
        );
        assert!(
            !RuntimeListenerBoolChange::for_test(
                0,
                RuntimeListenerInputTarget {
                    direct_input_index: Some(0),
                    nested_input_local_id: Some(1),
                },
                1,
            )
            .perform(&mut parent, &mut direct_inputs)
        );
        assert_eq!(direct_inputs[0].bool_value(), Some(false));

        let nested_number_value_key =
            property_key_for_name("NestedNumber", "nestedValue").expect("NestedNumber.nestedValue");
        for value in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY, 1.0, -0.0] {
            assert!(
                RuntimeListenerNumberChange::for_test(
                    0,
                    RuntimeListenerInputTarget {
                        direct_input_index: Some(1),
                        nested_input_local_id: Some(1),
                    },
                    value,
                )
                .perform(&mut parent, &mut direct_inputs)
            );
            let actual = parent
                .nested_state_machine_mut(8)
                .and_then(|machine| machine.input(1))
                .and_then(StateMachineInputInstance::number_value)
                .expect("nested number value");
            assert_eq!(actual.to_bits(), value.to_bits());
            assert_eq!(
                parent
                    .objects
                    .double_property(1, nested_number_value_key)
                    .expect("authored NestedNumber storage")
                    .to_bits(),
                0.0f32.to_bits(),
                "the live NestedNumber setter must not rewrite parent storage"
            );
            assert_eq!(
                parent
                    .double_property(1, nested_number_value_key)
                    .expect("virtual NestedNumber getter")
                    .to_bits(),
                value.to_bits()
            );
            assert_eq!(direct_inputs[1].number_value(), Some(0.0));
        }

        let nested_trigger_action = RuntimeListenerTriggerChange::for_test(
            0,
            RuntimeListenerInputTarget {
                direct_input_index: Some(2),
                nested_input_local_id: Some(2),
            },
        );
        assert!(
            !nested_trigger_action.perform(&mut parent, &mut direct_inputs),
            "the earlier direct fire remains latched until the child advances"
        );
        {
            let nested = parent
                .nested_artboards
                .get_mut(&3)
                .expect("mounted nested artboard");
            let (animations, child) = (&mut nested.animations, &mut nested.child);
            let occurrence = animations
                .iter_mut()
                .find_map(|animation| match animation {
                    RuntimeNestedAnimationInstance::StateMachine(occurrence)
                        if occurrence.local_id() == 9 =>
                    {
                        Some(occurrence)
                    }
                    _ => None,
                })
                .expect("nested trigger state machine");
            let _ = occurrence.advance(child, 0.0);
        }
        assert!(nested_trigger_action.perform(&mut parent, &mut direct_inputs));
        assert_eq!(direct_inputs[2].trigger_fired(), Some(false));
        {
            let nested = parent
                .nested_artboards
                .get_mut(&3)
                .expect("mounted nested artboard");
            let (animations, child) = (&mut nested.animations, &mut nested.child);
            let occurrence = animations
                .iter_mut()
                .find_map(|animation| match animation {
                    RuntimeNestedAnimationInstance::StateMachine(occurrence)
                        if occurrence.local_id() == 9 =>
                    {
                        Some(occurrence)
                    }
                    _ => None,
                })
                .expect("nested trigger state machine");
            let _ = occurrence.advance(child, 0.0);
        }
        assert!(nested_trigger_action.perform(&mut parent, &mut direct_inputs));
        assert_eq!(direct_inputs[2].trigger_fired(), Some(false));
    }

    #[test]
    fn nested_state_machine_retains_authored_input_slots_and_skips_initial_trigger() {
        let mut definition = empty_state_machine(11);
        definition.inputs = Arc::new(vec![
            Some(RuntimeStateMachineInput::new_bool(
                1,
                Some("duplicate".to_owned()),
                false,
            )),
            Some(RuntimeStateMachineInput::new_number(
                2,
                Some("duplicate".to_owned()),
                0.0,
            )),
            Some(RuntimeStateMachineInput::new_trigger(
                3,
                Some("fire".to_owned()),
            )),
        ]);
        let mut child = synthetic_instance(Vec::new(), Vec::new());
        let state_machine = StateMachineInstance::new(0, &definition, &mut child);
        let occurrence = RuntimeNestedStateMachineInstance::new(
            7,
            state_machine,
            vec![
                (0, Some("duplicate".to_owned()), Some(true), None),
                (1, Some("duplicate".to_owned()), None, Some(7.5)),
                (2, Some("fire".to_owned()), None, None),
                (99, None, None, None),
            ],
        );

        assert_eq!(occurrence.input_count(), 4);
        assert_eq!(occurrence.input_id_at(0), Some(0));
        assert_eq!(occurrence.input_id_at(3), Some(99));
        assert_eq!(occurrence.input_id_at(4), None);
        assert_eq!(occurrence.input_id_named("duplicate"), Some(0));
        assert_eq!(
            occurrence.input_id_named(""),
            Some(99),
            "an authored nested input with an absent child input has C++'s empty name"
        );
        assert_eq!(
            occurrence
                .state_machine()
                .expect("valid nested state machine")
                .input(0)
                .and_then(|input| input.bool_value()),
            Some(true)
        );
        assert_eq!(
            occurrence
                .state_machine()
                .expect("valid nested state machine")
                .input(1)
                .and_then(|input| input.number_value()),
            Some(7.5)
        );
        assert_eq!(
            occurrence
                .state_machine()
                .expect("valid nested state machine")
                .input(2)
                .and_then(|input| input.trigger_fired()),
            Some(false)
        );
    }

    #[test]
    fn nested_state_machine_forwards_empty_child_results_and_context_lifecycle() {
        let definition = empty_state_machine(11);
        let mut child = synthetic_instance(Vec::new(), Vec::new());
        let state_machine = StateMachineInstance::new(0, &definition, &mut child);
        child.state_machines = Arc::new(vec![definition]);
        let mut occurrence = RuntimeNestedStateMachineInstance::new(7, state_machine, Vec::new());

        assert!(!occurrence.hit_test(&child, 0.0, 0.0));
        assert!(!occurrence.pointer_down(&mut child, 0.0, 0.0, 1));
        assert!(!occurrence.pointer_move(&mut child, 0.0, 0.0, 0.25, 1));
        assert!(!occurrence.pointer_up(&mut child, 0.0, 0.0, 1));
        assert!(!occurrence.pointer_exit(&mut child, 0.0, 0.0, 1));
        assert!(!occurrence.drag_start(&mut child, 0.0, 0.0, 0.25, 1));
        assert!(!occurrence.drag_end(&mut child, 0.0, 0.0, 0.5, 1));
        assert!(!occurrence.try_change_state(&mut child));

        let data_context = RuntimeOwnedDataContext::default();
        assert!(occurrence.bind_owned_data_context(&data_context));
        assert!(occurrence.clear_data_context());
    }

    #[test]
    fn layer_initialization_loop_is_serial_in_authored_order() {
        const STATE_AT_START: u64 = 2 << 1;
        let bytes = synthetic_riv(9597, |bytes| {
            push_synthetic_object(bytes, "Backboard", &[]);
            push_synthetic_object(bytes, "Artboard", &[]);
            push_synthetic_object(bytes, "StateMachine", &[]);
            push_synthetic_object(bytes, "StateMachineNumber", &[]);
            push_synthetic_object(bytes, "StateMachineLayer", &[]);
            push_synthetic_object(bytes, "AnyState", &[]);
            push_synthetic_object(bytes, "EntryState", &[]);
            push_synthetic_object_with_properties(bytes, "ListenerNumberChange", |bytes| {
                push_synthetic_uint_property(bytes, "ListenerNumberChange", "inputId", 0);
                push_synthetic_uint_property(
                    bytes,
                    "ListenerNumberChange",
                    "flags",
                    STATE_AT_START,
                );
                push_synthetic_f32_property(bytes, "ListenerNumberChange", "value", 7.0);
            });
            push_synthetic_object(bytes, "ExitState", &[]);
            push_synthetic_object(bytes, "StateMachineLayer", &[]);
            push_synthetic_object(bytes, "AnyState", &[]);
            push_synthetic_object(bytes, "EntryState", &[]);
            push_synthetic_object(bytes, "ExitState", &[]);
        });
        let file = read_runtime_file(&bytes).expect("serial-layer fixture imports");
        let graph = GraphFile::from_runtime_file(&file).expect("serial-layer fixture graphs");
        let mut artboard =
            ArtboardInstance::from_graph(&file, graph.artboards.first().expect("artboard"))
                .expect("serial-layer fixture instances");

        StateMachineInstance::reset_layer_construction_number_snapshots();
        let state_machine = artboard
            .state_machine_instance(0)
            .expect("imported state-machine occurrence");

        assert_eq!(
            StateMachineInstance::layer_construction_number_snapshots(),
            vec![vec![Some(0.0)], vec![Some(7.0)]],
            "the layer-2 initialization observer must run after layer 1's entry action"
        );
        assert_eq!(
            state_machine
                .input(0)
                .and_then(|input| input.number_value()),
            Some(7.0)
        );
    }

    #[test]
    fn imported_nested_state_machine_retains_missing_child_owner_inputs_and_empty_forwarding() {
        let bytes = synthetic_riv(9598, |bytes| {
            push_synthetic_object(bytes, "Backboard", &[]);
            push_synthetic_object(bytes, "Artboard", &[]);
            push_synthetic_object(bytes, "NestedStateMachine", &[("parentId", 0)]);
            push_synthetic_object(bytes, "NestedBool", &[("parentId", 1), ("inputId", 9)]);
        });
        let file = read_runtime_file(&bytes).expect("synthetic riv should import");
        let graph = GraphFile::from_runtime_file(&file).expect("synthetic riv should graph");
        let artboard_graph = graph.artboards.first().expect("synthetic riv has artboard");
        let nested_object = artboard_graph
            .local_objects
            .iter()
            .find(|local| {
                file.object(local.global_id as usize)
                    .is_some_and(|object| object.type_name == "NestedStateMachine")
            })
            .expect("nested state-machine object");
        let object = file
            .object(nested_object.global_id as usize)
            .expect("nested state-machine runtime object");
        let mut child = synthetic_instance(Vec::new(), Vec::new());
        let mut occurrence = RuntimeNestedStateMachineInstance::from_imported(
            &file,
            artboard_graph,
            nested_object.local_id,
            object,
            &mut child,
        );

        assert_eq!(occurrence.animation_id(), u32::MAX as usize);
        assert!(!occurrence.has_state_machine());
        assert_eq!(occurrence.input_count(), 1);
        assert_eq!(occurrence.input_id_at(0), Some(9));
        assert_eq!(occurrence.input_id_at(1), None);
        assert_eq!(occurrence.input_id_named(""), Some(9));
        assert_eq!(occurrence.input_id_named("missing"), None);
        assert!(!occurrence.advance(&mut child, 0.0));
        assert!(!occurrence.hit_test(&child, 0.0, 0.0));
        assert!(!occurrence.pointer_down(&mut child, 0.0, 0.0, 1));
        assert!(!occurrence.pointer_move(&mut child, 0.0, 0.0, 0.25, 1));
        assert!(!occurrence.pointer_up(&mut child, 0.0, 0.0, 1));
        assert!(!occurrence.pointer_exit(&mut child, 0.0, 0.0, 1));
        assert!(!occurrence.drag_start(&mut child, 0.0, 0.0, 0.25, 1));
        assert!(!occurrence.drag_end(&mut child, 0.0, 0.0, 0.5, 1));
        assert!(!occurrence.try_change_state(&mut child));
        assert!(!occurrence.bind_owned_data_context(&RuntimeOwnedDataContext::default()));
        assert!(!occurrence.clear_data_context());

        let clone = occurrence.cold_clone(&mut child);
        assert_eq!(clone.animation_id(), u32::MAX as usize);
        assert!(!clone.has_state_machine());
        assert_eq!(clone.input_count(), 1);
        assert_eq!(clone.input_id_at(0), Some(9));
        assert_eq!(clone.input_id_named(""), Some(9));
    }

    #[test]
    fn public_clone_rebuilds_nested_state_machine_cold_but_transient_clone_keeps_live_value() {
        let mut definition = empty_state_machine(11);
        definition.inputs = Arc::new(vec![Some(RuntimeStateMachineInput::new_bool(
            1,
            Some("enabled".to_owned()),
            false,
        ))]);
        let mut nested = synthetic_nested_artboard_instance(22);
        nested.child.state_machines = Arc::new(vec![definition.clone()]);
        let state_machine = StateMachineInstance::new(0, &definition, &mut nested.child);
        let mut occurrence = RuntimeNestedStateMachineInstance::new(
            7,
            state_machine,
            vec![(0, Some("enabled".to_owned()), Some(true), None)],
        );
        assert!(
            occurrence
                .state_machine_mut()
                .expect("valid nested state machine")
                .set_bool(0, false)
        );
        nested
            .animations
            .push(RuntimeNestedAnimationInstance::StateMachine(occurrence));

        let mut parent = synthetic_instance(Vec::new(), Vec::new());
        parent.nested_artboards.insert(3, nested);
        let cloned = parent.clone();
        let transient = parent.clone_for_transient_layout();

        let bool_value = |artboard: &ArtboardInstance| {
            let nested = artboard
                .nested_artboards
                .get(&3)
                .expect("nested occurrence");
            let RuntimeNestedAnimationInstance::StateMachine(occurrence) = &nested.animations[0]
            else {
                panic!("nested animation remains a state machine");
            };
            occurrence
                .state_machine()
                .expect("valid nested state machine")
                .input(0)
                .and_then(|input| input.bool_value())
        };
        assert_eq!(
            bool_value(&cloned),
            Some(true),
            "public generated clone reapplies the authored nested input"
        );
        assert_eq!(
            bool_value(&transient),
            Some(false),
            "transient layout clone views the live occurrence snapshot"
        );
    }

    #[test]
    fn quantized_nested_skip_relies_on_unconditional_outer_state_probe() {
        let definition = empty_state_machine(11);
        let mut child = synthetic_instance(Vec::new(), Vec::new());
        let state_machine = StateMachineInstance::new(0, &definition, &mut child);
        child.state_machines = Arc::new(vec![definition]);
        let mut nested = RuntimeNestedArtboardInstance {
            child: Box::new(child),
            render_cache_revision: 0,
            render_resources: RefCell::new(crate::draw::RuntimeOccurrenceRenderResources::default()),
            initial_layout_paint_frame: RefCell::new(None),
            layout_data_transferred: false,
            layout_data_transfer_key: None,
            data_bind_path_ids: None,
            data_bind_path_is_relative: false,
            stateful_view_model_instance_local: None,
            stateful_view_model_instance_locals_by_id: BTreeMap::new(),
            stateful_view_model_context: None,
            stateful_global_view_model_contexts: BTreeMap::new(),
            data_bind_property_source_locals: Vec::new(),
            data_bind_image_source_locals: Vec::new(),
            data_bind_context_source_locals_by_path: BTreeMap::new(),
            animations: vec![RuntimeNestedAnimationInstance::StateMachine(
                RuntimeNestedStateMachineInstance::new(1, state_machine, Vec::new()),
            )],
            is_paused: false,
            speed: 1.0,
            quantize: 1.0,
            cumulated_seconds: 0.0,
        };

        assert_eq!(nested.begin_advance(0.25), Err(true));
        let RuntimeNestedAnimationInstance::StateMachine(occurrence) = &nested.animations[0] else {
            panic!("nested animation remains a state machine");
        };
        assert!(occurrence.state_machine().is_some());
    }

    #[test]
    fn mounted_nested_state_probe_is_unconditional() {
        let definition = empty_state_machine(11);
        let mut artboard = synthetic_instance(Vec::new(), Vec::new());
        let mut state_machine = StateMachineInstance::new(0, &definition, &mut artboard);
        artboard.state_machines = Arc::new(vec![definition]);

        assert!(!artboard.try_change_state_machine_instance(&mut state_machine));
        assert!(!artboard.try_change_state_machine_instance(&mut state_machine));
    }

    #[test]
    fn artboard_clone_shares_the_file_owned_external_font_snapshot() {
        let mut original = synthetic_instance(Vec::new(), Vec::new());
        let bytes = Arc::<[u8]>::from(vec![1, 2, 3]);
        original.external_font_assets = Arc::new(BTreeMap::from([(7, Arc::clone(&bytes))]));

        let cloned = original.clone();
        let cloned_bytes = cloned
            .external_font_assets
            .get(&7)
            .expect("cloned artboard retains external font asset");

        assert!(Arc::ptr_eq(
            &original.external_font_assets,
            &cloned.external_font_assets
        ));
        assert!(Arc::ptr_eq(&bytes, cloned_bytes));
    }

    #[test]
    fn unresolved_nested_artboard_binding_preserves_the_mounted_child() {
        let mut parent = synthetic_instance(Vec::new(), Vec::new());
        parent
            .nested_artboards
            .insert(3, synthetic_nested_artboard_instance(77));
        parent.nested_artboard_locals.push(3);

        // A synthetic instance has no build context, so every non-null id is
        // unresolvable. C++ keeps the outgoing mounted child in this case.
        assert!(!parent.set_nested_artboard_artboard_id(3, 12));
        assert_eq!(
            parent
                .nested_artboards
                .get(&3)
                .map(|nested| nested.child.graph_global_id),
            Some(77)
        );
        assert_eq!(parent.nested_artboard_locals, [3]);
    }

    #[test]
    fn null_then_unresolved_nested_artboard_binding_stays_absent() {
        let mut parent = synthetic_instance(Vec::new(), Vec::new());
        parent
            .nested_artboards
            .insert(3, synthetic_nested_artboard_instance(77));
        parent.nested_artboard_locals.push(3);

        assert!(parent.set_nested_artboard_artboard_id(3, u64::from(u32::MAX)));
        assert!(!parent.nested_artboards.contains_key(&3));
        assert!(parent.nested_artboard_locals.is_empty());

        // A later invalid/self target is not an explicit null and therefore
        // cannot resurrect the authored fallback or a new child.
        assert!(!parent.set_nested_artboard_artboard_id(3, 0));
        assert!(!parent.nested_artboards.contains_key(&3));
        assert!(parent.nested_artboard_locals.is_empty());
    }

    #[test]
    fn null_bound_artboard_swap_survives_pending_layout_sync() {
        // The swap host has a static artboard and an artboardId bind whose
        // view-model artboard property is never set. The first advance applies
        // the bind as an explicit null while the statically nested occurrence
        // still participates in hosting-artboard layout work.
        let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join("fixtures/sync/databind_null_artboard_swap.riv");
        let bytes = std::fs::read(&fixture)
            .unwrap_or_else(|error| panic!("read upstream fixture {}: {error}", fixture.display()));
        let file = read_runtime_file(&bytes).expect("null-artboard fixture imports");
        let graphs = GraphFile::from_runtime_file(&file).expect("null-artboard fixture graphs");
        let mut artboard = ArtboardInstance::from_graph_with_artboards(
            &file,
            &graphs.artboards[0],
            &graphs.artboards,
        )
        .expect("null-artboard fixture instance");
        let host_local_id = artboard
            .slots
            .iter()
            .find(|slot| slot.name.as_deref() == Some("swap host"))
            .map(|slot| slot.local_id)
            .expect("named swap host");

        assert!(artboard.nested_artboards.get(&host_local_id).is_some());
        artboard.advance(0.0).expect("first advance survives");
        artboard.advance(0.0).expect("second advance survives");

        assert!(artboard.nested_artboards.get(&host_local_id).is_none());
    }

    #[test]
    fn nested_artboard_swap_immediately_inherits_the_active_parent_context() {
        let number_key = property_key_for_name("Rectangle", "width").expect("rectangle width");
        let artboard = || {
            authoring_record(
                "Artboard",
                vec![authoring_property(
                    "Artboard",
                    "viewModelId",
                    AuthoringValue::Uint(0),
                )],
            )
        };
        let bound_rectangle = |width| {
            vec![
                authoring_record(
                    "Shape",
                    vec![authoring_property(
                        "Shape",
                        "parentId",
                        AuthoringValue::Uint(0),
                    )],
                ),
                authoring_record(
                    "Rectangle",
                    vec![
                        authoring_property("Rectangle", "parentId", AuthoringValue::Uint(1)),
                        authoring_property("Rectangle", "width", AuthoringValue::Double(width)),
                    ],
                ),
                authoring_record(
                    "DataBindContext",
                    vec![
                        authoring_property(
                            "DataBindContext",
                            "propertyKey",
                            AuthoringValue::Uint(u64::from(number_key)),
                        ),
                        authoring_property(
                            "DataBindContext",
                            "sourcePathIds",
                            AuthoringValue::Bytes(vec![0, 0]),
                        ),
                    ],
                ),
            ]
        };
        let mut records = vec![
            authoring_record("Backboard", Vec::new()),
            authoring_record(
                "ViewModel",
                vec![authoring_property(
                    "ViewModel",
                    "name",
                    AuthoringValue::String("Model".to_owned()),
                )],
            ),
            authoring_record(
                "ViewModelPropertyNumber",
                vec![authoring_property(
                    "ViewModelPropertyNumber",
                    "name",
                    AuthoringValue::String("width".to_owned()),
                )],
            ),
            artboard(),
            authoring_record(
                "NestedArtboard",
                vec![
                    authoring_property("NestedArtboard", "parentId", AuthoringValue::Uint(0)),
                    authoring_property("NestedArtboard", "artboardId", AuthoringValue::Uint(1)),
                ],
            ),
            artboard(),
        ];
        records.extend(bound_rectangle(1.0));
        records.push(artboard());
        records.extend(bound_rectangle(2.0));
        let file = RuntimeFile::from_authoring_records(records)
            .expect("nested replacement fixture imports");
        let graphs = GraphFile::from_runtime_file(&file).expect("nested replacement graphs");
        let mut parent = ArtboardInstance::from_graph_with_artboards(
            &file,
            &graphs.artboards[0],
            &graphs.artboards,
        )
        .expect("parent artboard instance");
        let host_local_id = graphs.artboards[0].nested_artboards[0].local_id;
        let mut context = RuntimeOwnedViewModelInstance::new(&file, 0).expect("owned context");
        assert!(context.set_number_by_property_index(0, 42.0));

        assert!(parent.bind_owned_view_model_artboard_context(&file, &context));
        assert_eq!(
            parent
                .nested_artboards
                .get(&host_local_id)
                .and_then(|nested| nested.child.artboard_data_bind_values.get(&[0, 0][..])),
            Some(&RuntimeDataBindGraphValue::Number(42.0)),
            "the authored child establishes that the synthetic binding resolves"
        );

        assert!(parent.set_nested_artboard_artboard_id(host_local_id, 2));
        let replacement = parent
            .nested_artboards
            .get(&host_local_id)
            .expect("replacement nested occurrence");
        assert_eq!(
            replacement.child.graph_global_id,
            graphs.artboards[2].global_id
        );
        assert_eq!(
            replacement.child.artboard_data_bind_values.get(&[0, 0][..]),
            Some(&RuntimeDataBindGraphValue::Number(42.0)),
            "C++ binds the existing DataContext during NestedArtboard::updateArtboard"
        );
    }

    #[test]
    fn stateful_nested_source_switch_uses_the_replacement_view_model_default() {
        let bytes = synthetic_riv(9700, |bytes| {
            push_synthetic_object(bytes, "Backboard", &[]);
            push_synthetic_object(bytes, "ViewModel", &[]);
            push_synthetic_object(bytes, "ViewModelInstance", &[("viewModelId", 0)]);
            push_synthetic_object(bytes, "ViewModel", &[]);
            push_synthetic_object(bytes, "ViewModelPropertyNumber", &[]);
            push_synthetic_object(bytes, "ViewModelInstance", &[("viewModelId", 1)]);
            push_synthetic_object(
                bytes,
                "ViewModelInstanceNumber",
                &[("viewModelPropertyId", 0)],
            );
            push_synthetic_object(bytes, "ViewModel", &[("viewModelType", 2)]);
            push_synthetic_object(bytes, "ViewModelInstance", &[("viewModelId", 2)]);
            push_synthetic_object(bytes, "Artboard", &[("viewModelId", 0)]);
            push_synthetic_object(
                bytes,
                "NestedArtboard",
                &[("parentId", 0), ("artboardId", 1), ("isStateful", 1)],
            );
            push_synthetic_object(
                bytes,
                "ViewModelInstance",
                &[("parentId", 1), ("viewModelId", 0)],
            );
            push_synthetic_object(
                bytes,
                "ViewModelInstance",
                &[("parentId", 1), ("viewModelId", 2)],
            );
            push_synthetic_object(bytes, "Artboard", &[("viewModelId", 0)]);
            push_synthetic_object(bytes, "Artboard", &[("viewModelId", 1)]);
            push_synthetic_object(bytes, "Artboard", &[("viewModelId", 1)]);
        });
        let file = read_runtime_file(&bytes).expect("stateful source-switch fixture imports");
        let graph = GraphFile::from_runtime_file(&file).expect("stateful fixture graphs");
        let parent_graph = &graph.artboards[0];
        let host_local_id = parent_graph.nested_artboards[0].local_id;
        let mut parent =
            ArtboardInstance::from_graph_with_artboards(&file, parent_graph, &graph.artboards)
                .expect("parent artboard instance");

        let authored = parent
            .nested_artboards
            .get(&host_local_id)
            .expect("authored nested occurrence");
        assert_eq!(
            authored
                .stateful_view_model_context
                .as_ref()
                .map(|context| context.borrow().view_model_index()),
            Some(0)
        );
        assert!(authored.stateful_view_model_instance_local.is_some());

        assert!(parent.set_nested_artboard_artboard_id(host_local_id, 2));
        let replacement = parent
            .nested_artboards
            .get(&host_local_id)
            .expect("replacement nested occurrence");
        assert_eq!(
            replacement.child.graph_global_id,
            graph.artboards[2].global_id
        );
        assert_eq!(
            replacement
                .stateful_view_model_context
                .as_ref()
                .map(|context| context.borrow().view_model_index()),
            Some(1),
            "a stateful source switch with no matching authored child must create the replacement VM default"
        );
        assert_eq!(replacement.stateful_view_model_instance_local, None);
        assert_eq!(
            replacement
                .stateful_global_view_model_contexts
                .get(&2)
                .map(|context| context.borrow().view_model_index()),
            Some(2),
            "the replacement local main remains combined with authored global contexts"
        );

        let replacement_context_identity = replacement
            .stateful_view_model_context
            .as_ref()
            .expect("generated replacement context")
            .borrow()
            .instance_identity();
        assert!(
            parent
                .nested_artboards
                .get_mut(&host_local_id)
                .and_then(|nested| nested.stateful_view_model_context.as_mut())
                .is_some_and(|context| {
                    context.borrow_mut().set_number_by_property_index(0, 42.0)
                })
        );
        assert!(parent.set_nested_artboard_artboard_id(host_local_id, 3));
        let same_view_model_replacement = parent
            .nested_artboards
            .get(&host_local_id)
            .expect("same-VM replacement occurrence");
        assert_eq!(
            same_view_model_replacement
                .stateful_view_model_context
                .as_ref()
                .map(|context| context.borrow().instance_identity()),
            Some(replacement_context_identity),
            "an owned replacement context survives a source switch to another artboard with the same VM"
        );
        assert_eq!(
            same_view_model_replacement
                .stateful_view_model_context
                .as_ref()
                .and_then(|context| context.borrow().number_value_by_slot(0)),
            Some(42.0),
            "same-VM reuse preserves runtime mutations"
        );
    }

    #[test]
    fn non_stateful_nested_host_activates_an_authored_child_view_model() {
        let bytes = synthetic_riv(9701, |bytes| {
            push_synthetic_object(bytes, "Backboard", &[]);
            push_synthetic_object(bytes, "ViewModel", &[]);
            push_synthetic_object(bytes, "ViewModelInstance", &[("viewModelId", 0)]);
            push_synthetic_object(bytes, "Artboard", &[("viewModelId", 0)]);
            push_synthetic_object(
                bytes,
                "NestedArtboard",
                &[("parentId", 0), ("artboardId", 1), ("isStateful", 0)],
            );
            push_synthetic_object(
                bytes,
                "ViewModelInstance",
                &[("parentId", 1), ("viewModelId", 0)],
            );
            push_synthetic_object(bytes, "Artboard", &[("viewModelId", 0)]);
        });
        let file = read_runtime_file(&bytes).expect("non-stateful nested fixture imports");
        let graph = GraphFile::from_runtime_file(&file).expect("non-stateful fixture graphs");
        let parent = ArtboardInstance::from_graph_with_artboards(
            &file,
            &graph.artboards[0],
            &graph.artboards,
        )
        .expect("parent artboard instance");
        let host_local_id = graph.artboards[0].nested_artboards[0].local_id;
        let nested = parent
            .nested_artboards
            .get(&host_local_id)
            .expect("authored nested occurrence");

        assert!(nested.stateful_view_model_instance_local.is_some());
        assert_eq!(
            nested
                .stateful_view_model_context
                .as_ref()
                .map(|context| context.borrow().view_model_index()),
            Some(0),
            "NestedArtboard::onAddedClean retains the authored standard child VMI even when the host is not stateful"
        );
        assert!(nested.stateful_global_view_model_contexts.is_empty());
    }

    #[test]
    fn public_artboard_clone_is_cold_but_transient_layout_clone_keeps_scripts() {
        let mut original = synthetic_instance(Vec::new(), Vec::new());
        original.set_script_instance_for_global(
            7,
            Box::new(UpdateScriptInstance {
                inits: Rc::new(Cell::new(0)),
                updates: Rc::new(Cell::new(0)),
            }),
        );
        let mut child = synthetic_instance(Vec::new(), Vec::new());
        child.set_script_instance_for_global(
            8,
            Box::new(UpdateScriptInstance {
                inits: Rc::new(Cell::new(0)),
                updates: Rc::new(Cell::new(0)),
            }),
        );
        child.layout_constraint_bounds_enabled = true;
        child.layout_constraint_bounds = Some(Arc::new(BTreeMap::from([(
            0,
            RuntimeLayoutBounds {
                x: 1.0,
                y: 2.0,
                width: 30.0,
                height: 40.0,
            },
        )])));
        original.nested_artboards.insert(
            0,
            RuntimeNestedArtboardInstance {
                child: Box::new(child),
                render_cache_revision: 0,
                render_resources: RefCell::new(
                    crate::draw::RuntimeOccurrenceRenderResources::default(),
                ),
                initial_layout_paint_frame: RefCell::new(Some(
                    RuntimeInitialNestedLayoutPaintFrame::default(),
                )),
                layout_data_transferred: true,
                layout_data_transfer_key: Some(RuntimeNestedLayoutDataTransferKey {
                    parent_layout: RuntimeNestedLayoutBoundsCacheKey {
                        graph_global_id: 11,
                        layout_revision: 3,
                    },
                    assigned_bounds: RuntimeLayoutBounds {
                        x: 1.0,
                        y: 2.0,
                        width: 30.0,
                        height: 40.0,
                    },
                    child_layout_revision: 5,
                }),
                data_bind_path_ids: None,
                data_bind_path_is_relative: false,
                stateful_view_model_instance_local: None,
                stateful_view_model_instance_locals_by_id: BTreeMap::new(),
                stateful_view_model_context: None,
                stateful_global_view_model_contexts: BTreeMap::new(),
                data_bind_property_source_locals: Vec::new(),
                data_bind_image_source_locals: Vec::new(),
                data_bind_context_source_locals_by_path: BTreeMap::new(),
                animations: Vec::new(),
                is_paused: false,
                speed: 1.0,
                quantize: -1.0,
                cumulated_seconds: 0.0,
            },
        );

        let original_identity = original.instance_identity();
        let original_nested_identity = original.nested_artboards[&0].child.instance_identity();
        let cloned = original.clone();
        let transient = original.clone_for_transient_layout();

        assert_ne!(cloned.instance_identity(), original_identity);
        assert_eq!(transient.instance_identity(), original_identity);
        assert_ne!(
            cloned.nested_artboards[&0].child.instance_identity(),
            original_nested_identity
        );
        assert_eq!(
            transient.nested_artboards[&0].child.instance_identity(),
            original_nested_identity
        );
        assert!(!cloned.nested_artboards[&0].layout_data_transferred);
        assert!(
            cloned.nested_artboards[&0]
                .layout_data_transfer_key
                .is_none()
        );
        assert!(
            cloned.nested_artboards[&0]
                .initial_layout_paint_frame
                .borrow()
                .is_none()
        );
        assert!(transient.nested_artboards[&0].layout_data_transferred);
        assert_eq!(
            transient.nested_artboards[&0].layout_data_transfer_key,
            original.nested_artboards[&0].layout_data_transfer_key
        );
        assert!(
            transient.nested_artboards[&0]
                .initial_layout_paint_frame
                .borrow()
                .is_none()
        );
        assert!(
            !cloned.nested_artboards[&0]
                .child
                .layout_constraint_bounds_enabled
        );
        assert!(
            cloned.nested_artboards[&0]
                .child
                .layout_constraint_bounds
                .is_none()
        );
        assert!(
            transient.nested_artboards[&0]
                .child
                .layout_constraint_bounds_enabled
        );
        assert!(Arc::ptr_eq(
            transient.nested_artboards[&0]
                .child
                .layout_constraint_bounds
                .as_ref()
                .expect("transient constraint bounds"),
            original.nested_artboards[&0]
                .child
                .layout_constraint_bounds
                .as_ref()
                .expect("source constraint bounds"),
        ));
        assert!(original.has_script_instance_for_global(7));
        assert!(!cloned.has_script_instance_for_global(7));
        assert!(transient.has_script_instance_for_global(7));
        assert!(
            !cloned
                .nested_artboards
                .get(&0)
                .is_some_and(|nested| nested.child.has_script_instance_for_global(8))
        );
        assert!(
            transient
                .nested_artboards
                .get(&0)
                .is_some_and(|nested| nested.child.has_script_instance_for_global(8))
        );
    }

    #[test]
    fn component_list_owner_clones_cold_but_transient_layout_keeps_retained_rows() {
        let mut original = synthetic_instance(
            vec![synthetic_component_for_type(0, "ArtboardComponentList")],
            vec![0],
        );
        {
            let list = original
                .component_list_state_mut(0)
                .expect("concrete component-list owner");
            list.item_transforms = vec![Mat2D([1.0, 0.0, 0.0, 1.0, 4.0, 8.0])];
            list.order_cache.borrow_mut().indices.push(0);
            list.order_cache.borrow_mut().valid = true;
        }

        let cloned = original.clone();
        let transient = original.clone_for_transient_layout();

        let cloned_list = cloned
            .component_list_state(0)
            .expect("clone retains concrete owner type");
        assert!(cloned_list.item_transforms.is_empty());
        assert!(cloned_list.order_cache.borrow().indices.is_empty());
        assert!(!cloned_list.order_cache.borrow().valid);

        let transient_list = transient
            .component_list_state(0)
            .expect("transient retains concrete owner type");
        assert_eq!(transient_list.item_transforms.len(), 1);
        assert_eq!(transient_list.order_cache.borrow().indices, [0]);
        assert!(transient_list.order_cache.borrow().valid);
    }

    #[test]
    fn scripted_updates_run_once_per_attach_or_input_change() {
        let mut instance = synthetic_instance(
            vec![synthetic_component_for_type(0, "ScriptedDrawable")],
            vec![0],
        );
        let inits = Rc::new(Cell::new(0));
        let updates = Rc::new(Cell::new(0));
        instance.set_script_instance_for_global(
            0,
            Box::new(UpdateScriptInstance {
                inits: Rc::clone(&inits),
                updates: Rc::clone(&updates),
            }),
        );

        assert!(instance.update_script_instances().expect("initial update"));
        assert_eq!(updates.get(), 1);
        assert!(!instance.update_script_instances().expect("clean update"));

        instance
            .set_script_input_for_global(0, "value", ScriptValue::Number(2.0))
            .expect("input update");
        assert!(instance.update_script_instances().expect("dirty update"));
        assert_eq!(updates.get(), 2);

        assert!(
            instance
                .reinitialize_script_instances()
                .expect("reinitialize")
        );
        assert_eq!(inits.get(), 1);
        assert!(
            instance
                .update_script_instances()
                .expect("post-init update")
        );
        assert_eq!(updates.get(), 3);
    }

    #[test]
    fn scripted_updates_run_at_their_dependency_slots_and_retry_from_the_failed_owner() {
        let mut first = synthetic_component_for_type(0, "ScriptedDrawable");
        first.global_id = 9;
        let mut second = synthetic_component_for_type(1, "ScriptedDrawable");
        second.global_id = 2;
        let mut instance = synthetic_instance(vec![first, second], vec![1, 0]);
        let calls = Rc::new(RefCell::new(Vec::new()));
        let fail_once = Rc::new(Cell::new(true));
        instance.set_script_instance_for_global(
            9,
            Box::new(OrderedUpdateScriptInstance {
                label: 9,
                calls: Rc::clone(&calls),
                fail_once: None,
            }),
        );
        instance.set_script_instance_for_global(
            2,
            Box::new(OrderedUpdateScriptInstance {
                label: 2,
                calls: Rc::clone(&calls),
                fail_once: Some(Rc::clone(&fail_once)),
            }),
        );

        instance
            .update_pass_with_script_errors()
            .expect_err("the first dependency owner fails once");
        assert_eq!(calls.borrow().as_slice(), [2, 9]);

        instance
            .update_pass_with_script_errors()
            .expect("the failed owner remains dirtied for an exact retry");
        assert_eq!(
            calls.borrow().as_slice(),
            [2, 9, 2],
            "ScriptedDrawable::update consumes ScriptUpdate at the concrete Component's retained \
             dependency slot; a failure rearms that owner without replaying an already-successful \
             later slot (`component.cpp:222-241`; `scripted_drawable.cpp:347-374`)"
        );
    }

    #[test]
    fn nested_scripts_advance_in_place_with_exact_local_speed_adjusted_steps() {
        let seconds = Rc::new(RefCell::new(Vec::new()));
        let mut child = synthetic_instance(
            vec![synthetic_component_for_type(0, "ScriptedDrawable")],
            vec![0],
        );
        child.set_script_instance_for_global(
            0,
            Box::new(RecordingAdvanceScriptInstance {
                seconds: Rc::clone(&seconds),
            }),
        );
        let mut parent = synthetic_instance(
            vec![synthetic_component_for_type(0, "NestedArtboard")],
            vec![0],
        );
        parent.nested_artboards.insert(
            0,
            RuntimeNestedArtboardInstance {
                child: Box::new(child),
                render_cache_revision: 0,
                render_resources: RefCell::new(
                    crate::draw::RuntimeOccurrenceRenderResources::default(),
                ),
                initial_layout_paint_frame: RefCell::new(None),
                layout_data_transferred: false,
                layout_data_transfer_key: None,
                data_bind_path_ids: None,
                data_bind_path_is_relative: false,
                stateful_view_model_instance_local: None,
                stateful_view_model_instance_locals_by_id: BTreeMap::new(),
                stateful_view_model_context: None,
                stateful_global_view_model_contexts: BTreeMap::new(),
                data_bind_property_source_locals: Vec::new(),
                data_bind_image_source_locals: Vec::new(),
                data_bind_context_source_locals_by_path: BTreeMap::new(),
                animations: Vec::new(),
                is_paused: false,
                speed: 2.0,
                quantize: -1.0,
                cumulated_seconds: 0.0,
            },
        );
        parent.nested_artboard_locals.push(0);

        let mut factory = RecordingFactory::new();
        parent
            .advance_frame_components_with_factory(0.25, &mut factory)
            .expect("first retained advance succeeds");
        parent
            .advance_frame_components_with_factory(0.125, &mut factory)
            .expect("second retained advance succeeds");

        assert_eq!(seconds.borrow().as_slice(), [0.5, 0.25]);
    }

    #[test]
    fn state_machine_batch_advances_nested_scripts_once() {
        let seconds = Rc::new(RefCell::new(Vec::new()));
        let mut child = synthetic_instance(
            vec![synthetic_component_for_type(0, "ScriptedDrawable")],
            vec![0],
        );
        child.set_script_instance_for_global(
            0,
            Box::new(RecordingAdvanceScriptInstance {
                seconds: Rc::clone(&seconds),
            }),
        );
        let mut parent = synthetic_instance(
            vec![synthetic_component_for_type(0, "NestedArtboard")],
            vec![0],
        );
        parent.nested_artboards.insert(
            0,
            RuntimeNestedArtboardInstance {
                child: Box::new(child),
                render_cache_revision: 0,
                render_resources: RefCell::new(
                    crate::draw::RuntimeOccurrenceRenderResources::default(),
                ),
                initial_layout_paint_frame: RefCell::new(None),
                layout_data_transferred: false,
                layout_data_transfer_key: None,
                data_bind_path_ids: None,
                data_bind_path_is_relative: false,
                stateful_view_model_instance_local: None,
                stateful_view_model_instance_locals_by_id: BTreeMap::new(),
                stateful_view_model_context: None,
                stateful_global_view_model_contexts: BTreeMap::new(),
                data_bind_property_source_locals: Vec::new(),
                data_bind_image_source_locals: Vec::new(),
                data_bind_context_source_locals_by_path: BTreeMap::new(),
                animations: Vec::new(),
                is_paused: false,
                speed: 2.0,
                quantize: -1.0,
                cumulated_seconds: 0.0,
            },
        );
        parent.nested_artboard_locals.push(0);
        parent.state_machines = Arc::new(vec![empty_state_machine(11), empty_state_machine(12)]);
        let definitions = Arc::clone(&parent.state_machines);
        let mut machines = definitions
            .iter()
            .enumerate()
            .map(|(index, definition)| StateMachineInstance::new(index, definition, &mut parent))
            .collect::<Vec<_>>();

        let mut factory = RecordingFactory::new();
        parent
            .advance_frame_components_with_state_machines_and_factory(
                &mut machines,
                0.25,
                &mut factory,
            )
            .expect("one mixed-family frame succeeds");

        assert_eq!(seconds.borrow().as_slice(), [0.5]);
    }

    #[test]
    fn failed_script_advance_parks_each_owner_while_surfacing_the_error() {
        for type_name in ["ScriptedDrawable", "ScriptedLayout", "ScriptedPathEffect"] {
            let attempts = Rc::new(RefCell::new(Vec::new()));
            let should_fail = Rc::new(Cell::new(true));
            let mut instance =
                synthetic_instance(vec![synthetic_component_for_type(0, type_name)], vec![0]);
            instance.set_script_instance_for_global(
                0,
                Box::new(FailOnceAdvanceScriptInstance {
                    attempts: Rc::clone(&attempts),
                    should_fail: Rc::clone(&should_fail),
                }),
            );
            let mut factory = RecordingFactory::new();

            let error = instance
                .advance_frame_components_with_factory(0.5, &mut factory)
                .expect_err("the protected script failure remains a typed host signal");
            assert_eq!(error.to_string(), "fail once");
            assert!(
                !instance
                    .advance_frame_components_with_factory(0.5, &mut factory)
                    .expect("a parked owner does not call the script again")
            );
            assert!(
                !instance
                    .advance_frame_components_with_factory(0.25, &mut factory)
                    .expect("the owner remains parked on later frames")
            );

            assert_eq!(
                attempts.borrow().as_slice(),
                [0.5],
                "{type_name} must retain C++ clear-before-call park semantics"
            );
        }
    }

    #[test]
    fn failed_script_advance_surfaces_after_later_retained_slots_run() {
        let attempts = Rc::new(RefCell::new(Vec::new()));
        let should_fail = Rc::new(Cell::new(true));
        let later_calls = Rc::new(RefCell::new(Vec::new()));
        let mut failing = synthetic_component_for_type(0, "ScriptedDrawable");
        failing.global_id = 10;
        let mut later = synthetic_component_for_type(1, "ScriptedPathEffect");
        later.global_id = 20;
        let mut instance = synthetic_instance(vec![failing, later], vec![0, 1]);
        instance.set_script_instance_for_global(
            10,
            Box::new(FailOnceAdvanceScriptInstance {
                attempts: Rc::clone(&attempts),
                should_fail,
            }),
        );
        instance.set_script_instance_for_global(
            20,
            Box::new(OrderedAdvanceScriptInstance {
                label: 20,
                calls: Rc::clone(&later_calls),
            }),
        );
        let mut factory = RecordingFactory::new();

        let error = instance
            .advance_frame_components_with_factory(0.5, &mut factory)
            .expect_err("the first typed error is surfaced after retained scheduling completes");

        assert_eq!(error.to_string(), "fail once");
        assert_eq!(attempts.borrow().as_slice(), [0.5]);
        assert_eq!(
            later_calls.borrow().as_slice(),
            [20],
            "C++ converts the failed call to false and continues the m_advancingComponents loop"
        );
    }

    #[test]
    fn nested_failed_script_advance_surfaces_after_later_parent_slots_run() {
        let attempts = Rc::new(RefCell::new(Vec::new()));
        let later_calls = Rc::new(RefCell::new(Vec::new()));
        let mut child_script = synthetic_component_for_type(0, "ScriptedDrawable");
        child_script.global_id = 101;
        let mut child = synthetic_instance(vec![child_script], vec![0]);
        child.set_script_instance_for_global(
            101,
            Box::new(FailOnceAdvanceScriptInstance {
                attempts: Rc::clone(&attempts),
                should_fail: Rc::new(Cell::new(true)),
            }),
        );
        let mut nested = synthetic_nested_artboard_instance(101);
        nested.child = Box::new(child);

        let nested_host = synthetic_component_for_type(0, "NestedArtboard");
        let mut later = synthetic_component_for_type(1, "ScriptedDrawable");
        later.global_id = 20;
        let mut parent = synthetic_instance(vec![nested_host, later], vec![0, 1]);
        parent.nested_artboards.insert(0, nested);
        parent.set_script_instance_for_global(
            20,
            Box::new(OrderedAdvanceScriptInstance {
                label: 20,
                calls: Rc::clone(&later_calls),
            }),
        );
        let mut factory = RecordingFactory::new();

        let error = parent
            .advance_frame_components_with_factory(0.5, &mut factory)
            .expect_err("the nested typed error is surfaced after the parent schedule completes");

        assert_eq!(error.to_string(), "fail once");
        assert_eq!(attempts.borrow().as_slice(), [0.5]);
        assert_eq!(later_calls.borrow().as_slice(), [20]);
    }

    #[test]
    fn root_advance_runs_update_pass_before_surfacing_script_advance_error() {
        let attempts = Rc::new(RefCell::new(Vec::new()));
        let root = synthetic_component_for_type(0, "Artboard");
        let mut scripted = synthetic_component_for_type(1, "ScriptedDrawable");
        scripted.global_id = 10;
        let mut instance = synthetic_instance(vec![root, scripted], vec![0, 1]);
        instance.set_script_instance_for_global(
            10,
            Box::new(FailOnceAdvanceScriptInstance {
                attempts: Rc::clone(&attempts),
                should_fail: Rc::new(Cell::new(true)),
            }),
        );
        instance
            .update_pass_with_script_errors()
            .expect("initial ScriptUpdate settles before the error probe");
        instance.install_persistent_dirt_component_fixture();

        let error = instance
            .advance(0.5)
            .expect_err("the typed error is surfaced after root settlement");

        assert_eq!(error.to_string(), "fail once");
        assert_eq!(attempts.borrow().as_slice(), [0.5]);
        assert_eq!(
            instance.persistent_dirt_component_fixture_receipt(),
            (1, 1, false),
            "the C++ update pass still consumes earlier component dirt before Rust surfaces the additive error"
        );
    }

    #[test]
    fn scripted_advances_stop_on_false_and_reactivate_on_input_change() {
        let mut instance = synthetic_instance(
            vec![synthetic_component_for_type(0, "ScriptedDrawable")],
            vec![0],
        );
        let advances = Rc::new(Cell::new(0));
        instance.set_script_instance_for_global(
            0,
            Box::new(AdvanceScriptInstance {
                advances: Rc::clone(&advances),
            }),
        );

        assert!(
            instance
                .advance_script_instances(0.1)
                .expect("first advance")
        );
        assert!(
            !instance
                .advance_script_instances(0.1)
                .expect("second advance")
        );
        assert!(
            !instance
                .advance_script_instances(0.1)
                .expect("inactive advance")
        );
        assert_eq!(advances.get(), 2);

        instance
            .set_script_input_for_global(0, "value", ScriptValue::Number(2.0))
            .expect("input update");
        assert!(
            instance
                .advance_script_instances(0.1)
                .expect("reactivated advance")
        );
        assert_eq!(advances.get(), 3);
    }

    #[test]
    fn true_scripted_path_effect_advance_schedules_effect_invalidation() {
        let mut effect = synthetic_component_for_type(0, "ScriptedPathEffect");
        effect.global_id = 10;
        let seconds = Rc::new(RefCell::new(Vec::new()));
        let mut instance = synthetic_instance(vec![effect], vec![0]);
        instance.set_script_instance_for_global(
            10,
            Box::new(RecordingAdvanceScriptInstance {
                seconds: Rc::clone(&seconds),
            }),
        );
        instance
            .update_pass_with_script_errors()
            .expect("consume the attachment-time ScriptUpdate");

        assert!(instance.advance_script_instances(0.25).unwrap());
        assert_eq!(seconds.borrow().as_slice(), [0.25]);
        assert!(
            instance
                .debug_component_dirt(0)
                .is_some_and(|dirt| dirt.contains(ComponentDirt::SCRIPT_UPDATE)),
            "a true advance must invalidate the retained EffectPath at the scripted effect's dependency slot (scripted_path_effect.cpp:111-132,199-207)",
        );
    }

    #[test]
    fn scripted_advances_follow_retained_object_order_not_global_id_order() {
        let mut first = synthetic_component_for_type(0, "ScriptedDrawable");
        first.global_id = 9;
        let mut second = synthetic_component_for_type(1, "ScriptedDrawable");
        second.global_id = 2;
        let mut instance = synthetic_instance(vec![first, second], vec![0, 1]);
        let calls = Rc::new(RefCell::new(Vec::new()));
        for global_id in [9, 2] {
            instance.set_script_instance_for_global(
                global_id,
                Box::new(OrderedAdvanceScriptInstance {
                    label: global_id,
                    calls: Rc::clone(&calls),
                }),
            );
        }

        assert!(instance.advance_frame_components(0.1).unwrap());
        assert_eq!(
            calls.borrow().as_slice(),
            [9, 2],
            "the complete frame entry point walks Artboard::m_advancingComponents insertion \
             order, not a late global-id script sweep (`artboard.cpp:1463-1480`; \
             `advancing_component.cpp:17-44`)"
        );
    }

    #[test]
    fn mixed_scripted_component_advances_run_at_their_retained_cpp_slots() {
        let calls = Rc::new(RefCell::new(Vec::new()));

        let mut nested_script = synthetic_component_for_type(0, "ScriptedDrawable");
        nested_script.global_id = 101;
        let mut child = synthetic_instance(vec![nested_script], vec![0]);
        child.set_script_instance_for_global(
            101,
            Box::new(OrderedAdvanceScriptInstance {
                label: 101,
                calls: Rc::clone(&calls),
            }),
        );
        let mut nested = synthetic_nested_artboard_instance(101);
        nested.child = Box::new(child);

        let mut drawable = synthetic_component_for_type(0, "ScriptedDrawable");
        drawable.global_id = 90;
        let nested_host = synthetic_component_for_type(1, "NestedArtboard");
        let mut path_effect = synthetic_component_for_type(2, "ScriptedPathEffect");
        path_effect.global_id = 20;
        let mut layout = synthetic_component_for_type(3, "ScriptedLayout");
        layout.global_id = 70;
        let mut instance = synthetic_instance(
            vec![drawable, nested_host, path_effect, layout],
            vec![0, 1, 2, 3],
        );
        instance.nested_artboards.insert(1, nested);
        for global_id in [90, 20, 70] {
            instance.set_script_instance_for_global(
                global_id,
                Box::new(OrderedAdvanceScriptInstance {
                    label: global_id,
                    calls: Rc::clone(&calls),
                }),
            );
        }

        let mut factory = RecordingFactory::new();
        assert!(
            instance
                .advance_frame_components_with_factory(0.1, &mut factory)
                .expect("mixed retained component advance succeeds")
        );
        assert_eq!(
            calls.borrow().as_slice(),
            [90, 101, 20, 70],
            concat!(
                "ScriptedDrawable, nested-artboard work, ScriptedPathEffect, and ScriptedLayout ",
                "run from their interleaved m_advancingComponents slots; the removed deferred ",
                "queue would have moved the root scripted calls after the nested slot ",
                "(`artboard.cpp:1463-1480`; `scripted_drawable.cpp:376-399`; ",
                "`scripted_path_effect.cpp:111-133`)"
            )
        );
    }

    #[test]
    fn collapsed_scripted_component_defers_update_and_advance_until_visible() {
        let mut instance = synthetic_instance(
            vec![
                synthetic_component_for_type(0, "ScriptedDrawable"),
                synthetic_component_for_type(1, "ScriptedDrawable"),
            ],
            vec![0, 1],
        );
        let inits = Rc::new(Cell::new(0));
        let updates = Rc::new(Cell::new(0));
        instance.set_script_instance_for_global(
            0,
            Box::new(UpdateScriptInstance {
                inits: Rc::clone(&inits),
                updates: Rc::clone(&updates),
            }),
        );
        let advances = Rc::new(Cell::new(0));
        instance.set_script_instance_for_global(
            1,
            Box::new(AdvanceScriptInstance {
                advances: Rc::clone(&advances),
            }),
        );

        assert!(instance.collapse_component(0, true));
        assert!(instance.collapse_component(1, true));
        assert!(
            !instance
                .update_script_instances()
                .expect("collapsed update is deferred")
        );
        assert!(
            !instance
                .advance_script_instances(0.1)
                .expect("collapsed advance is deferred")
        );
        assert_eq!(updates.get(), 0);
        assert_eq!(advances.get(), 0);

        assert!(instance.collapse_component(0, false));
        assert!(instance.collapse_component(1, false));
        assert!(
            instance
                .update_script_instances()
                .expect("deferred update runs when visible")
        );
        assert!(
            instance
                .advance_script_instances(0.1)
                .expect("armed advance runs when visible")
        );
        assert_eq!(updates.get(), 1);
        assert_eq!(advances.get(), 1);
    }

    #[test]
    fn waking_a_parked_script_advance_rearms_it_and_marks_paint_dirty() {
        let mut instance = synthetic_instance(
            vec![synthetic_component_for_type(0, "ScriptedDrawable")],
            vec![0],
        );
        let advances = Rc::new(Cell::new(0));
        instance.set_script_instance_for_global(
            0,
            Box::new(AdvanceScriptInstance {
                advances: Rc::clone(&advances),
            }),
        );

        assert!(instance.advance_script_instances(0.1).unwrap());
        assert!(!instance.advance_script_instances(0.1).unwrap());
        instance.clear_component_dirt(0);

        assert!(instance.wake_script_advance_for_global(0));
        assert!(
            instance
                .component(0)
                .expect("scripted drawable component")
                .dirt
                .contains(ComponentDirt::PAINT)
        );
        assert!(instance.advance_script_instances(0.1).unwrap());
        assert_eq!(advances.get(), 3);
    }

    #[test]
    fn successful_script_advance_invalidates_paint_without_calling_update() {
        let mut instance = synthetic_instance(
            vec![synthetic_component_for_type(0, "ScriptedDrawable")],
            vec![0],
        );
        let advances = Rc::new(Cell::new(0));
        let updates = Rc::new(Cell::new(0));
        instance.set_script_instance_for_global(
            0,
            Box::new(AdvanceAndUpdateScriptInstance {
                advances: Rc::clone(&advances),
                updates: Rc::clone(&updates),
            }),
        );
        assert!(instance.update_script_instances().unwrap());
        assert_eq!(updates.get(), 1);
        instance.clear_component_dirt(0);

        assert!(instance.advance_script_instances(0.1).unwrap());
        assert_eq!(advances.get(), 1);
        assert!(
            instance
                .component(0)
                .expect("scripted drawable component")
                .dirt
                .contains(ComponentDirt::PAINT)
        );
        assert!(!instance.update_script_instances().unwrap());
        assert_eq!(updates.get(), 1);
    }

    #[test]
    fn render_opacity_update_invalidates_a_prepared_zero_opacity_frame() {
        let mut instance = synthetic_instance(vec![synthetic_component(0, 0)], vec![0]);
        assert_eq!(instance.component(0).unwrap().transform.render_opacity, 0.0);
        let prepared_epoch = instance.prepared_epoch;

        let component = instance.component_handle(0).unwrap();
        instance.update_component(component, ComponentDirt::RENDER_OPACITY);

        assert_eq!(instance.component(0).unwrap().transform.render_opacity, 1.0);
        assert!(instance.prepared_epoch > prepared_epoch);
    }

    #[test]
    fn nested_animation_runtime_knobs_follow_keyed_parent_properties() {
        let animation_instance = |animation_index| {
            let animation = RuntimeLinearAnimation {
                global_id: animation_index as u32,
                name: None,
                fps: 60,
                duration: 60,
                speed: 1.0,
                loop_value: 0,
                work_start: 0,
                work_end: 60,
                enable_work_area: false,
                quantize: false,
                keyed_objects: Arc::new(Vec::new()),
                key_frame_data_bind_templates: Arc::new(Vec::new()),
                has_keyed_callbacks: false,
            };
            LinearAnimationInstance::new_for_test(
                RuntimeLinearAnimationHandle::new(animation_index),
                &animation,
                1.0,
            )
        };

        let mut host = synthetic_component(0, 0);
        host.type_name = "NestedArtboard";
        host.transform_property_keys =
            crate::components::TransformPropertyKeys::for_type(host.type_name);
        let mut simple = synthetic_component(1, 1);
        simple.type_name = "NestedSimpleAnimation";
        simple.transform_property_keys =
            crate::components::TransformPropertyKeys::for_type(simple.type_name);
        let mut remap = synthetic_component(2, 2);
        remap.type_name = "NestedRemapAnimation";
        remap.transform_property_keys =
            crate::components::TransformPropertyKeys::for_type(remap.type_name);

        let mut instance = synthetic_instance(vec![host, simple, remap], Vec::new());
        synthetic_link_parent(&mut instance, 1, 0);
        synthetic_link_parent(&mut instance, 2, 0);
        let mut nested = synthetic_nested_artboard_instance(7);
        nested.animations = vec![
            RuntimeNestedAnimationInstance::Simple {
                local_id: 1,
                animation: animation_instance(0),
                is_playing: false,
                speed: 1.0,
                mix: 1.0,
            },
            RuntimeNestedAnimationInstance::Remap {
                local_id: 2,
                animation: animation_instance(1),
                mix: 1.0,
            },
        ];
        instance.nested_artboards.insert(0, nested);

        let mix_key = property_key_for_name("NestedLinearAnimation", "mix").expect("mix key");
        let speed_key = property_key_for_name("NestedSimpleAnimation", "speed").expect("speed key");
        let playing_key =
            property_key_for_name("NestedSimpleAnimation", "isPlaying").expect("isPlaying key");
        assert!(instance.set_keyed_double_property(1, mix_key, 0.25));
        assert!(instance.set_keyed_double_property(2, mix_key, 0.0));
        assert!(instance.set_keyed_double_property(1, speed_key, 2.0));
        assert!(instance.set_bool_property(1, playing_key, true));

        let nested = instance.nested_artboards.get(&0).expect("nested host");
        match &nested.animations[0] {
            RuntimeNestedAnimationInstance::Simple {
                is_playing,
                speed,
                mix,
                ..
            } => {
                assert!(*is_playing);
                assert_eq!(*speed, 2.0);
                assert_eq!(*mix, 0.25);
            }
            _ => panic!("expected simple animation"),
        }
        match &nested.animations[1] {
            RuntimeNestedAnimationInstance::Remap { mix, .. } => assert_eq!(*mix, 0.0),
            _ => panic!("expected remap animation"),
        }
    }

    fn synthetic_component(local_id: usize, _graph_order: usize) -> RuntimeComponent {
        RuntimeComponent {
            local_id,
            global_id: local_id as u32,
            type_name: "Node",
            transform_property_keys: crate::components::TransformPropertyKeys::for_type("Node"),
            capabilities: RuntimeComponentCapabilities {
                world_transform: true,
                transform: true,
            },
            parent: None,
            parent_transform: None,
            children: Vec::new(),
            constraints: Vec::new(),
            dependents: Vec::new(),
            collapsables: Vec::new(),
            layout_ancestors: Vec::new(),
            constrained_layout_ancestor: None,
            graph_order: None,
            dirt: ComponentDirt::NONE,
            path_revision: Cell::new(1),
            transform: TransformRuntimeState::default(),
            concrete: crate::components::RuntimeConcreteComponentState::for_type("Node"),
        }
    }

    fn synthetic_component_for_type(local_id: usize, type_name: &'static str) -> RuntimeComponent {
        let mut component = synthetic_component(local_id, local_id);
        let definition =
            nuxie_schema::definition_by_name(type_name).expect("synthetic type exists");
        component.type_name = type_name;
        component.transform_property_keys =
            crate::components::TransformPropertyKeys::for_type(type_name);
        component.capabilities = RuntimeComponentCapabilities {
            world_transform: definition.is_a("WorldTransformComponent"),
            transform: definition.is_a("TransformComponent"),
        };
        component.concrete = crate::components::RuntimeConcreteComponentState::for_type(type_name);
        component
    }

    struct ScriptedLayoutTestInstance {
        measures: Rc<Cell<usize>>,
        resizes: Rc<RefCell<Vec<(f32, f32)>>>,
    }

    #[test]
    fn semantic_layout_bounds_use_current_animation_frame_not_solved_target() {
        let root = synthetic_component_for_type(0, "Artboard");
        let layout = synthetic_component_for_type(1, "LayoutComponent");
        let style = synthetic_component_for_type(2, "LayoutComponentStyle");
        let mut instance = synthetic_instance(vec![root, layout, style], vec![0, 1, 2]);
        synthetic_link_parent(&mut instance, 1, 0);
        synthetic_link_parent(&mut instance, 2, 1);
        let style = instance.component_handle(2).expect("layout style");
        instance
            .component_mut(1)
            .and_then(|component| component.concrete.layout.as_mut())
            .expect("layout state")
            .style = Some(style);

        let current = RuntimeLayoutBounds {
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 80.0,
        };
        instance.retain_runtime_layout_component_bounds(1, current, None);
        instance
            .component(1)
            .and_then(|component| component.concrete.layout.as_ref())
            .expect("layout state")
            .set_animation_style(2, 1, 1.0, None);

        let solved_target = RuntimeLayoutBounds {
            height: 40.0,
            ..current
        };
        instance.retain_runtime_layout_component_bounds(1, solved_target, None);
        instance.solved_layout_bounds = Some(Arc::new(BTreeMap::from([(1, solved_target)])));

        // Pinned C++ asks LayoutComponent::localBounds, which reads the
        // current m_layout frame, before SemanticData's bounds-delta gate:
        // semantic_provider.cpp:13-32,76-94;
        // layout_component.hpp:192-211; semantic_data.cpp:501-531.
        let bounds = crate::semantic_provider::SemanticProvider::semantic_bounds(&mut instance, 1);

        assert_eq!(
            bounds,
            crate::semantic_data::SemanticBounds::new(0.0, 0.0, 100.0, 80.0)
        );
    }

    impl ScriptInstance for ScriptedLayoutTestInstance {
        fn has_method(&self, method: ScriptMethod) -> Result<bool, ScriptError> {
            Ok(matches!(
                method,
                ScriptMethod::Measure | ScriptMethod::Resize
            ))
        }

        fn call_method(
            &mut self,
            method: ScriptMethod,
            args: &[ScriptValue],
            _host: &mut dyn crate::ScriptHost,
        ) -> Result<ScriptValue, ScriptError> {
            match method {
                ScriptMethod::Measure => {
                    self.measures.set(self.measures.get() + 1);
                    Ok(ScriptValue::Vec2 { x: 200.0, y: 150.0 })
                }
                ScriptMethod::Resize => {
                    if let Some(ScriptValue::Vec2 { x, y }) = args.first() {
                        self.resizes.borrow_mut().push((*x, *y));
                    }
                    Ok(ScriptValue::Nil)
                }
                _ => Ok(ScriptValue::Nil),
            }
        }

        fn get_input(&self, _name: &str) -> Result<ScriptValue, ScriptError> {
            Ok(ScriptValue::Nil)
        }

        fn set_input(&mut self, _name: &str, _value: ScriptValue) -> Result<(), ScriptError> {
            Ok(())
        }
    }

    #[test]
    fn scripted_layout_uses_its_parent_node_for_hydration_measure_and_resize() {
        let root = synthetic_component_for_type(0, "Artboard");
        let parent = synthetic_component_for_type(1, "LayoutComponent");
        let scripted = synthetic_component_for_type(2, "ScriptedLayout");
        let style = synthetic_component_for_type(3, "LayoutComponentStyle");
        let mut instance =
            synthetic_instance(vec![root, parent, scripted, style], vec![0, 1, 2, 3]);
        synthetic_link_parent(&mut instance, 1, 0);
        synthetic_link_parent(&mut instance, 2, 1);
        synthetic_link_parent(&mut instance, 3, 1);
        let style = instance.component_handle(3).expect("layout style");
        instance
            .component_mut(1)
            .and_then(|component| component.concrete.layout.as_mut())
            .expect("layout state")
            .style = Some(style);
        let measures = Rc::new(Cell::new(0));
        let resizes = Rc::new(RefCell::new(Vec::new()));
        instance.set_script_instance_for_global_with_implemented_methods(
            2,
            Box::new(ScriptedLayoutTestInstance {
                measures: Rc::clone(&measures),
                resizes: Rc::clone(&resizes),
            }),
            RuntimeScriptImplementedMethods::MEASURE | RuntimeScriptImplementedMethods::RESIZE,
        );

        assert_eq!(instance.scripted_layout_node(2), Some(1));
        assert!(instance.did_hydrate_scripted_layout(2));
        assert!(
            instance
                .component(2)
                .expect("scripted layout component")
                .dirt
                .contains(ComponentDirt::PAINT)
        );
        assert!(
            instance
                .component(1)
                .and_then(|component| component.concrete.layout.as_ref())
                .is_some_and(|layout| layout.layout_node_is_dirty())
        );
        assert_eq!(
            instance.measure_scripted_layout(2, Some(180.0), None),
            (180.0, 150.0)
        );
        assert_eq!(measures.get(), 1);

        instance.retain_runtime_layout_component_bounds(
            1,
            RuntimeLayoutBounds {
                x: 0.0,
                y: 0.0,
                width: 320.0,
                height: 240.0,
            },
            None,
        );
        assert_eq!(&*resizes.borrow(), &[(320.0, 240.0)]);

        let layout = instance
            .component(1)
            .and_then(|component| component.concrete.layout.as_ref())
            .expect("layout state");
        layout.set_animation_style(2, 1, 1.0, None);
        instance.retain_runtime_layout_component_bounds(
            1,
            RuntimeLayoutBounds {
                x: 0.0,
                y: 0.0,
                width: 520.0,
                height: 440.0,
            },
            None,
        );
        let entry = RuntimeAdvancingComponent {
            local_id: 1,
            object: instance.objects.object_handle(1).expect("layout object"),
            component: instance.component_handle(1),
            kind: AdvancingComponentKind::LayoutComponent,
        };
        assert!(instance.advance_layout_component_entry(entry, 0.5, true));
        assert!(instance.advance_layout_component_entry(entry, 0.5, true));
        assert!(!instance.advance_layout_component_entry(entry, 0.1, true));
        assert_eq!(
            &*resizes.borrow(),
            &[
                (320.0, 240.0),
                (320.0, 240.0),
                (420.0, 340.0),
                (520.0, 440.0),
            ]
        );
    }

    #[test]
    fn scripted_layout_resize_requires_a_visible_styled_layout() {
        let root = synthetic_component_for_type(0, "Artboard");
        let parent = synthetic_component_for_type(1, "LayoutComponent");
        let scripted = synthetic_component_for_type(2, "ScriptedLayout");
        let style = synthetic_component_for_type(3, "LayoutComponentStyle");
        let mut instance =
            synthetic_instance(vec![root, parent, scripted, style], vec![0, 1, 2, 3]);
        synthetic_link_parent(&mut instance, 1, 0);
        synthetic_link_parent(&mut instance, 2, 1);
        synthetic_link_parent(&mut instance, 3, 1);
        let resizes = Rc::new(RefCell::new(Vec::new()));
        instance.set_script_instance_for_global_with_implemented_methods(
            2,
            Box::new(ScriptedLayoutTestInstance {
                measures: Rc::new(Cell::new(0)),
                resizes: Rc::clone(&resizes),
            }),
            RuntimeScriptImplementedMethods::RESIZE,
        );

        instance.retain_runtime_layout_component_bounds(
            1,
            RuntimeLayoutBounds {
                x: 0.0,
                y: 0.0,
                width: 100.0,
                height: 80.0,
            },
            None,
        );
        assert!(
            resizes.borrow().is_empty(),
            "style-less layouts do not propagate"
        );

        let style = instance.component_handle(3).expect("layout style");
        instance
            .component_mut(1)
            .expect("layout component")
            .concrete
            .layout
            .as_mut()
            .expect("layout state")
            .style = Some(style);

        let drawable_flags =
            property_key_for_name("LayoutComponent", "drawableFlags").expect("drawable flags");
        assert!(instance.set_uint_property(1, drawable_flags, 1));
        instance.retain_runtime_layout_component_bounds(
            1,
            RuntimeLayoutBounds {
                x: 0.0,
                y: 0.0,
                width: 150.0,
                height: 120.0,
            },
            None,
        );
        assert!(
            resizes.borrow().is_empty(),
            "hidden drawable layouts do not propagate"
        );

        assert!(instance.set_uint_property(1, drawable_flags, 0));
        let display =
            property_key_for_name("LayoutComponentStyle", "displayValue").expect("display");
        assert!(instance.set_uint_property(3, display, 1));
        instance.retain_runtime_layout_component_bounds(
            1,
            RuntimeLayoutBounds {
                x: 0.0,
                y: 0.0,
                width: 175.0,
                height: 140.0,
            },
            None,
        );
        assert!(
            resizes.borrow().is_empty(),
            "display:none layouts do not propagate"
        );

        assert!(instance.set_uint_property(3, display, 0));
        instance.component_mut(1).expect("layout component").dirt |= ComponentDirt::COLLAPSED;
        instance.retain_runtime_layout_component_bounds(
            1,
            RuntimeLayoutBounds {
                x: 0.0,
                y: 0.0,
                width: 200.0,
                height: 160.0,
            },
            None,
        );
        assert!(
            resizes.borrow().is_empty(),
            "collapsed layouts do not propagate"
        );
    }

    #[derive(Clone, Copy)]
    enum ScriptedLayoutMeasureBehavior {
        Missing,
        Nil,
        Error,
    }

    struct ScriptedLayoutMeasureInstance(ScriptedLayoutMeasureBehavior);

    impl ScriptInstance for ScriptedLayoutMeasureInstance {
        fn has_method(&self, method: ScriptMethod) -> Result<bool, ScriptError> {
            Ok(method == ScriptMethod::Measure
                && !matches!(self.0, ScriptedLayoutMeasureBehavior::Missing))
        }

        fn call_method(
            &mut self,
            method: ScriptMethod,
            _args: &[ScriptValue],
            _host: &mut dyn crate::ScriptHost,
        ) -> Result<ScriptValue, ScriptError> {
            assert_eq!(method, ScriptMethod::Measure);
            match self.0 {
                ScriptedLayoutMeasureBehavior::Missing => {
                    panic!("a missing measure function must not be called")
                }
                ScriptedLayoutMeasureBehavior::Nil => Ok(ScriptValue::Nil),
                ScriptedLayoutMeasureBehavior::Error => Err(ScriptError::new("measure failed")),
            }
        }

        fn get_input(&self, _name: &str) -> Result<ScriptValue, ScriptError> {
            Ok(ScriptValue::Nil)
        }

        fn set_input(&mut self, _name: &str, _value: ScriptValue) -> Result<(), ScriptError> {
            Ok(())
        }
    }

    #[test]
    fn scripted_layout_measure_distinguishes_missing_from_invalid_callbacks() {
        let measure = |behavior| {
            let root = synthetic_component_for_type(0, "Artboard");
            let parent = synthetic_component_for_type(1, "LayoutComponent");
            let scripted = synthetic_component_for_type(2, "ScriptedLayout");
            let mut instance = synthetic_instance(vec![root, parent, scripted], vec![0, 1, 2]);
            synthetic_link_parent(&mut instance, 1, 0);
            synthetic_link_parent(&mut instance, 2, 1);
            instance.set_script_instance_for_global_with_implemented_methods(
                2,
                Box::new(ScriptedLayoutMeasureInstance(behavior)),
                RuntimeScriptImplementedMethods::MEASURE,
            );
            instance.measure_scripted_layout(2, Some(180.0), None)
        };

        assert_eq!(measure(ScriptedLayoutMeasureBehavior::Missing), (0.0, 0.0));
        assert_eq!(
            measure(ScriptedLayoutMeasureBehavior::Nil),
            (180.0, f32::MAX)
        );
        assert_eq!(
            measure(ScriptedLayoutMeasureBehavior::Error),
            (180.0, f32::MAX)
        );
    }

    fn synthetic_typed_component(local_id: usize, type_name: &'static str) -> RuntimeComponent {
        let definition = definition_by_name(type_name).expect("synthetic Component type");
        let mut component = synthetic_component(local_id, local_id);
        component.type_name = type_name;
        component.transform_property_keys =
            crate::components::TransformPropertyKeys::for_type(type_name);
        component.capabilities = RuntimeComponentCapabilities {
            world_transform: definition.is_a("WorldTransformComponent"),
            transform: definition.is_a("TransformComponent"),
        };
        component.concrete = crate::components::RuntimeConcreteComponentState::for_type(type_name);
        component
    }

    fn synthetic_link_parent(instance: &mut ArtboardInstance, child: usize, parent: usize) {
        let child = instance.component_handle(child).expect("synthetic child");
        let parent = instance.component_handle(parent).expect("synthetic parent");
        assert!(instance.objects.link_parent(child, parent));
        if instance
            .objects
            .component(child)
            .is_some_and(|component| component.capabilities.transform)
            && instance
                .objects
                .component(parent)
                .is_some_and(|component| component.capabilities.world_transform)
        {
            instance
                .objects
                .component_mut(child)
                .expect("synthetic child remains live")
                .parent_transform = Some(parent);
        }
    }

    fn synthetic_add_dependent(instance: &mut ArtboardInstance, source: usize, dependent: usize) {
        let source = instance.component_handle(source).expect("synthetic source");
        let dependent = instance
            .component_handle(dependent)
            .expect("synthetic dependent");
        assert!(instance.objects.add_dependent(source, dependent));
    }

    #[test]
    fn layout_drawable_hit_test_preserves_skip_hidden_and_parent_fallback() {
        let mut instance = synthetic_instance(
            vec![synthetic_component_for_type(0, "LayoutComponent")],
            vec![0],
        );
        let layout = instance.component_handle(0).expect("layout handle");
        instance
            .component(0)
            .and_then(|component| component.concrete.layout.as_ref())
            .expect("layout owner")
            .retain_bounds(0.0, 0.0, 100.0, 50.0);

        assert!(instance.component_hit_test_point(layout, (25.0, 25.0), false, true));
        assert!(!instance.component_hit_test_point(layout, (125.0, 25.0), false, true));
        assert!(
            instance.component_hit_test_point(layout, (125.0, 25.0), true, true),
            "skipOnUnclipped bypasses only an unclipped Layout's local bounds"
        );

        let drawable_flags =
            property_key_for_name("LayoutComponent", "drawableFlags").expect("Drawable flags");
        assert!(instance.set_uint_property(0, drawable_flags, 1));
        assert!(
            !instance.component_hit_test_point(layout, (25.0, 25.0), false, true),
            "Drawable::hitTestPoint rejects its sole generated Hidden flag"
        );
    }

    #[test]
    fn node_computed_local_uses_settled_world_and_retained_parent_transform() {
        let mut instance = synthetic_instance(
            vec![
                synthetic_typed_component(0, "Node"),
                synthetic_typed_component(1, "Node"),
            ],
            vec![0, 1],
        );
        synthetic_link_parent(&mut instance, 1, 0);
        let parent_world = Mat2D([2.0, 0.0, 0.0, 3.0, 10.0, 20.0]);
        let expected_local = Mat2D([1.0, 0.0, 0.0, 1.0, 4.0, 5.0]);
        instance.component_mut(0).unwrap().transform.world_transform = parent_world;
        instance.component_mut(1).unwrap().transform.world_transform =
            parent_world.multiply(expected_local);
        instance
            .component(1)
            .unwrap()
            .concrete
            .node
            .as_ref()
            .unwrap()
            .mark_computed_local_dirty();

        assert_eq!(
            instance.runtime_node_computed_local_transform(1),
            Some(expected_local)
        );

        instance.component_mut(0).unwrap().transform.world_transform =
            Mat2D([0.0, 0.0, 0.0, 0.0, 7.0, 9.0]);
        instance
            .component(1)
            .unwrap()
            .concrete
            .node
            .as_ref()
            .unwrap()
            .mark_computed_local_dirty();
        assert_eq!(
            instance.runtime_node_computed_local_transform(1),
            Some(Mat2D::IDENTITY)
        );

        instance.component_mut(1).unwrap().parent_transform = None;
        instance
            .component(1)
            .unwrap()
            .concrete
            .node
            .as_ref()
            .unwrap()
            .mark_computed_local_dirty();
        assert_eq!(
            instance.runtime_node_computed_local_transform(1),
            Some(Mat2D::IDENTITY)
        );
    }

    #[test]
    fn transform_render_opacity_uses_cpp_child_opacity_dispatch() {
        let mut world_parent = synthetic_instance(
            vec![
                synthetic_typed_component(0, "Artboard"),
                synthetic_typed_component(1, "Node"),
            ],
            vec![0, 1],
        );
        synthetic_link_parent(&mut world_parent, 1, 0);
        assert!(
            world_parent
                .objects
                .set_double_property_by_name(0, "opacity", 0.5)
        );
        assert!(
            world_parent
                .objects
                .set_double_property_by_name(1, "opacity", 0.8)
        );
        let root = world_parent.component_handle(0).unwrap();
        world_parent.update_component(root, ComponentDirt::RENDER_OPACITY);
        let child = world_parent.component_handle(1).unwrap();
        world_parent.update_component(child, ComponentDirt::RENDER_OPACITY);
        assert_eq!(
            world_parent.component(1).unwrap().transform.render_opacity,
            0.4
        );

        let mut transform_parent = synthetic_instance(
            vec![
                synthetic_typed_component(0, "Node"),
                synthetic_typed_component(1, "Node"),
            ],
            vec![0, 1],
        );
        synthetic_link_parent(&mut transform_parent, 1, 0);
        assert!(
            transform_parent
                .objects
                .set_double_property_by_name(0, "opacity", 0.5)
        );
        assert!(
            transform_parent
                .objects
                .set_double_property_by_name(1, "opacity", 0.8)
        );
        transform_parent
            .component_mut(0)
            .unwrap()
            .transform
            .render_opacity = 0.25;
        let child = transform_parent.component_handle(1).unwrap();
        transform_parent.update_component(child, ComponentDirt::RENDER_OPACITY);
        assert_eq!(
            transform_parent
                .component(1)
                .unwrap()
                .transform
                .render_opacity,
            0.2
        );
    }

    #[test]
    fn skin_on_dirty_targets_only_the_retained_skinnable_dirt_family() {
        for (type_name, expected) in [
            ("PointsPath", ComponentDirt::PATH),
            ("Mesh", ComponentDirt::VERTICES),
        ] {
            let mut instance = synthetic_instance(
                vec![
                    synthetic_typed_component(0, type_name),
                    synthetic_typed_component(1, "Skin"),
                ],
                vec![0, 1],
            );
            synthetic_link_parent(&mut instance, 1, 0);
            let skinnable = instance.component_handle(0).unwrap();
            let skin = instance.component_handle(1).unwrap();
            instance
                .component_mut(0)
                .unwrap()
                .concrete
                .skinnable
                .as_mut()
                .unwrap()
                .skin = Some(skin);
            instance
                .component_mut(1)
                .unwrap()
                .concrete
                .skin
                .as_mut()
                .unwrap()
                .skinnable = Some(skinnable);
            instance.component_mut(0).unwrap().dirt = ComponentDirt::NONE;
            instance.component_mut(1).unwrap().dirt = ComponentDirt::NONE;
            instance.set_artboard_dirt_for_test(ComponentDirt::NONE);

            assert!(instance.add_component_dirt(skin, ComponentDirt::SKIN, false));
            assert_eq!(instance.component(1).unwrap().dirt, ComponentDirt::SKIN);
            assert!(instance.component(0).unwrap().dirt.contains(expected));
            let other = if expected == ComponentDirt::PATH {
                ComponentDirt::VERTICES
            } else {
                ComponentDirt::PATH
            };
            assert!(!instance.component(0).unwrap().dirt.contains(other));
        }
    }

    #[test]
    fn skin_update_rebuilds_only_its_retained_tendon_buffer() {
        let mut instance = synthetic_instance(
            vec![
                synthetic_typed_component(0, "RootBone"),
                synthetic_typed_component(1, "Tendon"),
                synthetic_typed_component(2, "Skin"),
                synthetic_typed_component(3, "Mesh"),
            ],
            vec![0, 1, 2, 3],
        );
        let bone = instance.component_handle(0).unwrap();
        let tendon = instance.component_handle(1).unwrap();
        let skin = instance.component_handle(2).unwrap();
        let bone_world = Mat2D([2.0, 0.0, 0.0, 3.0, 4.0, 5.0]);
        let inverse_bind = Mat2D([1.0, 0.0, 0.0, 1.0, -1.0, -2.0]);
        instance.component_mut(0).unwrap().transform.world_transform = bone_world;
        {
            let tendon_state = instance
                .component_mut(1)
                .unwrap()
                .concrete
                .tendon
                .as_mut()
                .unwrap();
            tendon_state.bone = Some(bone);
            tendon_state.inverse_bind = inverse_bind;
        }
        {
            let skin_state = instance
                .component_mut(2)
                .unwrap()
                .concrete
                .skin
                .as_mut()
                .unwrap();
            skin_state.tendons.push(tendon);
            skin_state.bone_transforms = vec![Mat2D::IDENTITY; 2];
        }

        instance.update_component(skin, ComponentDirt::FILTHY);
        let expected = bone_world.multiply(inverse_bind);
        let state = instance
            .component(2)
            .unwrap()
            .concrete
            .skin
            .as_ref()
            .unwrap();
        assert_eq!(state.bone_transforms, vec![Mat2D::IDENTITY, expected]);
        assert_eq!(state.buffer_rebuilds, 1);

        instance.component_mut(0).unwrap().transform.world_transform =
            Mat2D([1.0, 0.0, 0.0, 1.0, 9.0, 11.0]);
        let state = instance
            .component(2)
            .unwrap()
            .concrete
            .skin
            .as_ref()
            .unwrap();
        assert_eq!(state.bone_transforms[1], expected);
        assert_eq!(state.buffer_rebuilds, 1);

        instance.update_component(skin, ComponentDirt::SKIN);
        let state = instance
            .component(2)
            .unwrap()
            .concrete
            .skin
            .as_ref()
            .unwrap();
        assert_eq!(
            state.bone_transforms[1],
            Mat2D([1.0, 0.0, 0.0, 1.0, 9.0, 11.0]).multiply(inverse_bind)
        );
        assert_eq!(state.buffer_rebuilds, 2);
    }

    fn callback_route_animation(has_keyed_callbacks: bool) -> RuntimeLinearAnimation {
        let events = has_keyed_callbacks
            .then(|| {
                vec![StateMachineReportedEvent {
                    event_local_index: 0,
                    event_core_type: 0,
                    name: Some("callback".to_owned()),
                    url: None,
                    target: None,
                    properties: Vec::new(),
                    string_properties: Vec::new(),
                    context: None,
                    seconds_delay: 0.0,
                }]
            })
            .unwrap_or_default();
        callback_route_animation_with_events(events)
    }

    fn callback_route_animation_with_events(
        events: Vec<StateMachineReportedEvent>,
    ) -> RuntimeLinearAnimation {
        let has_keyed_callbacks = !events.is_empty();
        let keyed_objects = events
            .into_iter()
            .enumerate()
            .map(|(index, event)| RuntimeKeyedObject {
                global_id: 1 + index as u32 * 3,
                object_id: event.event_local_index(),
                target_local_id: event.event_local_index(),
                keyed_properties: vec![RuntimeKeyedProperty {
                    global_id: 2 + index as u32 * 3,
                    property_key: 0,
                    target: RuntimeKeyedPropertyTarget::Callback {
                        event_local_index: Some(event.event_local_index()),
                    },
                    key_frames: vec![RuntimeKeyFrame::Callback(RuntimeKeyFrameCallback {
                        global_id: 3 + index as u32 * 3,
                        frame: 1,
                        seconds: 1.0,
                    })],
                }],
            })
            .collect::<Vec<_>>();
        RuntimeLinearAnimation {
            global_id: 0,
            name: None,
            fps: 1,
            duration: 2,
            speed: 1.0,
            loop_value: 0,
            work_start: 0,
            work_end: 2,
            enable_work_area: false,
            quantize: false,
            keyed_objects: Arc::new(keyed_objects),
            key_frame_data_bind_templates: Arc::new(Vec::new()),
            has_keyed_callbacks,
        }
    }

    fn add_nested_simple_transform_curve(animation: &mut RuntimeLinearAnimation) {
        let mut keyed_objects = animation.keyed_objects.as_ref().clone();
        keyed_objects.push(RuntimeKeyedObject {
            global_id: 100,
            object_id: 1,
            target_local_id: 1,
            keyed_properties: vec![RuntimeKeyedProperty {
                global_id: 101,
                property_key: property_key_for_name("Node", "x").expect("Node.x"),
                target: RuntimeKeyedPropertyTarget::Double {
                    transform_property: Some(TransformProperty::X),
                },
                key_frames: vec![
                    RuntimeKeyFrame::Double(RuntimeKeyFrameDouble {
                        global_id: 102,
                        frame: 0,
                        seconds: 0.0,
                        interpolation_type: 1,
                        interpolator_id: None,
                        interpolator: None,
                        value: 0.0,
                    }),
                    RuntimeKeyFrame::Double(RuntimeKeyFrameDouble {
                        global_id: 103,
                        frame: 2,
                        seconds: 2.0,
                        interpolation_type: 0,
                        interpolator_id: None,
                        interpolator: None,
                        value: 10.0,
                    }),
                ],
            }],
        });
        animation.keyed_objects = Arc::new(keyed_objects);
    }

    #[test]
    fn production_nested_simple_animation_delivers_each_callback_chain_before_mix() {
        let event_core_type = u32::from(
            definition_by_name("Event")
                .expect("Event definition")
                .type_key
                .int,
        );
        let event = StateMachineReportedEvent {
            event_local_index: 2,
            event_core_type,
            name: Some("timeline-event".to_owned()),
            url: None,
            target: None,
            properties: Vec::new(),
            string_properties: Vec::new(),
            context: None,
            seconds_delay: 0.0,
        };
        let (audio, audio_core_type) = nested_audio_event(3);
        let mut child = synthetic_instance(
            vec![
                synthetic_component_for_type(0, "Artboard"),
                synthetic_component_for_type(1, "Node"),
                synthetic_component_for_type(2, "Event"),
                synthetic_component_for_type(3, "AudioEvent"),
            ],
            vec![0, 1, 2, 3],
        );
        let mut timeline = callback_route_animation_with_events(vec![event, audio]);
        add_nested_simple_transform_curve(&mut timeline);
        child.linear_animations = Arc::new(vec![timeline]);

        let mut callback_probe_child = child.clone();
        let mut callback_probe_animation = callback_probe_child
            .linear_animation_instance(0)
            .expect("callback-order probe animation");
        let callback_observations = Rc::new(RefCell::new(Vec::new()));
        let callback_observations_sink = Rc::clone(&callback_observations);
        assert!(
            callback_probe_child.advance_linear_animation_instance_with_callback_sink(
                &mut callback_probe_animation,
                1.25,
                &mut |artboard, event| {
                    if event.is_some() {
                        callback_observations_sink.borrow_mut().push(
                            artboard
                                .transform_property(1, TransformProperty::X)
                                .expect("callback-time Node.x"),
                        );
                    }
                    false
                },
            )
        );
        assert_eq!(
            *callback_observations.borrow(),
            [0.0, 0.0],
            "both callbacks observe authored state because no animation mix has run yet",
        );
        assert!(
            callback_probe_child.apply_linear_animation_instance(&callback_probe_animation, 1.0,)
        );
        assert_eq!(
            callback_probe_child.transform_property(1, TransformProperty::X),
            Some(6.25),
            "the exact callback-sink path applies the sampled transform only after delivery",
        );

        let mut nested_target_definition = empty_state_machine(102);
        nested_target_definition.inputs = Arc::new(vec![Some(
            RuntimeStateMachineInput::new_number(1, Some("nested-observed".to_owned()), 0.0),
        )]);
        let nested_target = StateMachineInstance::new(0, &nested_target_definition, &mut child);
        child.state_machines = Arc::new(vec![nested_target_definition]);
        let animation = child
            .linear_animation_instance(0)
            .expect("nested simple animation instance");
        let mut nested = synthetic_nested_artboard_instance(101);
        nested.child = Box::new(child);
        nested
            .animations
            .push(RuntimeNestedAnimationInstance::Simple {
                local_id: 2,
                animation,
                is_playing: true,
                speed: 1.0,
                mix: 1.0,
            });
        nested
            .animations
            .push(RuntimeNestedAnimationInstance::StateMachine(
                RuntimeNestedStateMachineInstance::new(3, nested_target, Vec::new()),
            ));

        let mut root = synthetic_instance(
            vec![
                synthetic_component_for_type(0, "Artboard"),
                synthetic_component_for_type(1, "NestedArtboard"),
                synthetic_component_for_type(4, "NestedNumber"),
            ],
            vec![0, 1, 4],
        );
        let parent_id_key =
            property_key_for_name("Component", "parentId").expect("Component.parentId");
        let input_id_key =
            property_key_for_name("NestedInput", "inputId").expect("NestedInput.inputId");
        assert!(root.objects.set_uint_property(4, parent_id_key, 3));
        assert!(root.objects.set_uint_property(4, input_id_key, 0));
        root.nested_artboards.insert(1, nested);
        let mut definition = empty_state_machine(100);
        definition.inputs = Arc::new(vec![Some(RuntimeStateMachineInput::new_number(
            1,
            Some("observed".to_owned()),
            0.0,
        ))]);
        definition.listeners = Arc::new(vec![RuntimeStateMachineListener {
            name: None,
            target_local_id: 1,
            is_single: true,
            listener_types: vec![RuntimeListenerType::Event],
            event_local_indices: vec![2],
            view_model_path: None,
            view_model_input_types: Vec::new(),
            gamepad_input_types: Vec::new(),
            keyboard_input_types: Vec::new(),
            semantic_input_types: Vec::new(),
            hit_paths: Vec::new(),
            listener_actions: vec![
                RuntimeScheduledListenerAction::NumberChange(
                    RuntimeListenerNumberChange::for_test(
                        0,
                        RuntimeListenerInputTarget {
                            direct_input_index: Some(0),
                            nested_input_local_id: None,
                        },
                        7.0,
                    ),
                ),
                RuntimeScheduledListenerAction::NumberChange(
                    RuntimeListenerNumberChange::for_test(
                        0,
                        RuntimeListenerInputTarget {
                            direct_input_index: None,
                            nested_input_local_id: Some(4),
                        },
                        9.0,
                    ),
                ),
            ],
        }]);
        root.state_machines = Arc::new(vec![definition]);
        let definitions = Arc::clone(&root.state_machines);
        let mut root_machine = StateMachineInstance::new(0, &definitions[0], &mut root);

        let trace = RuntimeNestedEventChainTrace::start();
        let notify_batches = RuntimeNestedNotifyBatchTrace::start();
        root_machine
            .advance_and_apply(&mut root, 1.25)
            .expect("nested simple animation production advance");
        let steps = trace.finish();
        let notify_batches = notify_batches.finish();

        assert_eq!(
            root_machine
                .input(0)
                .and_then(StateMachineInputInstance::number_value),
            Some(7.0),
            "the parent Event listener fires for the nested simple timeline Event",
        );
        assert_eq!(
            root_machine.audio_event_seam_receipt(),
            (1, Some((3, audio_core_type))),
            "the parent audio seam receives exactly the nested simple timeline AudioEvent",
        );
        assert_eq!(
            root.nested_state_machine(3)
                .and_then(|machine| machine.input(0))
                .and_then(StateMachineInputInstance::number_value),
            Some(9.0),
            "the parent callback can mutate a same-host nested state-machine occurrence while that host is actively advancing",
        );
        assert_eq!(
            root.nested_artboards
                .get(&1)
                .and_then(|nested| { nested.child.transform_property(1, TransformProperty::X) }),
            Some(6.25),
            "the child finishes with the nonzero nested-simple mix after both callback chains",
        );
        assert_eq!(
            steps
                .iter()
                .map(|step| (step.source_local_id, step.phase))
                .collect::<Vec<_>>(),
            [
                (2, RuntimeNestedEventChainPhase::SourceLocal),
                (2, RuntimeNestedEventChainPhase::AncestorDispatch),
                (2, RuntimeNestedEventChainPhase::AudioUnwind),
                (2, RuntimeNestedEventChainPhase::SourceLocal),
                (2, RuntimeNestedEventChainPhase::AncestorDispatch),
                (2, RuntimeNestedEventChainPhase::AudioUnwind),
            ],
            "each crossed callback completes its singleton local/ancestor/audio chain before the next callback and before the animation mix",
        );
        assert_eq!(
            notify_batches
                .iter()
                .map(|batch| batch.size)
                .collect::<Vec<_>>(),
            [1, 1],
            "the real Rust notify entry receives one singleton batch per crossed callback",
        );
    }

    #[test]
    fn animation_advance_routes_only_callback_definitions_through_event_reporting() {
        for (has_keyed_callbacks, expected_events) in [(false, 0), (true, 1)] {
            let mut artboard =
                synthetic_instance(vec![synthetic_component_for_type(0, "Event")], vec![0]);
            artboard.linear_animations =
                Arc::new(vec![callback_route_animation(has_keyed_callbacks)]);
            let mut animation = artboard
                .linear_animation_instance(0)
                .expect("test animation instance");
            let mut events = Vec::new();

            assert!(artboard.advance_linear_animation_instance_with_events(
                &mut animation,
                1.0,
                &mut events,
            ));
            assert_eq!(animation.time(), 1.0);
            assert_eq!(events.len(), expected_events);
        }
    }

    #[test]
    fn keyed_event_callback_projects_the_live_event_occurrence() {
        let mut artboard =
            synthetic_instance(vec![synthetic_component_for_type(0, "Event")], vec![0]);
        let name_key = property_key_for_name("Event", "name").expect("Event.name");
        assert!(artboard.set_string_property(0, name_key, b"live-after-import".to_vec()));
        artboard.linear_animations = Arc::new(vec![callback_route_animation(true)]);
        let mut animation = artboard.linear_animation_instance(0).expect("animation");
        let mut events = Vec::new();

        assert!(artboard.advance_linear_animation_instance_with_events(
            &mut animation,
            1.0,
            &mut events,
        ));
        assert_eq!(events[0].name(), Some("live-after-import"));
    }

    #[test]
    fn update_pass_repolls_data_binds_before_each_late_joystick_and_final_components() {
        let mut artboard = synthetic_instance(vec![synthetic_component(0, 0)], vec![0]);
        artboard.joysticks_apply_before_update = false;
        let root = artboard.component_handle(0).expect("root component");
        artboard.joysticks = vec![
            RuntimeJoystick::test_fixture(root, false),
            RuntimeJoystick::test_fixture(root, true),
            RuntimeJoystick::test_fixture(root, false),
        ];

        artboard.update_pass();
        assert_eq!(
            artboard.update_pass_data_bind_call_count, 5,
            "initial, two late-joystick, final component, and did-update source passes each poll DataBinds"
        );
    }

    #[test]
    fn component_settlement_journals_generic_semantic_bounds_dirt_once() {
        let mut artboard = synthetic_instance(
            vec![
                synthetic_component_for_type(0, "Artboard"),
                synthetic_component_for_type(1, "Node"),
            ],
            vec![0, 1],
        );

        assert!(artboard.add_dirt(
            1,
            ComponentDirt::WORLD_TRANSFORM | ComponentDirt::PATH,
            false,
        ));
        assert!(artboard.update_pass());
        assert_eq!(
            artboard.take_semantic_bounds_dirty_locals(),
            BTreeSet::from([1]),
            "SemanticData::update receives its geometry owner's settled WorldTransform/Path dirt",
        );
        assert!(
            artboard.take_semantic_bounds_dirty_locals().is_empty(),
            "the journal is occurrence-local and consumed by one semantic synchronization",
        );
    }

    #[test]
    fn root_advance_settles_components_and_reports_retained_component_dirt() {
        let mut artboard =
            synthetic_instance(vec![synthetic_component_for_type(0, "Artboard")], vec![0]);
        artboard.install_persistent_dirt_component_fixture();

        assert!(artboard.advance(0.25).expect("root advance"));
        assert_eq!(
            artboard.persistent_dirt_component_fixture_receipt(),
            (1, 1, false),
            "advanceInternal precedes updatePass; root advance returns the union of work and any dirt retained after settlement"
        );
    }

    #[test]
    fn artboard_clones_share_one_linear_animation_definition_arena() {
        let mut artboard = synthetic_instance(vec![synthetic_component(0, 0)], vec![0]);
        artboard.linear_animations = Arc::new(vec![callback_route_animation(false)]);

        let cloned = artboard.clone();
        let first = artboard
            .linear_animation_instance(0)
            .expect("first occurrence");
        let second = cloned
            .linear_animation_instance(0)
            .expect("cloned occurrence");

        assert!(Arc::ptr_eq(
            &artboard.linear_animations,
            &cloned.linear_animations
        ));
        assert_eq!(first.animation_index(), second.animation_index());
    }

    #[test]
    fn state_machine_outer_settlement_preserves_deep_nested_artboard_opacity() {
        let typed_component = |local_id: usize, graph_order: usize, type_name: &'static str| {
            let mut component = synthetic_component(local_id, graph_order);
            component.type_name = type_name;
            component.transform_property_keys =
                crate::components::TransformPropertyKeys::for_type(type_name);
            component
        };

        let mut leaf_root = typed_component(0, 0, "Artboard");
        leaf_root.transform.render_opacity = 0.0;
        let mut leaf = synthetic_instance(vec![leaf_root], vec![0]);
        let opacity_key = property_key_for_name("Artboard", "opacity").expect("opacity key");
        assert!(leaf.set_double_property(0, opacity_key, 0.0));
        leaf.clear_component_dirt(0);
        leaf.set_artboard_dirt_for_test(ComponentDirt::NONE);

        let mut middle_root = typed_component(0, 0, "Artboard");
        middle_root.transform.render_opacity = 1.0;
        let mut middle_host = typed_component(1, 1, "NestedArtboard");
        middle_host.dirt = ComponentDirt::RENDER_OPACITY;
        let mut middle = synthetic_instance(vec![middle_root, middle_host], vec![1]);
        synthetic_link_parent(&mut middle, 1, 0);
        let mut leaf_mount = synthetic_nested_artboard_instance(2);
        leaf_mount.child = Box::new(leaf);
        middle.nested_artboards.insert(1, leaf_mount);
        middle.nested_artboard_locals.push(1);
        middle.set_artboard_dirt_for_test(ComponentDirt::COMPONENTS);

        let mut root_component = typed_component(0, 0, "Artboard");
        root_component.transform.render_opacity = 1.0;
        let mut root_host = typed_component(1, 1, "NestedArtboard");
        root_host.transform.render_opacity = 1.0;
        root_host.dirt = ComponentDirt::COMPONENTS;
        let mut root = synthetic_instance(vec![root_component, root_host], vec![1]);
        synthetic_link_parent(&mut root, 1, 0);
        let mut middle_mount = synthetic_nested_artboard_instance(1);
        middle_mount.child = Box::new(middle);
        root.nested_artboards.insert(1, middle_mount);
        root.nested_artboard_locals.push(1);

        root.update_pass();

        let middle = root
            .nested_artboards
            .values()
            .next()
            .expect("middle occurrence");
        let leaf = middle
            .child
            .nested_artboards
            .values()
            .next()
            .expect("leaf occurrence");
        let leaf_root = leaf.child.component(0).expect("leaf root component");
        assert_eq!(leaf_root.transform.render_opacity, 0.0);
        assert_eq!(leaf.child.host_opacity, 1.0);
        assert_eq!(leaf.child.child_opacity(), 0.0);
        assert!(!leaf_root.dirt.contains(ComponentDirt::RENDER_OPACITY));

        root.settle_state_machine_update_passes();

        let middle = root
            .nested_artboards
            .values()
            .next()
            .expect("middle occurrence");
        let leaf = middle
            .child
            .nested_artboards
            .values()
            .next()
            .expect("leaf occurrence");
        let leaf_root = leaf.child.component(0).expect("leaf root component");
        assert_eq!(leaf_root.transform.render_opacity, 0.0);
        assert_eq!(leaf.child.host_opacity, 1.0);
        assert_eq!(leaf.child.child_opacity(), 0.0);
        assert!(!leaf_root.dirt.contains(ComponentDirt::RENDER_OPACITY));
    }

    #[test]
    fn paused_nested_artboard_flushes_child_dirt_after_host_opacity_changes() {
        let typed_component = |local_id: usize, graph_order: usize, type_name: &'static str| {
            let mut component = synthetic_component(local_id, graph_order);
            component.type_name = type_name;
            component.transform_property_keys =
                crate::components::TransformPropertyKeys::for_type(type_name);
            component
        };

        let mut child_root = typed_component(0, 0, "Artboard");
        child_root.transform.render_opacity = 1.0;
        let mut child_content = typed_component(1, 1, "Shape");
        child_content.transform.render_opacity = 1.0;
        let mut child = synthetic_instance(vec![child_root, child_content], vec![0, 1]);
        synthetic_link_parent(&mut child, 1, 0);
        synthetic_add_dependent(&mut child, 0, 1);
        child.clear_component_dirt(0);
        child.clear_component_dirt(1);
        child.set_artboard_dirt_for_test(ComponentDirt::NONE);

        let root = typed_component(0, 0, "Artboard");
        let mut host = typed_component(1, 1, "NestedArtboard");
        host.transform.render_opacity = 0.5;
        let mut parent = synthetic_instance(vec![root, host], vec![1]);
        synthetic_link_parent(&mut parent, 1, 0);
        let mut nested = synthetic_nested_artboard_instance(2);
        nested.child = Box::new(child);
        nested.is_paused = true;
        parent.nested_artboards.insert(1, nested);

        let mut script_mode = RuntimeScriptUpdateMode::HostOnly;
        assert!(parent.update_nested_artboard_from_host_dirt(
            1,
            ComponentDirt::RENDER_OPACITY,
            &mut script_mode,
            Mat2D::IDENTITY,
        ));

        let child = &parent.nested_artboards.get(&1).unwrap().child;
        assert_eq!(child.component(0).unwrap().transform.render_opacity, 1.0);
        assert_eq!(child.host_opacity, 0.5);
        assert_eq!(child.child_opacity(), 0.5);
        assert_eq!(child.component(1).unwrap().transform.render_opacity, 0.5);
        assert!(!child.has_dirt(ComponentDirt::COMPONENTS));
        assert!(
            child
                .components()
                .iter()
                .all(|component| !component.dirt.contains(ComponentDirt::RENDER_OPACITY))
        );
    }

    #[test]
    fn component_dirt_bits_match_cpp_layout() {
        assert_eq!(ComponentDirt::NONE.0, 0);
        assert_eq!(ComponentDirt::COLLAPSED.0, 1 << 0);
        assert_eq!(ComponentDirt::DEPENDENTS.0, 1 << 1);
        assert_eq!(ComponentDirt::COMPONENTS.0, 1 << 2);
        assert_eq!(ComponentDirt::DRAW_ORDER.0, 1 << 3);
        assert_eq!(ComponentDirt::PATH.0, 1 << 4);
        assert_eq!(ComponentDirt::TEXT_SHAPE.0, ComponentDirt::PATH.0);
        assert_eq!(ComponentDirt::SKIN.0, ComponentDirt::PATH.0);
        assert_eq!(ComponentDirt::VERTICES.0, 1 << 5);
        assert_eq!(ComponentDirt::TEXT_COVERAGE.0, ComponentDirt::VERTICES.0);
        assert_eq!(ComponentDirt::TRANSFORM.0, 1 << 6);
        assert_eq!(ComponentDirt::WORLD_TRANSFORM.0, 1 << 7);
        assert_eq!(ComponentDirt::RENDER_OPACITY.0, 1 << 8);
        assert_eq!(ComponentDirt::PAINT.0, 1 << 9);
        assert_eq!(ComponentDirt::STOPS.0, 1 << 10);
        assert_eq!(ComponentDirt::LAYOUT_STYLE.0, 1 << 11);
        assert_eq!(ComponentDirt::BINDINGS.0, 1 << 12);
        assert_eq!(ComponentDirt::N_SLICER.0, 1 << 13);
        assert_eq!(ComponentDirt::SCRIPT_UPDATE.0, 1 << 14);
        assert_eq!(ComponentDirt::CLIPPING.0, 1 << 15);
        assert_eq!(ComponentDirt::FILTHY.0, 0xFFFE);
    }

    #[test]
    fn retained_reset_schedule_recurses_into_nested_artboards() {
        let property_value_key = property_key_for_name("CustomPropertyTrigger", "propertyValue")
            .expect("trigger property value");
        let mut child = synthetic_instance(
            vec![synthetic_component_for_type(0, "CustomPropertyTrigger")],
            vec![0],
        );
        assert!(child.set_uint_property(0, property_value_key, 3));

        let mut parent = synthetic_instance(
            vec![synthetic_component_for_type(0, "NestedArtboard")],
            vec![0],
        );
        let mut nested = synthetic_nested_artboard_instance(1);
        nested.child = Box::new(child);
        parent.nested_artboards.insert(0, nested);

        parent.reset_retained_components_for_state_machine_settlement();

        assert_eq!(
            parent
                .nested_artboards
                .get(&0)
                .and_then(|nested| { nested.child.uint_property(0, property_value_key) }),
            Some(0),
            "Artboard::reset walks the authored reset schedule recursively \
             (`artboard.cpp:1483-1493`; `nested_artboard.cpp:1035-1043`)"
        );
    }

    #[test]
    fn component_list_reset_visits_unmounted_logical_rows() {
        let (file, _, _) = owned_view_model_action_fixture(9_706, false);
        let context = RuntimeOwnedViewModelHandle::new(
            RuntimeOwnedViewModelInstance::new(&file, 1).expect("row context"),
        );
        assert!(context.borrow_mut().set_trigger_by_property_index(2, 1));
        let mut list = RuntimeConstrainableListState::default();
        list.logical_items.push(RuntimeComponentListLogicalItem {
            occurrence_identity: 41,
            context: context.clone(),
            size: (100.0, 20.0),
            mapped_artboard_global: None,
        });
        assert!(
            list.items.is_empty(),
            "logical row is deliberately unmounted"
        );

        reset_component_list_instances(&mut list, true);

        assert_eq!(
            context.borrow().trigger_value_by_property_path(&[2]),
            Some(0),
            "C++ acknowledges every logical m_listItems VMI before the \
             optional m_artboardInstancesMap lookup \
             (`artboard_component_list.cpp:888-920`)"
        );
    }

    #[test]
    fn range_mapper_reverse_conversion_swaps_input_and_output_ranges() {
        let converter = RuntimeDataBindGraphConverter::RangeMapper {
            global_id: 0,
            min_input: 0.0,
            max_input: 10.0,
            min_output: 100.0,
            max_output: 200.0,
            flags: 0,
            interpolation_type: 1,
            interpolator: None,
        };

        let Some(RuntimeDataBindGraphValue::Number(value)) =
            runtime_data_bind_graph_reverse_convert_value(
                &converter,
                &RuntimeDataBindGraphValue::Number(160.0),
            )
        else {
            panic!("range mapper reverse conversion did not return a number");
        };

        assert!(
            (value - 6.0).abs() <= 0.0001,
            "range mapper reverse conversion mismatch: expected 6, got {value}"
        );
    }

    #[test]
    fn range_mapper_reverse_conversion_preserves_reverse_flag() {
        let converter = RuntimeDataBindGraphConverter::RangeMapper {
            global_id: 0,
            min_input: 0.0,
            max_input: 10.0,
            min_output: 100.0,
            max_output: 200.0,
            flags: 1 << 3,
            interpolation_type: 1,
            interpolator: None,
        };

        let Some(RuntimeDataBindGraphValue::Number(value)) =
            runtime_data_bind_graph_reverse_convert_value(
                &converter,
                &RuntimeDataBindGraphValue::Number(160.0),
            )
        else {
            panic!("range mapper reverse conversion did not return a number");
        };

        assert!(
            (value - 4.0).abs() <= 0.0001,
            "range mapper reverse conversion mismatch: expected 4, got {value}"
        );
    }

    #[test]
    fn add_dirt_recurses_to_graph_dependents() {
        let source = synthetic_component(0, 0);
        let first_dependent = synthetic_component(1, 1);
        let second_dependent = synthetic_component(2, 2);
        let mut instance = synthetic_instance(
            vec![source, first_dependent, second_dependent],
            vec![0, 1, 2],
        );
        synthetic_add_dependent(&mut instance, 0, 2);
        synthetic_add_dependent(&mut instance, 0, 1);
        let source_handle = instance.component_handle(0).expect("source handle");
        let second_handle = instance
            .component_handle(2)
            .expect("second dependent handle");
        assert!(!instance.objects.add_dependent(source_handle, second_handle));
        assert_eq!(
            instance.objects.dependent_at(source_handle, 0),
            instance.component_handle(2)
        );
        assert_eq!(
            instance.objects.dependent_at(source_handle, 1),
            instance.component_handle(1)
        );
        assert_eq!(instance.objects.dependent_len(source_handle), 2);

        instance.set_artboard_dirt_for_test(ComponentDirt::NONE);
        assert!(!instance.has_dirt(ComponentDirt::COMPONENTS));
        assert!(instance.add_dirt(0, ComponentDirt::PATH, true));
        assert!(
            instance
                .component(0)
                .unwrap()
                .dirt
                .contains(ComponentDirt::PATH | ComponentDirt::COMPONENTS),
            "The root Artboard is the same retained Component dirt owner that publishes Components loop-control dirt (`src/component.cpp:32-45`; `src/artboard.cpp:1205-1241`)"
        );
        assert!(
            instance
                .component(1)
                .unwrap()
                .dirt
                .contains(ComponentDirt::PATH)
        );
        assert!(
            instance
                .component(2)
                .unwrap()
                .dirt
                .contains(ComponentDirt::PATH)
        );
        assert!(instance.has_dirt(ComponentDirt::COMPONENTS));

        assert!(!instance.add_dirt(0, ComponentDirt::PATH, true));
        assert!(instance.add_dirt(0, ComponentDirt::PAINT, false));
        assert!(
            instance
                .component(0)
                .unwrap()
                .dirt
                .contains(ComponentDirt::PATH | ComponentDirt::PAINT)
        );
    }

    #[test]
    fn occurrence_dependency_sort_matches_cpp_diamond_visitation_order() {
        let mut instance = synthetic_instance(
            vec![
                synthetic_component(0, 0),
                synthetic_component(1, 1),
                synthetic_component(2, 2),
                synthetic_component(3, 3),
            ],
            Vec::new(),
        );
        synthetic_add_dependent(&mut instance, 0, 1);
        synthetic_add_dependent(&mut instance, 0, 2);
        synthetic_add_dependent(&mut instance, 1, 3);
        synthetic_add_dependent(&mut instance, 2, 3);

        assert!(instance.objects.sort_dependencies_from_root());
        assert_eq!(
            instance
                .objects
                .dependency_order()
                .iter()
                .filter_map(|handle| instance.objects.component_local_id(*handle))
                .collect::<Vec<_>>(),
            vec![0, 2, 1, 3],
            "DependencySorter visits retained dependents in insertion order and front-inserts each completed owner (`src/dependency_sorter.cpp:6-48`)"
        );
    }

    #[test]
    fn path_effect_dependencies_are_selected_at_their_authored_component_slots() {
        let nodes = vec![
            DependencyNode {
                node_id: 0,
                kind: DependencyNodeKind::PathComposer {
                    shape_local: 3,
                    shape_global: 30,
                },
            },
            DependencyNode {
                node_id: 1,
                kind: DependencyNodeKind::Component {
                    local_id: 7,
                    global_id: 70,
                    type_name: "Feather",
                    name: None,
                },
            },
        ];
        let edge = nuxie_graph::DependencyNodeEdge {
            source_node: 0,
            dependent_node: 1,
            kind: nuxie_graph::DependencyKind::FeatherPathBuilder,
        };

        assert!(dependency_edge_targets_component(
            &edge,
            &nodes,
            nuxie_graph::DependencyKind::FeatherPathBuilder,
            7,
        ));
        assert!(!dependency_edge_targets_component(
            &edge,
            &nodes,
            nuxie_graph::DependencyKind::FeatherPathBuilder,
            8,
        ));
        assert!(!dependency_edge_targets_component(
            &edge,
            &nodes,
            nuxie_graph::DependencyKind::LinearGradientPaintContainer,
            7,
        ));
    }

    #[test]
    fn occurrence_dependency_sort_publishes_cpp_partial_order_on_cycle() {
        let mut instance = synthetic_instance(
            vec![
                synthetic_component(0, 0),
                synthetic_component(1, 1),
                synthetic_component(2, 2),
            ],
            Vec::new(),
        );
        synthetic_add_dependent(&mut instance, 0, 2);
        synthetic_add_dependent(&mut instance, 0, 1);
        synthetic_add_dependent(&mut instance, 1, 0);

        assert!(!instance.objects.sort_dependencies_from_root());
        assert_eq!(
            instance
                .objects
                .dependency_order()
                .iter()
                .filter_map(|handle| instance.objects.component_local_id(*handle))
                .collect::<Vec<_>>(),
            vec![2],
            "DependencySorter::sort ignores visit's cycle result and publishes owners completed before the cyclic branch (`src/dependency_sorter.cpp:6-10`)"
        );
        assert_eq!(
            instance
                .component(2)
                .and_then(|component| component.graph_order),
            Some(crate::components::GraphOrder::new(0))
        );
        assert_eq!(
            instance
                .component(0)
                .and_then(|component| component.graph_order),
            None
        );
        assert_eq!(
            instance
                .component(1)
                .and_then(|component| component.graph_order),
            None
        );
    }

    #[test]
    fn enabling_empty_layout_constraint_bounds_does_not_dirty_layout_dependents() {
        let mut layout = synthetic_component(0, 0);
        layout.type_name = "LayoutComponent";
        let dependent = synthetic_component(1, 1);
        let mut instance = synthetic_instance(vec![layout, dependent], vec![0, 1]);
        synthetic_add_dependent(&mut instance, 0, 1);
        instance.set_artboard_dirt_for_test(ComponentDirt::NONE);
        let prepared_epoch = instance.prepared_epoch();

        instance.enable_layout_constraint_bounds();

        assert!(instance.layout_constraint_bounds_enabled);
        assert!(
            !instance
                .component(0)
                .unwrap()
                .dirt
                .contains(ComponentDirt::WORLD_TRANSFORM)
        );
        assert!(
            !instance
                .component(1)
                .unwrap()
                .dirt
                .contains(ComponentDirt::WORLD_TRANSFORM)
        );
        assert_eq!(instance.prepared_epoch(), prepared_epoch);

        let cache_epoch = instance.cache_epoch();
        instance.enable_layout_constraint_bounds();
        assert_eq!(instance.cache_epoch(), cache_epoch);
    }

    #[test]
    fn enabling_layout_constraint_bounds_dirties_layout_dependents() {
        let mut layout = synthetic_component(0, 0);
        layout.type_name = "LayoutComponent";
        let dependent = synthetic_component(1, 1);
        let mut instance = synthetic_instance(vec![layout, dependent], vec![0, 1]);
        synthetic_add_dependent(&mut instance, 0, 1);
        instance.layout_constraint_bounds = Some(Arc::new(BTreeMap::from([(
            0,
            RuntimeLayoutBounds {
                x: 0.0,
                y: 0.0,
                width: 10.0,
                height: 10.0,
            },
        )])));
        instance.set_artboard_dirt_for_test(ComponentDirt::NONE);

        instance.enable_layout_constraint_bounds();

        assert!(
            instance
                .component(0)
                .unwrap()
                .dirt
                .contains(ComponentDirt::WORLD_TRANSFORM)
        );
        assert!(
            instance
                .component(1)
                .unwrap()
                .dirt
                .contains(ComponentDirt::WORLD_TRANSFORM)
        );
    }

    #[test]
    fn retained_layout_owner_keeps_parent_local_coordinates_for_constraint_offsets() {
        let root = synthetic_component_for_type(0, "Artboard");
        let parent = synthetic_component_for_type(1, "LayoutComponent");
        let child = synthetic_component_for_type(2, "LayoutComponent");
        let mut instance = synthetic_instance(vec![root, parent, child], vec![0, 1, 2]);
        synthetic_link_parent(&mut instance, 1, 0);
        synthetic_link_parent(&mut instance, 2, 1);
        let accumulated = BTreeMap::from([
            (
                0,
                RuntimeLayoutBounds {
                    x: 0.0,
                    y: 0.0,
                    width: 300.0,
                    height: 200.0,
                },
            ),
            (
                1,
                RuntimeLayoutBounds {
                    x: 11.0,
                    y: 19.0,
                    width: 100.0,
                    height: 80.0,
                },
            ),
            (
                2,
                RuntimeLayoutBounds {
                    x: 28.0,
                    y: 42.0,
                    width: 40.0,
                    height: 30.0,
                },
            ),
        ]);

        instance.retain_runtime_layout_component_bounds(2, accumulated[&2], Some(&accumulated));
        let layout = instance
            .component(2)
            .and_then(|component| component.concrete.layout.as_ref())
            .expect("retained LayoutComponent state");
        assert_eq!(layout.transform_property(TransformProperty::X), Some(17.0));
        assert_eq!(layout.transform_property(TransformProperty::Y), Some(23.0));
    }

    #[test]
    fn layout_world_update_uses_retained_position_and_root_origin() {
        // LayoutComponent::update replaces TransformComponent's authored
        // local matrix with retained Yoga left/top and subtracts the root
        // Artboard normalized origin before multiplying by its parent
        // (`src/layout/layout_component.cpp:82-121`).
        let root = synthetic_component_for_type(0, "Artboard");
        let layout = synthetic_component_for_type(1, "LayoutComponent");
        let mut instance = synthetic_instance(vec![root, layout], vec![0, 1]);
        synthetic_link_parent(&mut instance, 1, 0);
        instance.width = 100.0;
        instance.height = 80.0;
        instance.origin_x = 0.5;
        instance.origin_y = 0.25;
        instance.component_mut(0).unwrap().transform.world_transform =
            Mat2D([2.0, 0.0, 0.0, 3.0, 10.0, 20.0]);
        instance
            .component(1)
            .and_then(|component| component.concrete.layout.as_ref())
            .expect("retained LayoutComponent state")
            .retain_bounds(60.0, 70.0, 40.0, 30.0);

        let layout = instance.component_handle(1).unwrap();
        instance.update_component(layout, ComponentDirt::WORLD_TRANSFORM);

        assert_eq!(
            instance.component(1).unwrap().transform.world_transform,
            Mat2D([2.0, 0.0, 0.0, 3.0, 30.0, 170.0])
        );
    }

    #[test]
    fn layout_ancestry_retains_exact_nearest_first_owner_identity() {
        // C++ `Drawable::isChildOfLayout` walks from the queried Component
        // through concrete parent pointers and compares LayoutComponent
        // identity (`src/drawable.cpp:45-59`). The Rust occurrence retains the
        // same identities instead of the displaced boolean-only mirror.
        let root = synthetic_component_for_type(0, "Artboard");
        let outer = synthetic_component_for_type(1, "LayoutComponent");
        let inner = synthetic_component_for_type(2, "LayoutComponent");
        let shape = synthetic_component_for_type(3, "Shape");
        let mut instance = synthetic_instance(vec![root, outer, inner, shape], vec![0, 1, 2, 3]);
        synthetic_link_parent(&mut instance, 1, 0);
        synthetic_link_parent(&mut instance, 2, 1);
        synthetic_link_parent(&mut instance, 3, 2);
        crate::components::retain_runtime_component_layout_topology(&mut instance.objects);

        let outer = instance.component_handle(1).expect("outer layout");
        let inner = instance.component_handle(2).expect("inner layout");
        assert_eq!(
            instance.component(3).expect("shape").layout_ancestors,
            vec![inner, outer]
        );
        assert_eq!(
            instance.component(2).expect("inner").layout_ancestors,
            vec![inner, outer],
            "C++ begins the identity walk at `this`"
        );
        assert!(
            instance
                .component(0)
                .expect("root")
                .layout_ancestors
                .is_empty()
        );
    }

    #[test]
    fn nested_layout_constraint_space_refreshes_for_parent_or_child_layout_generation() {
        let mut host = synthetic_component(0, 0);
        host.type_name = "NestedArtboardLayout";
        let mut parent = synthetic_instance(vec![host], vec![0]);

        let mut child_layout = synthetic_component(0, 0);
        child_layout.type_name = "LayoutComponent";
        let mut child =
            synthetic_instance(vec![child_layout, synthetic_component(1, 1)], vec![0, 1]);
        synthetic_add_dependent(&mut child, 0, 1);
        let mut nested = synthetic_nested_artboard_instance(7);
        nested.child = Box::new(child);
        parent.nested_artboards.insert(0, nested);

        let assigned_bounds = RuntimeLayoutBounds {
            x: 4.0,
            y: 5.0,
            width: 120.0,
            height: 80.0,
        };
        let layout_bounds = BTreeMap::from([(0, assigned_bounds)]);
        let first_parent_layout = RuntimeNestedLayoutBoundsCacheKey {
            graph_global_id: 3,
            layout_revision: 9,
        };

        assert!(parent.apply_nested_artboard_layout_bounds(
            0,
            Some(&layout_bounds),
            first_parent_layout,
        ));
        let first_transfer_key = parent.nested_artboards[&0].layout_data_transfer_key;
        let first_cache_epoch = parent.nested_artboards[&0].child.cache_epoch();

        assert!(!parent.apply_nested_artboard_layout_bounds(
            0,
            Some(&layout_bounds),
            first_parent_layout,
        ));
        assert_eq!(
            parent.nested_artboards[&0].child.cache_epoch(),
            first_cache_epoch
        );

        // The assigned root writes from the first transfer are already part of
        // the stored generation (the identical apply above stabilized). A
        // later child layout change emulates C++ bubbling
        // `markHostingLayoutDirty` back to the owner of the Yoga node.
        parent
            .nested_artboards
            .get_mut(&0)
            .expect("nested child")
            .child
            .mark_layout_changed();
        assert!(parent.apply_nested_artboard_layout_bounds(
            0,
            Some(&layout_bounds),
            first_parent_layout,
        ));
        let after_child_refresh = parent.nested_artboards[&0].child.cache_epoch();
        let after_child_transfer_key = parent.nested_artboards[&0].layout_data_transfer_key;
        assert_ne!(after_child_transfer_key, first_transfer_key);
        assert!(!parent.apply_nested_artboard_layout_bounds(
            0,
            Some(&layout_bounds),
            first_parent_layout,
        ));
        assert_eq!(
            parent.nested_artboards[&0].child.cache_epoch(),
            after_child_refresh
        );

        let next_parent_layout = RuntimeNestedLayoutBoundsCacheKey {
            layout_revision: first_parent_layout.layout_revision + 1,
            ..first_parent_layout
        };
        assert!(parent.apply_nested_artboard_layout_bounds(
            0,
            Some(&layout_bounds),
            next_parent_layout,
        ));
        assert_eq!(
            parent.nested_artboards[&0]
                .layout_data_transfer_key
                .expect("refreshed transfer")
                .parent_layout,
            next_parent_layout
        );
        assert_eq!(
            parent.nested_artboards[&0].child.cache_epoch(),
            after_child_refresh,
            "an unchanged assigned rectangle must not dirty the child world tree"
        );
    }

    #[test]
    fn path_dirt_tracks_geometry_revision_separately_from_draw_cache_epoch() {
        let component = synthetic_component(0, 0);
        let mut instance = synthetic_instance(vec![component], vec![0]);

        let initial_path_epoch = instance.path_epoch();
        let initial_cache_epoch = instance.cache_epoch();
        assert!(instance.add_dirt(0, ComponentDirt::PAINT, false));
        assert_eq!(instance.path_epoch(), initial_path_epoch);
        assert!(instance.cache_epoch() > initial_cache_epoch);

        let paint_cache_epoch = instance.cache_epoch();
        assert!(instance.add_dirt(0, ComponentDirt::PATH, false));
        assert!(instance.path_epoch() > initial_path_epoch);
        assert!(instance.cache_epoch() > paint_cache_epoch);

        let path_epoch = instance.path_epoch();
        assert!(!instance.add_dirt(0, ComponentDirt::PATH, false));
        assert_eq!(instance.path_epoch(), path_epoch);

        assert!(instance.collapse_component(0, true));
        assert!(instance.path_epoch() > path_epoch);
    }

    #[test]
    fn world_transform_dirt_invalidates_world_state_without_rebuilding_paths() {
        let component = synthetic_component(0, 0);
        let mut instance = synthetic_instance(vec![component], vec![0]);

        let initial_path_epoch = instance.path_epoch();
        let initial_prepared_epoch = instance.prepared_epoch();

        assert!(instance.add_dirt(0, ComponentDirt::WORLD_TRANSFORM, false));

        assert_eq!(instance.path_epoch(), initial_path_epoch);
        assert!(instance.prepared_epoch() > initial_prepared_epoch);
    }

    #[test]
    fn effect_callbacks_do_not_publish_synthetic_path_epochs() {
        let mut trim = synthetic_component(0, 0);
        trim.type_name = "TrimPath";
        let mut instance = synthetic_instance(vec![trim], vec![0]);
        instance.objects = InstanceObjectArena::from_runtime_objects(vec![Some(
            synthetic_runtime_object(0, "TrimPath", Vec::new()),
        )]);
        let trim_start = property_key_for_name("TrimPath", "start").expect("TrimPath.start");
        let trim_mode = property_key_for_name("TrimPath", "modeValue").expect("TrimPath.modeValue");

        let path_epoch = instance.path_epoch();
        assert!(instance.set_double_property(0, trim_start, 0.25));
        assert_eq!(instance.path_epoch(), path_epoch);

        assert!(instance.set_uint_property(0, trim_mode, 2));
        assert_eq!(instance.path_epoch(), path_epoch);

        let mut dash_path = synthetic_component(0, 0);
        dash_path.type_name = "DashPath";
        let mut instance = synthetic_instance(vec![dash_path], vec![0]);
        instance.objects = InstanceObjectArena::from_runtime_objects(vec![Some(
            synthetic_runtime_object(0, "DashPath", Vec::new()),
        )]);
        let offset_is_percentage = property_key_for_name("DashPath", "offsetIsPercentage")
            .expect("DashPath.offsetIsPercentage");

        let path_epoch = instance.path_epoch();
        assert!(instance.set_bool_property(0, offset_is_percentage, true));
        assert_eq!(instance.path_epoch(), path_epoch);

        let mut dash = synthetic_component(0, 0);
        dash.type_name = "Dash";
        let mut instance = synthetic_instance(vec![dash], vec![0]);
        instance.objects = InstanceObjectArena::from_runtime_objects(vec![Some(
            synthetic_runtime_object(0, "Dash", Vec::new()),
        )]);
        let length = property_key_for_name("Dash", "length").expect("Dash.length");

        let path_epoch = instance.path_epoch();
        assert!(instance.set_double_property(0, length, 4.0));
        assert_eq!(instance.path_epoch(), path_epoch);

        let mut feather = synthetic_component(0, 0);
        feather.type_name = "Feather";
        let mut instance = synthetic_instance(vec![feather], vec![0]);
        instance.objects = InstanceObjectArena::from_runtime_objects(vec![Some(
            synthetic_runtime_object(0, "Feather", Vec::new()),
        )]);
        let inner = property_key_for_name("Feather", "inner").expect("Feather.inner");
        let space_value =
            property_key_for_name("Feather", "spaceValue").expect("Feather.spaceValue");

        let path_epoch = instance.path_epoch();
        let prepared_epoch = instance.prepared_epoch();
        assert!(instance.set_bool_property(0, inner, true));
        assert_eq!(instance.path_epoch(), path_epoch);
        assert_eq!(instance.prepared_epoch(), prepared_epoch);

        assert!(instance.set_uint_property(0, space_value, 1));
        assert_eq!(instance.path_epoch(), path_epoch);
        assert_eq!(instance.prepared_epoch(), prepared_epoch);
    }

    #[test]
    fn layout_revision_tracks_layout_dirt_separately_from_draw_cache_epoch() {
        let component = synthetic_component(0, 0);
        let mut instance = synthetic_instance(vec![component], vec![0]);

        let initial_layout_revision = instance.layout_revision();
        let initial_cache_epoch = instance.cache_epoch();
        assert!(instance.add_dirt(0, ComponentDirt::PAINT, false));
        assert_eq!(instance.layout_revision(), initial_layout_revision);
        assert!(instance.cache_epoch() > initial_cache_epoch);

        let paint_cache_epoch = instance.cache_epoch();
        assert!(instance.add_dirt(0, ComponentDirt::LAYOUT_STYLE, false));
        assert!(instance.layout_revision() > initial_layout_revision);
        assert!(instance.cache_epoch() > paint_cache_epoch);

        let layout_revision = instance.layout_revision();
        assert!(!instance.add_dirt(0, ComponentDirt::LAYOUT_STYLE, false));
        assert_eq!(instance.layout_revision(), layout_revision);
    }

    #[test]
    fn layout_style_padding_write_dirties_only_retained_parent_layout_node() {
        let mut instance = synthetic_instance(
            vec![
                synthetic_component_for_type(0, "Artboard"),
                synthetic_component_for_type(1, "LayoutComponent"),
                synthetic_component_for_type(2, "LayoutComponentStyle"),
                synthetic_component_for_type(3, "LayoutComponent"),
            ],
            vec![0, 1, 2, 3],
        );
        synthetic_link_parent(&mut instance, 1, 0);
        synthetic_link_parent(&mut instance, 2, 1);
        synthetic_link_parent(&mut instance, 3, 0);
        for local_id in 0..4 {
            instance.clear_component_dirt(local_id);
        }

        let padding_left = property_key_for_name("LayoutComponentStyle", "paddingLeft")
            .expect("LayoutComponentStyle.paddingLeft");
        let prepared_epoch = instance.prepared_epoch();
        assert!(instance.set_double_property(2, padding_left, 12.0));

        let parent = instance
            .component(1)
            .and_then(|component| component.concrete.layout.as_ref())
            .expect("parent layout owner");
        let sibling = instance
            .component(3)
            .and_then(|component| component.concrete.layout.as_ref())
            .expect("sibling layout owner");
        assert!(parent.layout_node_is_dirty());
        assert_eq!(parent.layout_node_revision(), 1);
        assert!(!sibling.layout_node_is_dirty());
        assert!(
            instance
                .component(0)
                .unwrap()
                .dirt
                .contains(ComponentDirt::COMPONENTS)
        );
        assert_eq!(
            instance.prepared_epoch(),
            prepared_epoch,
            "layout-node publication must not dirty unrelated paint preparation"
        );

        let revision = instance.layout_revision();
        assert!(!instance.set_double_property(2, padding_left, 12.0));
        assert_eq!(instance.layout_revision(), revision);
    }

    #[test]
    fn animated_justify_self_write_dirties_only_retained_parent_layout_node() {
        let mut instance = synthetic_instance(
            vec![
                synthetic_component_for_type(0, "Artboard"),
                synthetic_component_for_type(1, "LayoutComponent"),
                synthetic_component_for_type(2, "LayoutComponentStyle"),
                synthetic_component_for_type(3, "LayoutComponent"),
            ],
            vec![0, 1, 2, 3],
        );
        synthetic_link_parent(&mut instance, 1, 0);
        synthetic_link_parent(&mut instance, 2, 1);
        synthetic_link_parent(&mut instance, 3, 0);
        for local_id in 0..4 {
            instance.clear_component_dirt(local_id);
        }

        let justify_self = property_key_for_name("LayoutComponentStyle", "justifySelfValue")
            .expect("LayoutComponentStyle.justifySelfValue");
        let prepared_epoch = instance.prepared_epoch();
        assert!(instance.set_uint_property(2, justify_self, 1));

        let parent = instance
            .component(1)
            .and_then(|component| component.concrete.layout.as_ref())
            .expect("parent layout owner");
        let sibling = instance
            .component(3)
            .and_then(|component| component.concrete.layout.as_ref())
            .expect("sibling layout owner");
        assert!(parent.layout_node_is_dirty());
        assert_eq!(parent.layout_node_revision(), 1);
        assert!(!sibling.layout_node_is_dirty());
        assert!(
            instance
                .component(0)
                .unwrap()
                .dirt
                .contains(ComponentDirt::COMPONENTS)
        );
        assert_eq!(
            instance.prepared_epoch(),
            prepared_epoch,
            "layout-node publication must not dirty unrelated paint preparation"
        );

        let revision = instance.layout_revision();
        assert!(!instance.set_uint_property(2, justify_self, 1));
        assert_eq!(instance.layout_revision(), revision);
    }

    #[test]
    fn animated_justify_items_write_dirties_only_retained_parent_layout_node() {
        let mut instance = synthetic_instance(
            vec![
                synthetic_component_for_type(0, "Artboard"),
                synthetic_component_for_type(1, "LayoutComponent"),
                synthetic_component_for_type(2, "LayoutComponentStyle"),
                synthetic_component_for_type(3, "LayoutComponent"),
            ],
            vec![0, 1, 2, 3],
        );
        synthetic_link_parent(&mut instance, 1, 0);
        synthetic_link_parent(&mut instance, 2, 1);
        synthetic_link_parent(&mut instance, 3, 0);
        for local_id in 0..4 {
            instance.clear_component_dirt(local_id);
        }

        let justify_items = property_key_for_name("LayoutComponentStyle", "justifyItemsValue")
            .expect("LayoutComponentStyle.justifyItemsValue");
        let prepared_epoch = instance.prepared_epoch();
        assert!(instance.set_uint_property(2, justify_items, 1));

        let parent = instance
            .component(1)
            .and_then(|component| component.concrete.layout.as_ref())
            .expect("parent layout owner");
        let sibling = instance
            .component(3)
            .and_then(|component| component.concrete.layout.as_ref())
            .expect("sibling layout owner");
        assert!(parent.layout_node_is_dirty());
        assert_eq!(parent.layout_node_revision(), 1);
        assert!(!sibling.layout_node_is_dirty());
        assert!(
            instance
                .component(0)
                .unwrap()
                .dirt
                .contains(ComponentDirt::COMPONENTS)
        );
        assert_eq!(
            instance.prepared_epoch(),
            prepared_epoch,
            "layout-node publication must not dirty unrelated paint preparation"
        );

        let revision = instance.layout_revision();
        assert!(!instance.set_uint_property(2, justify_items, 1));
        assert_eq!(instance.layout_revision(), revision);
    }

    #[test]
    fn layout_type_write_dirties_retained_parent_layout_and_its_layout_children() {
        let mut instance = synthetic_instance(
            vec![
                synthetic_component_for_type(0, "Artboard"),
                synthetic_component_for_type(1, "LayoutComponent"),
                synthetic_component_for_type(2, "LayoutComponentStyle"),
                synthetic_component_for_type(3, "LayoutComponent"),
                synthetic_component_for_type(4, "LayoutComponent"),
            ],
            vec![0, 1, 2, 3, 4],
        );
        synthetic_link_parent(&mut instance, 1, 0);
        synthetic_link_parent(&mut instance, 2, 1);
        synthetic_link_parent(&mut instance, 3, 1);
        synthetic_link_parent(&mut instance, 4, 0);
        for local_id in 0..5 {
            instance.clear_component_dirt(local_id);
        }

        let layout_type = property_key_for_name("LayoutComponentStyle", "layoutTypeValue")
            .expect("LayoutComponentStyle.layoutTypeValue");
        let prepared_epoch = instance.prepared_epoch();
        assert!(instance.set_uint_property(2, layout_type, 1));

        let parent = instance
            .component(1)
            .and_then(|component| component.concrete.layout.as_ref())
            .expect("parent layout owner");
        let child = instance
            .component(3)
            .and_then(|component| component.concrete.layout.as_ref())
            .expect("child layout owner");
        let sibling = instance
            .component(4)
            .and_then(|component| component.concrete.layout.as_ref())
            .expect("sibling layout owner");
        assert!(parent.layout_node_is_dirty());
        assert_eq!(parent.layout_node_revision(), 1);
        assert!(child.layout_node_is_dirty());
        assert_eq!(child.layout_node_revision(), 1);
        assert!(!sibling.layout_node_is_dirty());
        assert!(
            instance
                .component(0)
                .unwrap()
                .dirt
                .contains(ComponentDirt::COMPONENTS)
        );
        assert_eq!(
            instance.prepared_epoch(),
            prepared_epoch,
            "layout-node publication must not dirty unrelated paint preparation"
        );

        let revision = instance.layout_revision();
        assert!(!instance.set_uint_property(2, layout_type, 1));
        assert_eq!(instance.layout_revision(), revision);
    }

    #[test]
    fn fractional_height_write_dirties_retained_layout_node() {
        let mut instance = synthetic_instance(
            vec![synthetic_component_for_type(0, "LayoutComponent")],
            vec![0],
        );
        instance.clear_component_dirt(0);

        let fractional_height = property_key_for_name("LayoutComponent", "fractionalHeight")
            .expect("LayoutComponent.fractionalHeight");
        assert!(instance.set_double_property(0, fractional_height, 0.75));
        assert!(
            instance
                .component(0)
                .and_then(|component| component.concrete.layout.as_ref())
                .is_some_and(|layout| layout.layout_node_is_dirty())
        );
    }

    #[test]
    fn text_style_metrics_follow_owning_text_into_layout_without_sibling_dirt() {
        let mut instance = synthetic_instance(
            vec![
                synthetic_component_for_type(0, "Artboard"),
                synthetic_component_for_type(1, "LayoutComponent"),
                synthetic_component_for_type(2, "Text"),
                synthetic_component_for_type(3, "TextStylePaint"),
                synthetic_component_for_type(4, "LayoutComponent"),
            ],
            vec![0, 1, 2, 3, 4],
        );
        synthetic_link_parent(&mut instance, 1, 0);
        synthetic_link_parent(&mut instance, 2, 1);
        synthetic_link_parent(&mut instance, 3, 2);
        synthetic_link_parent(&mut instance, 4, 0);
        for local_id in 0..5 {
            instance.clear_component_dirt(local_id);
        }

        let font_size = property_key_for_name("TextStyle", "fontSize").expect("TextStyle.fontSize");
        assert!(instance.set_double_property(3, font_size, 24.0));
        let text_dirt = instance.component(2).expect("text").dirt;
        assert!(text_dirt.contains(ComponentDirt::PATH));
        assert!(text_dirt.contains(ComponentDirt::WORLD_TRANSFORM));
        assert!(
            instance
                .component(1)
                .and_then(|component| component.concrete.layout.as_ref())
                .is_some_and(|layout| layout.layout_node_is_dirty())
        );
        assert!(
            instance
                .component(4)
                .and_then(|component| component.concrete.layout.as_ref())
                .is_some_and(|layout| !layout.layout_node_is_dirty())
        );

        let revision = instance.layout_revision();
        assert!(!instance.set_double_property(3, font_size, 24.0));
        assert_eq!(instance.layout_revision(), revision);
    }

    #[test]
    fn text_vertical_trim_passthrough_dirties_shape_and_layout() {
        // The generated top/bottom fields are bitmask passthrough setters for
        // Text.verticalTrimValue. Their concrete callback is still
        // Text::verticalTrimValueChanged, which invalidates shape and layout
        // (`src/text/text.cpp:1403-1408`; generated text_base.hpp:225-241).
        for property in ["verticalTrimTopValue", "verticalTrimBottomValue"] {
            let mut instance = synthetic_instance(
                vec![
                    synthetic_component_for_type(0, "Artboard"),
                    synthetic_component_for_type(1, "LayoutComponent"),
                    synthetic_component_for_type(2, "Text"),
                ],
                vec![0, 1, 2],
            );
            synthetic_link_parent(&mut instance, 1, 0);
            synthetic_link_parent(&mut instance, 2, 1);
            for local_id in 0..3 {
                instance.clear_component_dirt(local_id);
            }

            let key = property_key_for_name("Text", property).expect("vertical trim passthrough");
            assert!(instance.set_uint_property(2, key, 1));
            let text_dirt = instance.component(2).expect("text").dirt;
            assert!(text_dirt.contains(ComponentDirt::PATH));
            assert!(text_dirt.contains(ComponentDirt::WORLD_TRANSFORM));
            assert!(
                instance
                    .component(1)
                    .and_then(|component| component.concrete.layout.as_ref())
                    .is_some_and(|layout| layout.layout_node_is_dirty())
            );
        }
    }

    #[test]
    fn position_only_layout_change_dirties_world_transform() {
        let mut instance = synthetic_instance(
            vec![
                synthetic_component_for_type(0, "Artboard"),
                synthetic_component_for_type(1, "LayoutComponent"),
            ],
            vec![0, 1],
        );
        synthetic_link_parent(&mut instance, 1, 0);
        instance.retain_runtime_layout_component_bounds(
            1,
            RuntimeLayoutBounds {
                x: 10.0,
                y: 20.0,
                width: 100.0,
                height: 50.0,
            },
            None,
        );
        for local_id in 0..2 {
            instance.clear_component_dirt(local_id);
        }

        instance.retain_runtime_layout_component_bounds(
            1,
            RuntimeLayoutBounds {
                x: 10.0,
                y: 25.0,
                width: 100.0,
                height: 50.0,
            },
            None,
        );

        let layout = instance.component(1).expect("layout component");
        assert!(layout.dirt.contains(ComponentDirt::WORLD_TRANSFORM));
        assert!(!layout.dirt.contains(ComponentDirt::PATH));
    }

    #[test]
    fn joystick_generated_callback_publishes_only_root_components_dirt() {
        let mut root = synthetic_component(0, 0);
        root.type_name = "Artboard";
        let mut joystick = synthetic_component(1, 1);
        joystick.type_name = "Joystick";
        let mut instance = synthetic_instance(vec![root.clone(), joystick.clone()], vec![0, 1]);
        let x_key = property_key_for_name("Joystick", "x").expect("Joystick.x");
        let mut objects = InstanceObjectArena::from_runtime_objects(vec![
            Some(synthetic_runtime_object(0, "Artboard", Vec::new())),
            Some(synthetic_runtime_object(
                1,
                "Joystick",
                vec![RuntimeProperty {
                    key: x_key,
                    name: "x",
                    owner: "Joystick",
                    value: FieldValue::Double(0.0),
                }],
            )),
        ]);
        objects
            .attach_component(0, root)
            .expect("synthetic root component");
        objects
            .attach_component(1, joystick)
            .expect("synthetic joystick component");
        objects.set_dependency_order(vec![
            objects.component_handle(0).expect("root handle"),
            objects.component_handle(1).expect("joystick handle"),
        ]);
        instance.objects = objects;
        instance.clear_component_dirt(0);
        instance.clear_component_dirt(1);

        let cache_epoch = instance.cache_epoch();
        assert!(instance.set_double_property(1, x_key, 0.5));

        assert!(instance.cache_epoch() > cache_epoch);
        assert_eq!(
            instance.component(0).expect("root component").dirt,
            ComponentDirt::COMPONENTS
        );
        assert_eq!(
            instance.component(1).expect("joystick component").dirt,
            ComponentDirt::NONE
        );
    }

    #[test]
    fn solid_color_changes_keep_prepared_topology_epoch_stable() {
        let mut solid = synthetic_component(0, 0);
        solid.type_name = "SolidColor";
        let mut instance = synthetic_instance(vec![solid], vec![0]);
        let color_key =
            property_key_for_name("SolidColor", "colorValue").expect("SolidColor.colorValue");
        instance.objects =
            InstanceObjectArena::from_runtime_objects(vec![Some(synthetic_runtime_object(
                0,
                "SolidColor",
                vec![RuntimeProperty {
                    key: color_key,
                    name: "colorValue",
                    owner: "SolidColor",
                    value: FieldValue::Color(0xffff_ffff),
                }],
            ))]);

        let initial_cache_epoch = instance.cache_epoch();
        let initial_prepared_epoch = instance.prepared_epoch();
        let initial_path_epoch = instance.path_epoch();
        let initial_layout_revision = instance.layout_revision();
        let initial_paint_revision = instance.solid_color_paint_revision(0);

        assert!(instance.set_keyed_solid_color_property(0, color_key, false, 0xff00_ff00));

        assert_eq!(
            instance.cache_epoch(),
            initial_cache_epoch,
            "SolidColor::colorChanged mutates the existing RenderPaint synchronously without invalidating draw topology"
        );
        assert_eq!(instance.prepared_epoch(), initial_prepared_epoch);
        assert_eq!(instance.path_epoch(), initial_path_epoch);
        assert_eq!(instance.layout_revision(), initial_layout_revision);
        assert!(instance.solid_color_paint_revision(0) > initial_paint_revision);

        let settled_cache_epoch = instance.cache_epoch();
        let settled_paint_revision = instance.solid_color_paint_revision(0);
        assert!(!instance.set_keyed_solid_color_property(0, color_key, false, 0xff00_ff00));
        assert_eq!(instance.cache_epoch(), settled_cache_epoch);
        assert_eq!(
            instance.solid_color_paint_revision(0),
            settled_paint_revision
        );
    }

    #[test]
    fn solid_color_visibility_changes_invalidate_prepared_topology() {
        let mut solid = synthetic_component(0, 0);
        solid.type_name = "SolidColor";
        let mut instance = synthetic_instance(vec![solid], vec![0]);
        let color_key =
            property_key_for_name("SolidColor", "colorValue").expect("SolidColor.colorValue");
        instance.objects =
            InstanceObjectArena::from_runtime_objects(vec![Some(synthetic_runtime_object(
                0,
                "SolidColor",
                vec![RuntimeProperty {
                    key: color_key,
                    name: "colorValue",
                    owner: "SolidColor",
                    value: FieldValue::Color(0xffff_ffff),
                }],
            ))]);

        let initial_prepared_epoch = instance.prepared_epoch();

        assert!(instance.set_color_property(0, color_key, 0x00ff_ffff));

        assert!(instance.prepared_epoch() > initial_prepared_epoch);
    }

    #[test]
    fn prepared_epoch_ignores_nested_input_proxy_value_changes() {
        let mut nested_number = synthetic_component(0, 0);
        nested_number.type_name = "NestedNumber";
        let mut instance = synthetic_instance(vec![nested_number], vec![0]);
        let nested_value =
            property_key_for_name("NestedNumber", "nestedValue").expect("NestedNumber.nestedValue");

        let initial_cache_epoch = instance.cache_epoch();
        let initial_prepared_epoch = instance.prepared_epoch();

        // C++ forwards the virtual setter only to a live SMINumber and leaves
        // the serialized parent field untouched when no child input resolves
        // (`src/animation/nested_number.cpp:24-48`).
        assert!(!instance.set_double_property(0, nested_value, 1.0));
        assert_eq!(instance.double_property(0, nested_value), Some(0.0));
        assert_eq!(instance.cache_epoch(), initial_cache_epoch);
        assert_eq!(instance.prepared_epoch(), initial_prepared_epoch);
    }

    #[test]
    fn prepared_epoch_ignores_nested_artboard_animation_knobs() {
        let mut nested_artboard = synthetic_component(0, 0);
        nested_artboard.type_name = "NestedArtboard";
        let mut instance = synthetic_instance(vec![nested_artboard], vec![0]);
        let speed_key = property_key_for_name("NestedArtboard", "speed").expect("speed");

        let initial_cache_epoch = instance.cache_epoch();
        let initial_prepared_epoch = instance.prepared_epoch();

        assert!(instance.set_double_property(0, speed_key, 2.0));

        assert!(instance.cache_epoch() > initial_cache_epoch);
        assert_eq!(instance.prepared_epoch(), initial_prepared_epoch);
    }

    #[test]
    fn nested_layout_bounds_cache_tracks_layout_revision() {
        let mut host = synthetic_component(0, 0);
        host.type_name = "NestedArtboardLayout";
        let mut instance = synthetic_instance(vec![host], vec![0]);
        instance.nested_artboards.insert(
            0,
            RuntimeNestedArtboardInstance {
                child: Box::new(synthetic_instance(
                    vec![synthetic_component(10, 0)],
                    vec![10],
                )),
                render_cache_revision: 0,
                render_resources: RefCell::new(
                    crate::draw::RuntimeOccurrenceRenderResources::default(),
                ),
                initial_layout_paint_frame: RefCell::new(None),
                layout_data_transferred: false,
                layout_data_transfer_key: None,
                data_bind_path_ids: None,
                data_bind_path_is_relative: false,
                stateful_view_model_instance_local: None,
                stateful_view_model_instance_locals_by_id: BTreeMap::new(),
                stateful_view_model_context: None,
                stateful_global_view_model_contexts: BTreeMap::new(),
                data_bind_property_source_locals: Vec::new(),
                data_bind_image_source_locals: Vec::new(),
                data_bind_context_source_locals_by_path: BTreeMap::new(),
                animations: Vec::new(),
                is_paused: false,
                speed: 1.0,
                quantize: 0.0,
                cumulated_seconds: 0.0,
            },
        );
        instance.nested_artboard_locals.push(0);

        let first_frame = instance.runtime_nested_artboard_layout_bounds_frame();
        let first_bounds = first_frame.bounds.clone();
        assert_eq!(first_frame.key.layout_revision, instance.layout_revision());
        assert!(Arc::ptr_eq(&first_bounds, &first_frame.bounds));

        assert!(instance.add_dirt(0, ComponentDirt::PAINT, false));
        let after_paint = instance.runtime_nested_artboard_layout_bounds_frame();
        assert_eq!(
            instance
                .nested_layout_bounds
                .as_ref()
                .expect("nested layout bounds frame")
                .key
                .layout_revision,
            instance.layout_revision()
        );
        assert!(Arc::ptr_eq(&first_bounds, &after_paint.bounds));

        assert!(instance.add_dirt(0, ComponentDirt::LAYOUT_STYLE, false));
        let after_layout = instance.runtime_nested_artboard_layout_bounds_frame();
        assert_eq!(
            instance
                .nested_layout_bounds
                .as_ref()
                .expect("nested layout bounds frame")
                .key
                .layout_revision,
            instance.layout_revision()
        );
        assert!(!Arc::ptr_eq(&first_bounds, &after_layout.bounds));
    }

    #[test]
    fn layout_revision_tracks_retained_layout_owners_not_global_text_writes() {
        let layout = synthetic_component_for_type(0, "LayoutComponent");
        let mut text_run = synthetic_component(1, 1);
        text_run.type_name = "TextValueRun";
        let mut solid = synthetic_component(2, 2);
        solid.type_name = "SolidColor";
        let mut instance = synthetic_instance(vec![layout, text_run, solid], vec![0, 1, 2]);

        let fractional_width =
            property_key_for_name("LayoutComponent", "fractionalWidth").expect("fractional width");
        let text = property_key_for_name("TextValueRun", "text").expect("text run text");
        let color = property_key_for_name("SolidColor", "colorValue").expect("solid color");

        let layout_revision = instance.layout_revision();
        let layout_node_revision = instance
            .component(0)
            .and_then(|component| component.concrete.layout.as_ref())
            .expect("layout owner")
            .layout_node_revision();
        assert!(instance.set_double_property(0, fractional_width, 0.5));
        assert!(instance.layout_revision() > layout_revision);
        assert!(
            instance
                .component(0)
                .and_then(|component| component.concrete.layout.as_ref())
                .expect("layout owner")
                .layout_node_revision()
                > layout_node_revision
        );

        let layout_revision = instance.layout_revision();

        assert!(instance.set_string_property(1, text, b"hello".to_vec()));
        assert_eq!(instance.layout_revision(), layout_revision);

        assert!(instance.set_color_property(2, color, 0xff00ff00));
        assert_eq!(instance.layout_revision(), layout_revision);
    }

    #[test]
    fn named_root_text_value_run_write_uses_first_local_match_and_ignores_nested_runs() {
        let text_key =
            property_key_for_name("TextValueRun", "text").expect("TextValueRun.text key");
        let mut first = synthetic_component(0, 0);
        first.type_name = "TextValueRun";
        let mut second = synthetic_component(1, 1);
        second.type_name = "TextValueRun";
        let mut instance = synthetic_instance(vec![first, second], vec![0, 1]);
        instance.slots[0].name = Some("headline".to_owned());
        instance.slots[1].name = Some("headline".to_owned());
        // Resolution is explicitly local-id ordered even if an embedding's
        // slot enumeration is not already sorted that way.
        instance.slots.reverse();
        instance.objects = InstanceObjectArena::from_runtime_objects(vec![
            Some(synthetic_runtime_object(
                0,
                "TextValueRun",
                vec![RuntimeProperty {
                    key: text_key,
                    name: "text",
                    owner: "TextValueRun",
                    value: FieldValue::String(StringValue {
                        value: Some("first".to_owned()),
                        raw: b"first".to_vec(),
                    }),
                }],
            )),
            Some(synthetic_runtime_object(
                1,
                "TextValueRun",
                vec![RuntimeProperty {
                    key: text_key,
                    name: "text",
                    owner: "TextValueRun",
                    value: FieldValue::String(StringValue {
                        value: Some("second".to_owned()),
                        raw: b"second".to_vec(),
                    }),
                }],
            )),
        ]);

        let mut nested_run = synthetic_component(0, 0);
        nested_run.type_name = "TextValueRun";
        let mut nested = synthetic_instance(vec![nested_run], vec![0]);
        nested.slots[0].name = Some("headline".to_owned());
        nested.objects =
            InstanceObjectArena::from_runtime_objects(vec![Some(synthetic_runtime_object(
                0,
                "TextValueRun",
                vec![RuntimeProperty {
                    key: text_key,
                    name: "text",
                    owner: "TextValueRun",
                    value: FieldValue::String(StringValue {
                        value: Some("nested".to_owned()),
                        raw: b"nested".to_vec(),
                    }),
                }],
            ))]);
        instance.nested_artboards.insert(
            9,
            RuntimeNestedArtboardInstance {
                child: Box::new(nested),
                ..synthetic_nested_artboard_instance(9)
            },
        );

        assert_eq!(
            instance.set_root_text_value_run("headline", b"updated".to_vec()),
            Some(true)
        );
        assert_eq!(
            instance.string_property(0, text_key),
            Some(b"updated".as_slice())
        );
        assert_eq!(
            instance.string_property(1, text_key),
            Some(b"second".as_slice())
        );
        assert_eq!(
            instance
                .nested_artboards
                .get(&9)
                .and_then(|nested| nested.child.string_property(0, text_key)),
            Some(b"nested".as_slice())
        );
        assert_eq!(
            instance.set_root_text_value_run("headline", b"updated".to_vec()),
            Some(false)
        );
        assert_eq!(
            instance.set_root_text_value_run("missing", b"ignored".to_vec()),
            None
        );
    }

    #[test]
    fn gradient_property_changes_mark_cpp_dirty_bits() {
        let mut gradient = synthetic_component(0, 0);
        gradient.type_name = "LinearGradient";
        let mut stop = synthetic_component(1, 1);
        stop.type_name = "GradientStop";
        let mut instance = synthetic_instance(vec![gradient, stop], vec![0, 1]);
        let parent_key = property_key_for_name("Component", "parentId").expect("parentId key");
        let start_x_key = property_key_for_name("LinearGradient", "startX").expect("startX key");
        let opacity_key = property_key_for_name("LinearGradient", "opacity").expect("opacity key");
        let stop_color_key =
            property_key_for_name("GradientStop", "colorValue").expect("stop color key");
        let stop_position_key =
            property_key_for_name("GradientStop", "position").expect("stop position key");
        let _ = instance.objects.set_uint_property(1, parent_key, 0);

        instance.set_artboard_dirt_for_test(ComponentDirt::NONE);
        instance.component_mut(0).unwrap().dirt = ComponentDirt::NONE;
        assert!(instance.set_double_property(0, start_x_key, 10.0));
        assert!(
            instance
                .component(0)
                .unwrap()
                .dirt
                .contains(ComponentDirt::TRANSFORM)
        );

        instance.set_artboard_dirt_for_test(ComponentDirt::NONE);
        instance.component_mut(0).unwrap().dirt = ComponentDirt::NONE;
        assert!(instance.set_double_property(0, opacity_key, 0.5));
        assert!(
            instance
                .component(0)
                .unwrap()
                .dirt
                .contains(ComponentDirt::PAINT)
        );

        instance.set_artboard_dirt_for_test(ComponentDirt::NONE);
        instance.component_mut(0).unwrap().dirt = ComponentDirt::NONE;
        assert!(instance.set_color_property(1, stop_color_key, 0xff00_ff00));
        assert!(
            instance
                .component(0)
                .unwrap()
                .dirt
                .contains(ComponentDirt::PAINT)
        );
        assert!(
            !instance
                .component(0)
                .unwrap()
                .dirt
                .contains(ComponentDirt::STOPS),
            "GradientStop::colorValueChanged publishes Paint only; positionChanged adds Stops"
        );

        instance.set_artboard_dirt_for_test(ComponentDirt::NONE);
        instance.component_mut(0).unwrap().dirt = ComponentDirt::NONE;
        assert!(instance.set_double_property(1, stop_position_key, 0.25));
        assert!(
            instance
                .component(0)
                .unwrap()
                .dirt
                .contains(ComponentDirt::PAINT | ComponentDirt::STOPS)
        );
    }

    #[test]
    fn follow_path_property_changes_dirty_the_constrained_parent_transform() {
        let parent = synthetic_component_for_type(0, "Node");
        let constraint = synthetic_component_for_type(1, "FollowPathConstraint");
        let mut instance = synthetic_instance(vec![parent, constraint], vec![0, 1]);
        synthetic_link_parent(&mut instance, 1, 0);
        let distance_key = property_key_for_name("FollowPathConstraint", "distance")
            .expect("FollowPathConstraint.distance key");
        let orient_key = property_key_for_name("FollowPathConstraint", "orient")
            .expect("FollowPathConstraint.orient key");
        let strength_key =
            property_key_for_name("Constraint", "strength").expect("Constraint.strength key");

        fn assert_parent_transform_dirty(instance: &mut ArtboardInstance, changed: bool) {
            assert!(changed);
            assert!(
                instance
                    .component(0)
                    .unwrap()
                    .dirt
                    .contains(ComponentDirt::TRANSFORM | ComponentDirt::WORLD_TRANSFORM)
            );
            instance.set_artboard_dirt_for_test(ComponentDirt::NONE);
            instance.component_mut(0).unwrap().dirt = ComponentDirt::NONE;
        }

        let changed = instance.set_double_property(1, distance_key, 0.5);
        assert_parent_transform_dirty(&mut instance, changed);
        let changed = instance.set_bool_property(1, orient_key, false);
        assert_parent_transform_dirty(&mut instance, changed);
        let changed = instance.set_double_property(1, strength_key, 0.5);
        assert_parent_transform_dirty(&mut instance, changed);
    }

    #[test]
    fn list_follow_path_inherited_and_leaf_callbacks_dirty_the_constrained_list() {
        let list = synthetic_component_for_type(0, "ArtboardComponentList");
        let constraint = synthetic_component_for_type(1, "ListFollowPathConstraint");
        let mut instance = synthetic_instance(vec![list, constraint], vec![0, 1]);
        synthetic_link_parent(&mut instance, 1, 0);
        let distance_key = property_key_for_name("FollowPathConstraint", "distance").unwrap();
        let orient_key = property_key_for_name("FollowPathConstraint", "orient").unwrap();
        let distance_end_key =
            property_key_for_name("ListFollowPathConstraint", "distanceEnd").unwrap();
        let distance_offset_key =
            property_key_for_name("ListFollowPathConstraint", "distanceOffset").unwrap();
        let offset_key = property_key_for_name("FollowPathConstraint", "offset").unwrap();

        for change in [
            (distance_key, 0.25),
            (distance_end_key, 0.75),
            (distance_offset_key, 0.1),
        ] {
            instance.component_mut(0).unwrap().dirt = ComponentDirt::NONE;
            assert!(instance.set_double_property(1, change.0, change.1));
            assert!(
                instance
                    .component(0)
                    .unwrap()
                    .dirt
                    .contains(ComponentDirt::TRANSFORM | ComponentDirt::WORLD_TRANSFORM)
            );
        }

        instance.component_mut(0).unwrap().dirt = ComponentDirt::NONE;
        assert!(instance.set_bool_property(1, orient_key, false));
        assert!(
            instance
                .component(0)
                .unwrap()
                .dirt
                .contains(ComponentDirt::TRANSFORM | ComponentDirt::WORLD_TRANSFORM)
        );

        // Generated FollowPath.offset has an intentional no-op changed
        // callback (`follow_path_constraint_base.hpp:107-110`).
        instance.component_mut(0).unwrap().dirt = ComponentDirt::NONE;
        assert!(instance.set_bool_property(1, offset_key, true));
        assert!(instance.component(0).unwrap().dirt.is_empty());
    }

    #[test]
    fn ik_strength_and_invert_dirty_the_retained_chain_but_parent_count_is_noop() {
        let root = synthetic_component_for_type(0, "RootBone");
        let middle = synthetic_component_for_type(1, "Bone");
        let tip = synthetic_component_for_type(2, "Bone");
        let constraint = synthetic_component_for_type(3, "IKConstraint");
        let mut instance =
            synthetic_instance(vec![root, middle, tip, constraint], vec![0, 1, 2, 3]);
        synthetic_link_parent(&mut instance, 1, 0);
        synthetic_link_parent(&mut instance, 2, 1);
        synthetic_link_parent(&mut instance, 3, 2);
        let links = [0usize, 1, 2]
            .into_iter()
            .enumerate()
            .map(|(index, local)| RuntimeIkChainLink {
                index,
                bone: instance.component_handle(local).unwrap(),
                angle: 0.0,
                transform_components: TransformComponents::default(),
                parent_world_inverse: Mat2D::IDENTITY,
            })
            .collect();
        instance
            .component_mut(3)
            .unwrap()
            .concrete
            .ik
            .as_mut()
            .unwrap()
            .chain = links;

        let clear_chain = |instance: &mut ArtboardInstance| {
            for local in 0..3 {
                instance.component_mut(local).unwrap().dirt = ComponentDirt::NONE;
            }
        };
        let assert_chain_dirty = |instance: &ArtboardInstance| {
            for local in 0..3 {
                assert!(
                    instance
                        .component(local)
                        .unwrap()
                        .dirt
                        .contains(ComponentDirt::TRANSFORM | ComponentDirt::WORLD_TRANSFORM)
                );
            }
        };

        let strength_key = property_key_for_name("Constraint", "strength").unwrap();
        clear_chain(&mut instance);
        assert!(instance.set_double_property(3, strength_key, 0.5));
        assert_chain_dirty(&instance);

        clear_chain(&mut instance);
        assert!(instance.set_bool_property(
            3,
            crate::constraints::IK_INVERT_DIRECTION_PROPERTY_KEY,
            true,
        ));
        assert_chain_dirty(&instance);

        clear_chain(&mut instance);
        assert!(instance.set_uint_property(
            3,
            crate::constraints::IK_PARENT_BONE_COUNT_PROPERTY_KEY,
            2,
        ));
        for local in 0..3 {
            assert!(instance.component(local).unwrap().dirt.is_empty());
        }
        assert_eq!(
            instance
                .component(3)
                .unwrap()
                .concrete
                .ik
                .as_ref()
                .unwrap()
                .chain
                .len(),
            3,
            "parentBoneCountChanged is a no-op; lifecycle owns chain rebuild"
        );
    }

    #[test]
    fn a5_owned_state_resets_on_occurrence_clone() {
        let follow = synthetic_component_for_type(0, "FollowPathConstraint");
        let list = synthetic_component_for_type(1, "ArtboardComponentList");
        let ik = synthetic_component_for_type(2, "IKConstraint");
        let mut instance = synthetic_instance(vec![follow, list, ik], vec![0, 1, 2]);
        let follow_handle = instance.component_handle(0).unwrap();
        let ik_handle = instance.component_handle(2).unwrap();
        {
            let follow = instance
                .component_mut(0)
                .unwrap()
                .concrete
                .follow_path
                .as_mut()
                .unwrap();
            follow.raw_path.move_to(1.0, 2.0);
            follow.path_measure = crate::draw::RuntimePathMeasure::from_raw_path(&follow.raw_path);
        }
        instance
            .component_mut(1)
            .unwrap()
            .concrete
            .constrainable_list
            .as_mut()
            .unwrap()
            .constraints
            .push(follow_handle);
        instance
            .component_mut(2)
            .unwrap()
            .concrete
            .ik
            .as_mut()
            .unwrap()
            .chain
            .push(RuntimeIkChainLink {
                index: 0,
                bone: ik_handle,
                angle: 1.0,
                transform_components: TransformComponents::default(),
                parent_world_inverse: Mat2D::IDENTITY,
            });

        let cloned = instance.clone();
        assert!(
            cloned
                .component(0)
                .unwrap()
                .concrete
                .follow_path
                .as_ref()
                .unwrap()
                .raw_path
                .verbs()
                .is_empty()
        );
        assert!(
            cloned
                .component(1)
                .unwrap()
                .concrete
                .constrainable_list
                .as_ref()
                .unwrap()
                .constraints
                .is_empty()
        );
        assert!(
            cloned
                .component(2)
                .unwrap()
                .concrete
                .ik
                .as_ref()
                .unwrap()
                .chain
                .is_empty()
        );
    }

    #[test]
    fn scripted_path_snapshot_exposes_current_retained_authored_geometry() {
        let root = synthetic_component_for_type(0, "Node");
        let shape = synthetic_component_for_type(1, "Shape");
        let path = synthetic_component_for_type(2, "PointsPath");
        let mut instance = synthetic_instance(vec![root, shape, path], vec![2, 0]);

        instance.runtime_shapes.seed_follow_path_source_for_test(
            1,
            2,
            &[
                crate::draw::RuntimePathCommand::Move { x: 1.0, y: 2.0 },
                crate::draw::RuntimePathCommand::Line { x: 3.0, y: 4.0 },
            ],
            false,
        );
        let first = instance
            .runtime_shapes
            .retained_script_path(2)
            .expect("updated Path exposes its retained rawPath to scripts");
        assert_eq!(first.verbs().len(), 2);
        assert_eq!(
            first.points(),
            &[
                nuxie_render_api::Vec2D::new(1.0, 2.0),
                nuxie_render_api::Vec2D::new(3.0, 4.0),
            ]
        );

        instance.runtime_shapes.seed_follow_path_source_for_test(
            1,
            2,
            &[
                crate::draw::RuntimePathCommand::Move { x: 5.0, y: 6.0 },
                crate::draw::RuntimePathCommand::Line { x: 7.0, y: 8.0 },
            ],
            false,
        );
        let current = instance
            .runtime_shapes
            .retained_script_path(2)
            .expect("subsequent lookup exposes the rebuilt path");
        assert_eq!(
            first.points(),
            &[
                nuxie_render_api::Vec2D::new(1.0, 2.0),
                nuxie_render_api::Vec2D::new(3.0, 4.0),
            ]
        );
        assert_eq!(
            current.points(),
            &[
                nuxie_render_api::Vec2D::new(5.0, 6.0),
                nuxie_render_api::Vec2D::new(7.0, 8.0),
            ]
        );
    }

    #[test]
    fn follow_path_measure_rebuilds_on_owner_update_and_preserves_when_shape_is_empty() {
        let root = synthetic_component_for_type(0, "Node");
        let shape = synthetic_component_for_type(1, "Shape");
        let path = synthetic_component_for_type(2, "PointsPath");
        let constraint = synthetic_component_for_type(3, "FollowPathConstraint");
        let empty_shape = synthetic_component_for_type(4, "Shape");
        let mut instance = synthetic_instance(
            vec![root, shape, path, constraint, empty_shape],
            vec![2, 3, 0],
        );
        synthetic_link_parent(&mut instance, 1, 0);
        synthetic_link_parent(&mut instance, 2, 1);
        synthetic_link_parent(&mut instance, 3, 0);
        synthetic_link_parent(&mut instance, 4, 0);
        let constraint_handle = instance.component_handle(3).unwrap();
        let shape_handle = instance.component_handle(1).unwrap();
        let path_handle = instance.component_handle(2).unwrap();
        instance
            .component_mut(1)
            .unwrap()
            .concrete
            .shape
            .as_mut()
            .unwrap()
            .paths
            .push(path_handle);
        {
            let path = instance
                .component_mut(2)
                .unwrap()
                .concrete
                .path
                .as_mut()
                .unwrap();
            path.shape = Some(shape_handle);
        }
        instance
            .component_mut(3)
            .unwrap()
            .concrete
            .constraint
            .as_mut()
            .unwrap()
            .target = Some(shape_handle);
        instance.runtime_shapes.seed_follow_path_source_for_test(
            1,
            2,
            &[
                crate::draw::RuntimePathCommand::Move { x: 0.0, y: 0.0 },
                crate::draw::RuntimePathCommand::Line { x: 10.0, y: 0.0 },
            ],
            false,
        );

        assert!(
            crate::constraints::follow_path_constraint::update_follow_path_constraint(
                &mut instance,
                constraint_handle
            )
        );
        let follow = instance
            .component(3)
            .unwrap()
            .concrete
            .follow_path
            .as_ref()
            .unwrap();
        assert_eq!(follow.measure_rebuilds, 1);
        assert_eq!(follow.path_measure.length(), 10.0);
        let retained_verb_storage = follow.raw_path.verbs().as_ptr();
        let retained_point_storage = follow.raw_path.points().as_ptr();

        instance.runtime_shapes.seed_follow_path_source_for_test(
            1,
            2,
            &[
                crate::draw::RuntimePathCommand::Move { x: 2.0, y: 0.0 },
                crate::draw::RuntimePathCommand::Line { x: 14.0, y: 0.0 },
            ],
            false,
        );
        assert!(
            crate::constraints::follow_path_constraint::update_follow_path_constraint(
                &mut instance,
                constraint_handle
            )
        );
        let follow = instance
            .component(3)
            .unwrap()
            .concrete
            .follow_path
            .as_ref()
            .unwrap();
        assert_eq!(follow.measure_rebuilds, 2);
        assert_eq!(follow.path_measure.length(), 12.0);
        assert_eq!(
            follow.raw_path.verbs().as_ptr(),
            retained_verb_storage,
            "FollowPath reuses m_rawPath verb storage across equal-size rewinds (`follow_path_constraint.cpp:137-145`)"
        );
        assert_eq!(
            follow.raw_path.points().as_ptr(),
            retained_point_storage,
            "FollowPath reuses m_rawPath point storage across equal-size rewinds (`follow_path_constraint.cpp:137-145`)"
        );

        instance
            .component_mut(3)
            .unwrap()
            .concrete
            .constraint
            .as_mut()
            .unwrap()
            .target = Some(instance.component_handle(4).unwrap());
        assert!(
            !crate::constraints::follow_path_constraint::update_follow_path_constraint(
                &mut instance,
                constraint_handle
            )
        );
        let follow = instance
            .component(3)
            .unwrap()
            .concrete
            .follow_path
            .as_ref()
            .unwrap();
        assert_eq!(follow.measure_rebuilds, 2);
        assert_eq!(follow.path_measure.length(), 12.0);

        // A clean Artboard pass never reaches the FollowPath update site.
        instance.set_artboard_dirt_for_test(ComponentDirt::NONE);
        for local in 0..5 {
            instance.component_mut(local).unwrap().dirt = ComponentDirt::NONE;
        }
        assert!(!instance.update_components().did_update);
        assert_eq!(
            instance
                .component(3)
                .unwrap()
                .concrete
                .follow_path
                .as_ref()
                .unwrap()
                .measure_rebuilds,
            2
        );
    }

    #[test]
    fn path_deferral_consumes_occurrence_owned_follow_flags_and_rearms_on_dirty() {
        let bytes = synthetic_riv(11_905, |bytes| {
            push_synthetic_object(bytes, "Backboard", &[]);
            push_synthetic_object(bytes, "Artboard", &[]);
            push_synthetic_object(bytes, "Shape", &[("parentId", 0)]);
            push_synthetic_object_with_properties(bytes, "Rectangle", |bytes| {
                push_synthetic_uint_property(bytes, "Rectangle", "parentId", 1);
                push_synthetic_f32_property(bytes, "Rectangle", "width", 20.0);
                push_synthetic_f32_property(bytes, "Rectangle", "height", 10.0);
            });
        });
        let mut instance = instance_from_riv(&bytes);
        assert!(instance.update_components().did_update);
        let path = instance.component_handle(2).expect("Rectangle Path handle");
        let shape = instance.component_handle(1).expect("Shape handle");
        let initial_mutation = instance
            .runtime_shapes
            .retained_path_mutation_id_for_test(2)
            .expect("visible first update builds the Path");

        instance.component_at_mut(shape).transform.render_opacity = 0.0;
        assert!(instance.add_component_dirt(path, ComponentDirt::PATH, false));
        assert!(instance.update_components().did_update);
        assert_eq!(
            instance
                .runtime_shapes
                .retained_path_mutation_id_for_test(2),
            Some(initial_mutation),
            "an invisible ordinary Path retains its prior RawPath while deferred (`path.cpp:111-125,350-372`)"
        );
        assert!(
            instance
                .component_at(path)
                .concrete
                .path
                .as_ref()
                .unwrap()
                .deferred_path_dirt
                .get()
        );

        instance.component_at_mut(shape).transform.render_opacity = 1.0;
        assert!(instance.add_component_dirt(path, ComponentDirt::WORLD_TRANSFORM, false));
        assert!(
            instance
                .component_at(path)
                .dirt
                .contains(ComponentDirt::PATH),
            "the next Path::onDirty re-adds deferred Path dirt (`path.cpp:336-347`)"
        );
        assert!(instance.update_components().did_update);
        let restored_mutation = instance
            .runtime_shapes
            .retained_path_mutation_id_for_test(2)
            .expect("restored visible Path rebuilds");
        assert_ne!(restored_mutation, initial_mutation);
        assert!(
            !instance
                .component_at(path)
                .concrete
                .path
                .as_ref()
                .unwrap()
                .deferred_path_dirt
                .get()
        );

        instance.component_at_mut(shape).transform.render_opacity = 0.0;
        instance
            .component_at(shape)
            .concrete
            .shape
            .as_ref()
            .unwrap()
            .add_flags(crate::components::RuntimeShapeState::FOLLOW_PATH);
        assert!(instance.add_component_dirt(path, ComponentDirt::PATH, false));
        assert!(instance.update_components().did_update);
        assert_ne!(
            instance
                .runtime_shapes
                .retained_path_mutation_id_for_test(2),
            Some(restored_mutation),
            "Shape followPath forbids deferral even at zero opacity (`follow_path_constraint.cpp:149-165`; `path.cpp:111-125`)"
        );

        let cloned = instance.clone();
        assert!(
            !cloned
                .component_at(shape)
                .concrete
                .shape
                .as_ref()
                .unwrap()
                .is_flagged(crate::components::RuntimeShapeState::FOLLOW_PATH),
            "generated clone resets runtime flags before clean-phase producers rebuild them"
        );
        assert!(
            !cloned
                .component_at(path)
                .concrete
                .path
                .as_ref()
                .unwrap()
                .deferred_path_dirt
                .get()
        );
    }

    #[test]
    fn artboard_clip_property_updates_draw_cache() {
        let mut artboard = synthetic_component(0, 0);
        artboard.type_name = "Artboard";
        let mut instance = synthetic_instance(vec![artboard], vec![0]);
        let clip_key = property_key_for_name("Artboard", "clip").expect("Artboard.clip key");

        assert!(instance.clip);
        assert!(instance.set_bool_property(0, clip_key, false));
        assert!(!instance.clip);
        assert!(instance.set_bool_property(0, clip_key, true));
        assert!(instance.clip);
    }

    #[test]
    fn update_components_skips_collapsed_components_without_clearing_dirt() {
        let mut first = synthetic_component(0, 0);
        first.dirt = ComponentDirt::PATH;
        let mut second = synthetic_component(1, 1);
        second.dirt = ComponentDirt::PATH | ComponentDirt::COLLAPSED;
        let mut instance = synthetic_instance(vec![first, second], vec![0, 1]);

        let report = instance.update_components();

        assert!(report.did_update);
        assert_eq!(report.updated_locals, vec![0]);
        assert_eq!(instance.component(0).unwrap().dirt, ComponentDirt::NONE);
        assert!(
            instance
                .component(1)
                .unwrap()
                .dirt
                .contains(ComponentDirt::PATH)
        );
        assert!(instance.component(1).unwrap().is_collapsed());
    }

    #[test]
    fn update_components_restarts_when_update_dirties_earlier_graph_order() {
        let first = synthetic_component(0, 0);
        let mut second = synthetic_component(1, 1);
        second.dirt = ComponentDirt::PATH;
        let mut instance = synthetic_instance(vec![first, second], vec![0, 1]);
        let mut dirtied_earlier = false;

        let report = instance.update_components_with_hook(|instance, local_id, _| {
            if local_id == 1 && !dirtied_earlier {
                dirtied_earlier = true;
                instance.add_dirt(0, ComponentDirt::PATH, false);
            }
        });

        assert_eq!(report.steps, 2);
        assert_eq!(report.updated_locals, vec![1, 0]);
        assert!(!report.max_steps_reached);
    }

    #[test]
    fn update_components_surfaces_cpp_max_pass_guard() {
        let mut component = synthetic_component(0, 0);
        component.dirt = ComponentDirt::PATH;
        let mut instance = synthetic_instance(vec![component], vec![0]);

        let report = instance.update_components_with_hook(|instance, local_id, _| {
            instance.add_dirt(local_id, ComponentDirt::PATH, false);
        });

        assert_eq!(report.steps, 100);
        assert_eq!(report.updated_locals.len(), 100);
        assert!(report.max_steps_reached);
        assert!(instance.has_dirt(ComponentDirt::COMPONENTS));
    }

    fn synthetic_runtime_object(
        id: u32,
        type_name: &'static str,
        properties: Vec<RuntimeProperty>,
    ) -> RuntimeObject {
        let definition = definition_by_name(type_name).expect("synthetic runtime object type");
        RuntimeObject {
            id,
            type_key: definition.type_key.int,
            type_name: definition.name,
            rust_variant: definition.rust_variant,
            properties,
            skipped_properties: Vec::new(),
        }
    }

    #[test]
    fn instance_object_arena_uses_generated_core_registry_setter_families() {
        let node_x_key = property_key_for_name("Node", "x").expect("Node.x key");
        let bytes_key =
            property_key_for_name("FileAssetContents", "bytes").expect("FileAssetContents.bytes");
        let mut arena = InstanceObjectArena::from_runtime_objects(vec![
            Some(synthetic_runtime_object(0, "Node", Vec::new())),
            Some(synthetic_runtime_object(1, "FileAssetContents", Vec::new())),
        ]);

        assert!(arena.set_double_property(0, node_x_key, 12.5));
        assert_eq!(arena.double_property(0, node_x_key), Some(12.5));

        assert!(!arena.set_uint_property(0, node_x_key, 12));
        assert_eq!(arena.double_property(0, node_x_key), Some(12.5));

        assert!(!arena.set_string_property(1, bytes_key, vec![1, 2, 3]));
        assert_eq!(arena.string_property(1, bytes_key), None);
    }

    #[test]
    fn instance_object_arena_keeps_mutable_properties_in_instance_storage() {
        let node_x_key = property_key_for_name("Node", "x").expect("Node.x key");
        let source = synthetic_runtime_object(0, "Node", Vec::new());
        let mut arena = InstanceObjectArena::from_runtime_objects(vec![Some(source.clone())]);

        assert!(arena.set_double_property(0, node_x_key, 42.0));

        assert!(source.properties.is_empty());
        assert_eq!(arena.double_property(0, node_x_key), Some(42.0));
    }

    #[test]
    fn instance_object_arena_reads_generated_defaults_and_imported_fields() {
        let node_x_key = property_key_for_name("Node", "x").expect("Node.x key");
        let artboard_clip_key = property_key_for_name("Artboard", "clip").expect("Artboard.clip");
        let bytes_key =
            property_key_for_name("FileAssetContents", "bytes").expect("FileAssetContents.bytes");
        let arena = InstanceObjectArena::from_runtime_objects(vec![
            Some(synthetic_runtime_object(
                0,
                "Node",
                vec![RuntimeProperty {
                    key: node_x_key,
                    name: "x",
                    owner: "Node",
                    value: FieldValue::Double(7.5),
                }],
            )),
            Some(synthetic_runtime_object(1, "Artboard", Vec::new())),
            Some(synthetic_runtime_object(
                2,
                "FileAssetContents",
                vec![RuntimeProperty {
                    key: bytes_key,
                    name: "bytes",
                    owner: "FileAssetContents",
                    value: FieldValue::Bytes(BytesValue::new(vec![1, 2, 3])),
                }],
            )),
        ]);

        assert_eq!(arena.double_property(0, node_x_key), Some(7.5));
        assert_eq!(arena.bool_property(1, artboard_clip_key), Some(true));
        assert_eq!(arena.string_property(2, bytes_key), Some(&[1, 2, 3][..]));
    }

    #[test]
    fn artboard_typed_property_reads_surface_defaults_and_reject_wrong_value_kinds() {
        let opacity_key = property_key_for_name("Shape", "opacity").expect("Shape.opacity");
        let color_key =
            property_key_for_name("SolidColor", "colorValue").expect("SolidColor.colorValue");
        let mut instance = synthetic_instance(Vec::new(), Vec::new());
        instance.objects = InstanceObjectArena::from_runtime_objects(vec![
            Some(synthetic_runtime_object(0, "Shape", Vec::new())),
            Some(synthetic_runtime_object(1, "SolidColor", Vec::new())),
        ]);

        assert_eq!(instance.double_property(0, opacity_key), Some(1.0));
        assert_eq!(instance.color_property(1, color_key), Some(0xff74_7474));
        assert_eq!(instance.color_property(0, opacity_key), None);
        assert_eq!(instance.double_property(1, color_key), None);
    }

    #[test]
    fn update_transform_reads_generated_instance_storage() {
        let node_x_key = property_key_for_name("Node", "x").expect("Node.x key");
        let node_scale_x_key = property_key_for_name("Node", "scaleX").expect("Node.scaleX key");
        let mut component = synthetic_component(0, 0);
        component.dirt = ComponentDirt::TRANSFORM;
        let mut instance = synthetic_instance(vec![component], vec![0]);

        assert!(instance.objects.set_double_property_by_name(0, "x", 8.0));
        assert!(
            instance
                .objects
                .set_double_property_by_name(0, "scaleX", 2.5)
        );

        let report = instance.update_components();

        assert_eq!(report.updated_locals, vec![0]);
        assert_eq!(instance.double_property(0, node_x_key), Some(8.0));
        assert_eq!(instance.double_property(0, node_scale_x_key), Some(2.5));
        assert_eq!(
            instance.component(0).unwrap().transform.local_transform,
            Mat2D([2.5, 0.0, -0.0, 1.0, 8.0, 0.0])
        );
    }

    #[test]
    fn transform_update_matches_basic_cpp_order() {
        let mut root = synthetic_component(0, 0);
        root.type_name = "Artboard";
        root.transform.render_opacity = 0.5;
        let mut child = synthetic_component(1, 1);
        child.dirt = ComponentDirt::TRANSFORM
            | ComponentDirt::WORLD_TRANSFORM
            | ComponentDirt::RENDER_OPACITY;
        let mut instance = synthetic_instance(vec![root, child], vec![0, 1]);
        synthetic_link_parent(&mut instance, 1, 0);
        assert!(instance.objects.set_double_property_by_name(1, "x", 2.0));
        assert!(instance.objects.set_double_property_by_name(1, "y", 3.0));
        assert!(
            instance
                .objects
                .set_double_property_by_name(1, "scaleX", 4.0)
        );
        assert!(
            instance
                .objects
                .set_double_property_by_name(1, "scaleY", 5.0)
        );
        assert!(
            instance
                .objects
                .set_double_property_by_name(1, "opacity", 0.25)
        );

        let report = instance.update_components();

        assert_eq!(report.updated_locals, vec![1]);
        let child = instance.component(1).unwrap();
        assert_eq!(
            child.transform.local_transform,
            Mat2D([4.0, 0.0, -0.0, 5.0, 2.0, 3.0])
        );
        assert_eq!(
            child.transform.world_transform,
            child.transform.local_transform
        );
        assert_eq!(child.transform.render_opacity, 0.125);
    }

    #[test]
    fn duplicate_transform_dirt_does_not_redirty_world_dependents() {
        // `TransformComponent::markTransformDirty` returns immediately when
        // Transform is already present; recursive WorldTransform dirt is
        // gated by that first addition (`src/transform_component.cpp:54-61`).
        let mut source = synthetic_component(0, 0);
        source.dirt = ComponentDirt::TRANSFORM;
        let dependent = synthetic_component(1, 1);
        let mut instance = synthetic_instance(vec![source, dependent], vec![0, 1]);
        synthetic_add_dependent(&mut instance, 0, 1);
        instance.clear_component_dirt(1);

        let source = instance.component_handle(0).expect("source");
        assert!(!instance.mark_transform_dirty_handle(source));
        assert!(
            !instance
                .component(1)
                .expect("dependent")
                .dirt
                .contains(ComponentDirt::WORLD_TRANSFORM)
        );
    }

    #[test]
    fn collapse_dirties_constrained_dependents_only_for_transform_owners() {
        // Only TransformComponent::collapse has the constrained-dependent
        // tail, after ContainerComponent child propagation
        // (`src/transform_component.cpp:18-44`). Component/Skin collapse must
        // not acquire that virtual override accidentally.
        for (source_type, should_dirty) in [("Node", true), ("Skin", false)] {
            let source = synthetic_component_for_type(0, source_type);
            let dependent = synthetic_component_for_type(1, "Node");
            let constraint = synthetic_component_for_type(2, "RotationConstraint");
            let mut instance =
                synthetic_instance(vec![source, dependent, constraint], vec![0, 1, 2]);
            let source = instance.component_handle(0).unwrap();
            let dependent = instance.component_handle(1).unwrap();
            let constraint = instance.component_handle(2).unwrap();
            synthetic_add_dependent(&mut instance, 0, 1);
            assert!(instance.objects.add_constraint(dependent, constraint));
            instance.clear_component_dirt(0);
            instance.clear_component_dirt(1);
            instance.clear_component_dirt(2);

            assert!(instance.collapse_component_tree(0, true));
            assert_eq!(
                instance
                    .component_at(dependent)
                    .dirt
                    .contains(ComponentDirt::WORLD_TRANSFORM),
                should_dirty,
                "{source_type} collapse dispatch"
            );
            assert!(instance.component_at(source).is_collapsed());
        }
    }

    #[test]
    fn bone_length_changed_dirties_only_retained_child_bones() {
        // Bone::lengthChanged walks m_ChildBones and calls
        // markTransformDirty on each (`src/bones/bone.cpp:20-26`).
        let parent = synthetic_component_for_type(0, "RootBone");
        let child = synthetic_component_for_type(1, "Bone");
        let unrelated = synthetic_component_for_type(2, "Bone");
        let mut instance = synthetic_instance(vec![parent, child, unrelated], vec![0, 1, 2]);
        let parent = instance.component_handle(0).unwrap();
        let child = instance.component_handle(1).unwrap();
        instance
            .component_at_mut(parent)
            .concrete
            .bone
            .as_mut()
            .unwrap()
            .child_bones
            .push(child);
        instance.clear_component_dirt(0);
        instance.clear_component_dirt(1);
        instance.clear_component_dirt(2);
        let length = property_key_for_name("Bone", "length").expect("Bone.length");

        assert!(instance.set_double_property(0, length, 42.0));
        assert!(
            instance
                .component(1)
                .unwrap()
                .dirt
                .contains(ComponentDirt::TRANSFORM | ComponentDirt::WORLD_TRANSFORM)
        );
        assert_eq!(instance.component(2).unwrap().dirt, ComponentDirt::NONE);
    }

    #[test]
    fn world_transform_parent_uses_base_child_opacity() {
        // WorldTransformComponent::childOpacity returns authored opacity,
        // while TransformComponent overrides it with settled render opacity
        // (`src/world_transform_component.cpp:8`;
        // `include/rive/transform_component.hpp:40-43`).
        // The pinned schema has no concrete WorldTransformComponent-only
        // subtype, so exercise the abstract base dispatch directly.
        let mut parent = synthetic_component(0, 0);
        parent.type_name = "WorldTransformComponent";
        parent.capabilities = RuntimeComponentCapabilities {
            world_transform: true,
            transform: false,
        };
        parent.transform.render_opacity = 0.125;
        assert_eq!(parent.child_opacity(0.5), 0.5);
    }

    #[test]
    fn node_computed_local_is_lazy_and_singular_parent_falls_back_to_identity() {
        // Node invalidates its distinct computed-local cache before the base
        // world update. Query derives inverse(parent world) * settled world;
        // missing/singular parents yield identity (`src/node.cpp:26-45`).
        let parent = synthetic_component(0, 0);
        let child = synthetic_component(1, 1);
        let mut instance = synthetic_instance(vec![parent, child], vec![0, 1]);
        synthetic_link_parent(&mut instance, 1, 0);
        instance.component_mut(0).unwrap().transform.world_transform =
            Mat2D([1.0, 0.0, 0.0, 1.0, 10.0, 3.0]);
        {
            let child = instance.component_mut(1).unwrap();
            child.transform.world_transform = Mat2D([1.0, 0.0, 0.0, 1.0, 25.0, 8.0]);
            child
                .concrete
                .node
                .as_ref()
                .expect("Node state")
                .mark_computed_local_dirty();
        }
        assert_eq!(
            instance.runtime_node_computed_local_transform(1),
            Some(Mat2D([1.0, 0.0, 0.0, 1.0, 15.0, 5.0]))
        );

        instance.component_mut(0).unwrap().transform.world_transform =
            Mat2D([0.0, 0.0, 0.0, 0.0, 10.0, 3.0]);
        instance
            .component_mut(1)
            .unwrap()
            .concrete
            .node
            .as_ref()
            .expect("Node state")
            .mark_computed_local_dirty();
        assert_eq!(
            instance.runtime_node_computed_local_transform(1),
            Some(Mat2D::IDENTITY)
        );
    }

    #[test]
    fn skin_on_dirty_targets_exact_skinnable_dirt_family() {
        // Skin::onDirty calls one retained Skinnable. PointsPath translates
        // that to Path; Mesh translates it to Vertices
        // (`src/bones/skin.cpp:88-94`,
        // `src/shapes/points_path.cpp:43-52`,
        // `src/shapes/mesh.cpp:84-85`).
        for (type_name, expected, rejected) in [
            ("PointsPath", ComponentDirt::PATH, ComponentDirt::VERTICES),
            ("Mesh", ComponentDirt::VERTICES, ComponentDirt::PATH),
        ] {
            let skinnable = synthetic_component_for_type(0, type_name);
            let skin = synthetic_component_for_type(1, "Skin");
            let mut instance = synthetic_instance(vec![skinnable, skin], vec![1, 0]);
            synthetic_link_parent(&mut instance, 1, 0);
            let skinnable = instance.component_handle(0).unwrap();
            let skin = instance.component_handle(1).unwrap();
            instance
                .component_at_mut(skinnable)
                .concrete
                .skinnable
                .as_mut()
                .unwrap()
                .skin = Some(skin);
            {
                let state = instance
                    .component_at_mut(skin)
                    .concrete
                    .skin
                    .as_mut()
                    .unwrap();
                state.skinnable = Some(skinnable);
            }
            instance.clear_component_dirt(0);
            instance.clear_component_dirt(1);

            assert!(instance.add_component_dirt(skin, ComponentDirt::PAINT, false));
            let dirt = instance.component_at(skinnable).dirt;
            assert!(dirt.contains(expected));
            assert!(!dirt.contains(rejected));
        }
    }

    #[test]
    fn skin_buffer_rebuilds_only_when_skin_owner_updates() {
        // Skin owns `(tendons + 1)` matrices and rebuilds them from the
        // retained Tendon/Bone links in update, not during draw
        // (`src/bones/skin.cpp:38-77`).
        let mut bone = synthetic_component_for_type(0, "RootBone");
        bone.transform.world_transform = Mat2D([1.0, 0.0, 0.0, 1.0, 7.0, 9.0]);
        let tendon = synthetic_component_for_type(1, "Tendon");
        let mut skin = synthetic_component_for_type(2, "Skin");
        skin.dirt = ComponentDirt::SKIN;
        let mut instance = synthetic_instance(vec![bone, tendon, skin], vec![0, 1, 2]);
        let bone = instance.component_handle(0).unwrap();
        let tendon = instance.component_handle(1).unwrap();
        let skin = instance.component_handle(2).unwrap();
        {
            let state = instance
                .component_at_mut(tendon)
                .concrete
                .tendon
                .as_mut()
                .unwrap();
            state.bone = Some(bone);
            state.inverse_bind = Mat2D([1.0, 0.0, 0.0, 1.0, 2.0, 3.0]);
        }
        {
            let state = instance
                .component_at_mut(skin)
                .concrete
                .skin
                .as_mut()
                .unwrap();
            state.tendons.push(tendon);
            state.bone_transforms = vec![Mat2D::IDENTITY; 2];
        }

        instance.update_components();
        let state = instance.component_at(skin).concrete.skin.as_ref().unwrap();
        assert_eq!(
            state.bone_transforms,
            vec![Mat2D::IDENTITY, Mat2D([1.0, 0.0, 0.0, 1.0, 9.0, 12.0])]
        );
        assert_eq!(state.buffer_rebuilds, 1);

        instance.update_components();
        assert_eq!(
            instance
                .component_at(skin)
                .concrete
                .skin
                .as_ref()
                .unwrap()
                .buffer_rebuilds,
            1
        );
    }

    #[test]
    fn weight_deformation_reads_live_packed_occurrence_fields() {
        // Weight decodes four packed bytes from the live generated fields and
        // writes its retained translation without normalization beyond
        // weight/255 (see the direct Weight owner in `bones/weight.rs`;
        // `src/shapes/vertex.cpp:17-23`).
        let path = synthetic_component_for_type(0, "PointsPath");
        let skin = synthetic_component_for_type(1, "Skin");
        let vertex = synthetic_component_for_type(2, "StraightVertex");
        let weight = synthetic_component_for_type(3, "Weight");
        let mut instance = synthetic_instance(vec![path, skin, vertex, weight], vec![1, 0, 2, 3]);
        synthetic_link_parent(&mut instance, 1, 0);
        synthetic_link_parent(&mut instance, 2, 0);
        synthetic_link_parent(&mut instance, 3, 2);
        let path = instance.component_handle(0).unwrap();
        let skin = instance.component_handle(1).unwrap();
        let vertex = instance.component_handle(2).unwrap();
        let weight = instance.component_handle(3).unwrap();
        instance
            .component_at_mut(path)
            .concrete
            .skinnable
            .as_mut()
            .unwrap()
            .skin = Some(skin);
        instance
            .component_at_mut(skin)
            .concrete
            .skin
            .as_mut()
            .unwrap()
            .bone_transforms = vec![Mat2D::IDENTITY, Mat2D([1.0, 0.0, 0.0, 1.0, 10.0, 20.0])];
        instance
            .component_at_mut(vertex)
            .concrete
            .vertex
            .as_mut()
            .unwrap()
            .weight = Some(weight);
        let indices = property_key_for_name("Weight", "indices").unwrap();
        let values = property_key_for_name("Weight", "values").unwrap();
        assert_eq!(instance.objects.uint_property(3, indices), Some(1));
        assert!(instance.objects.set_uint_property(3, values, 0));
        assert!(instance.deform_runtime_vertex_weight(2, (2.0, 3.0), None));
        assert_eq!(
            instance.runtime_vertex_weight_state(2).unwrap().translation,
            (0.0, 0.0)
        );

        assert!(instance.objects.set_uint_property(3, values, 255));
        assert!(instance.deform_runtime_vertex_weight(2, (2.0, 3.0), None));
        assert_eq!(
            instance.runtime_vertex_weight_state(2).unwrap().translation,
            (12.0, 23.0)
        );
    }

    #[test]
    fn weight_deformation_matches_clang_contracted_accumulation() {
        // The two active packed influences are the minimal tape.riv
        // counterexample for the one-ulp difference between separate
        // multiply/add and the C++ geometry pipeline's default contraction.
        // Preserve Weight::deform's source loop and clang rounding exactly
        // (see `bones/weight.rs`; `docs/PORTING.md` §4.2).
        let f = f32::from_bits;
        let bones = [
            Mat2D::IDENTITY,
            Mat2D([
                f(1_057_841_340),
                f(3_194_226_667),
                f(1_049_771_165),
                f(1_057_885_778),
                f(1_107_583_000),
                f(1_124_670_380),
            ]),
            Mat2D([
                f(1_057_841_342),
                f(3_194_226_637),
                f(1_049_771_148),
                f(1_057_885_779),
                f(1_106_043_744),
                f(1_125_092_309),
            ]),
        ];
        let point = (f(3_265_177_136), f(1_091_681_984));
        let world = Mat2D([1.0, 0.0, 0.0, 1.0, f(1_135_168_757), f(1_135_822_912)]);

        let deformed =
            deform_point_from_skin(point, 258, 32_385, world, &bones).expect("valid bone indices");
        assert_eq!(
            (deformed.0.to_bits(), deformed.1.to_bits()),
            (1_133_233_234, 1_133_466_484)
        );
    }

    #[test]
    fn cubic_weight_retains_independent_live_in_and_out_translations() {
        // CubicWeight owns independent in/out translations and decodes each
        // packed generated field through Weight::deform
        // (`include/rive/bones/cubic_weight.hpp:9-15`;
        // `src/shapes/cubic_vertex.cpp:16-24`).
        let path = synthetic_component_for_type(0, "PointsPath");
        let skin = synthetic_component_for_type(1, "Skin");
        let vertex = synthetic_component_for_type(2, "CubicDetachedVertex");
        let weight = synthetic_component_for_type(3, "CubicWeight");
        let mut instance = synthetic_instance(vec![path, skin, vertex, weight], vec![1, 0, 2, 3]);
        synthetic_link_parent(&mut instance, 1, 0);
        synthetic_link_parent(&mut instance, 2, 0);
        synthetic_link_parent(&mut instance, 3, 2);
        let path = instance.component_handle(0).unwrap();
        let skin = instance.component_handle(1).unwrap();
        let vertex = instance.component_handle(2).unwrap();
        let weight = instance.component_handle(3).unwrap();
        instance
            .component_at_mut(path)
            .concrete
            .skinnable
            .as_mut()
            .unwrap()
            .skin = Some(skin);
        instance
            .component_at_mut(skin)
            .concrete
            .skin
            .as_mut()
            .unwrap()
            .bone_transforms = vec![Mat2D::IDENTITY, Mat2D([1.0, 0.0, 0.0, 1.0, 10.0, 20.0])];
        instance
            .component_at_mut(vertex)
            .concrete
            .vertex
            .as_mut()
            .unwrap()
            .weight = Some(weight);
        for (owner, field) in [
            ("Weight", "values"),
            ("CubicWeight", "inValues"),
            ("CubicWeight", "outValues"),
        ] {
            let key = property_key_for_name(owner, field).unwrap();
            if instance.objects.uint_property(3, key) != Some(255) {
                assert!(instance.objects.set_uint_property(3, key, 255));
            }
        }
        for (owner, field) in [
            ("Weight", "indices"),
            ("CubicWeight", "inIndices"),
            ("CubicWeight", "outIndices"),
        ] {
            let key = property_key_for_name(owner, field).unwrap();
            if instance.objects.uint_property(3, key) != Some(1) {
                assert!(instance.objects.set_uint_property(3, key, 1));
            }
        }

        assert!(instance.deform_runtime_vertex_weight(
            2,
            (2.0, 3.0),
            Some(((0.0, 1.0), (4.0, 5.0))),
        ));
        let state = instance.runtime_vertex_weight_state(2).unwrap();
        assert_eq!(state.translation, (12.0, 23.0));
        assert_eq!(state.in_translation, (10.0, 21.0));
        assert_eq!(state.out_translation, (14.0, 25.0));
    }

    #[test]
    fn clone_resets_every_bone_runtime_relation_and_buffer() {
        // Generated fields clone, but Bone/Skin/Tendon/Skinnable/Weight
        // runtime pointers, buffers, and lazy outputs are defaulted and later
        // rebuilt against clone-owned objects (`artboard.hpp:548-601`;
        // the direct bone owner modules).
        let bone = synthetic_component_for_type(0, "RootBone");
        let tendon = synthetic_component_for_type(1, "Tendon");
        let skin = synthetic_component_for_type(2, "Skin");
        let vertex = synthetic_component_for_type(3, "CubicDetachedVertex");
        let weight = synthetic_component_for_type(4, "CubicWeight");
        let mut instance = synthetic_instance(
            vec![bone, tendon, skin, vertex, weight],
            vec![0, 1, 2, 3, 4],
        );
        let bone = instance.component_handle(0).unwrap();
        let tendon = instance.component_handle(1).unwrap();
        let skin = instance.component_handle(2).unwrap();
        let vertex = instance.component_handle(3).unwrap();
        let weight = instance.component_handle(4).unwrap();
        instance
            .component_at_mut(bone)
            .concrete
            .bone
            .as_mut()
            .unwrap()
            .child_bones
            .push(bone);
        instance
            .component_at_mut(tendon)
            .concrete
            .tendon
            .as_mut()
            .unwrap()
            .bone = Some(bone);
        {
            let state = instance
                .component_at_mut(skin)
                .concrete
                .skin
                .as_mut()
                .unwrap();
            state.tendons.push(tendon);
            state.skinnable = Some(vertex);
            state.bone_transforms = vec![Mat2D::IDENTITY; 2];
        }
        instance
            .component_at_mut(vertex)
            .concrete
            .vertex
            .as_mut()
            .unwrap()
            .weight = Some(weight);
        instance
            .component_at_mut(weight)
            .concrete
            .weight
            .as_mut()
            .unwrap()
            .translation = (7.0, 9.0);

        let cloned = instance.clone();
        assert!(
            cloned
                .component(0)
                .unwrap()
                .concrete
                .bone
                .as_ref()
                .unwrap()
                .child_bones
                .is_empty()
        );
        assert!(
            cloned
                .component(1)
                .unwrap()
                .concrete
                .tendon
                .as_ref()
                .unwrap()
                .bone
                .is_none()
        );
        let cloned_skin = cloned.component(2).unwrap().concrete.skin.as_ref().unwrap();
        assert!(cloned_skin.tendons.is_empty());
        assert!(cloned_skin.skinnable.is_none());
        assert!(cloned_skin.bone_transforms.is_empty());
        assert!(
            cloned
                .component(3)
                .unwrap()
                .concrete
                .vertex
                .as_ref()
                .unwrap()
                .weight
                .is_none()
        );
        assert_eq!(
            cloned
                .component(4)
                .unwrap()
                .concrete
                .weight
                .as_ref()
                .unwrap()
                .translation,
            (0.0, 0.0)
        );
    }

    #[test]
    fn parent_traversal_crosses_mount_once_and_resets_metadata() {
        // ParentTraversal returns current, then advances; the source Artboard
        // root is the call that records a host crossing
        // (`src/parent_traversal.cpp:14-60`).
        use crate::parent_traversal::{ParentTraversal, ParentTraversalFrame};

        let parent_root = synthetic_component(0, 0);
        let parent_ancestor = synthetic_component(1, 1);
        let parent_host = synthetic_component(2, 2);
        let mut parent = synthetic_instance(
            vec![parent_root, parent_ancestor, parent_host],
            vec![0, 1, 2],
        );
        synthetic_link_parent(&mut parent, 1, 0);
        synthetic_link_parent(&mut parent, 2, 1);

        let child_root = synthetic_component(0, 0);
        let child_parent = synthetic_component(1, 1);
        let child_start = synthetic_component(2, 2);
        let mut child =
            synthetic_instance(vec![child_root, child_parent, child_start], vec![0, 1, 2]);
        synthetic_link_parent(&mut child, 1, 0);
        synthetic_link_parent(&mut child, 2, 1);

        let host = parent.component_handle(2).unwrap();
        let start = child.component_handle(2).unwrap();
        let frames = [
            ParentTraversalFrame {
                artboard: &parent,
                host_component_in_parent: None,
            },
            ParentTraversalFrame {
                artboard: &child,
                host_component_in_parent: Some(host),
            },
        ];
        let mut traversal = ParentTraversal::new(&frames, start);
        assert_eq!(
            child.component_local_id(traversal.next().unwrap().component),
            Some(1)
        );
        assert!(!traversal.did_cross_boundary());
        assert_eq!(
            child.component_local_id(traversal.next().unwrap().component),
            Some(0)
        );
        assert!(traversal.did_cross_boundary());
        assert_eq!(traversal.crossing_host(), Some(host));
        assert!(std::ptr::eq(traversal.source_artboard().unwrap(), &child));
        assert!(std::ptr::eq(traversal.current_artboard().unwrap(), &parent));
        assert_eq!(
            parent.component_local_id(traversal.next().unwrap().component),
            Some(1)
        );
        assert!(!traversal.did_cross_boundary());
        assert_eq!(traversal.crossing_host(), None);
        assert!(traversal.source_artboard().is_none());
    }

    #[test]
    fn transform_property_mutation_marks_instance_dirty() {
        let node_x_key = property_key_for_name("Node", "x").expect("Node.x key");
        let mut component = synthetic_component(0, 0);
        component.dirt = ComponentDirt::NONE;
        let mut instance = synthetic_instance(vec![component], vec![0]);
        instance.set_artboard_dirt_for_test(ComponentDirt::NONE);
        instance.did_change.set(false);

        assert!(instance.set_transform_property(0, TransformProperty::X, 12.0));
        let component = instance.component(0).unwrap();
        assert_eq!(
            instance.transform_property(0, TransformProperty::X),
            Some(12.0)
        );
        assert_eq!(instance.double_property(0, node_x_key), Some(12.0));
        assert!(
            component
                .dirt
                .contains(ComponentDirt::TRANSFORM | ComponentDirt::WORLD_TRANSFORM)
        );
        assert!(instance.has_dirt(ComponentDirt::COMPONENTS));
        assert!(instance.did_change());

        assert!(!instance.set_transform_property(0, TransformProperty::X, 12.0));
    }

    #[test]
    fn transform_property_mutation_rejects_missing_dense_local() {
        let node_x_key = property_key_for_name("Node", "x").expect("Node.x key");
        let mut instance = synthetic_instance(vec![synthetic_component(0, 0)], vec![0]);

        assert!(!instance.set_transform_property(1, TransformProperty::X, 12.0));
        assert!(!instance.set_transform_property_with_key(
            1,
            TransformProperty::X,
            node_x_key,
            12.0,
        ));
        assert_eq!(
            instance.transform_property(0, TransformProperty::X),
            Some(0.0)
        );
    }

    #[test]
    fn transform_property_mutation_writes_generated_storage_by_concrete_type() {
        let node_x_key = property_key_for_name("Node", "x").expect("Node.x key");
        let vertex_x_key = property_key_for_name("StraightVertex", "x").expect("StraightVertex.x");
        let mut vertex = synthetic_component(0, 0);
        vertex.type_name = "StraightVertex";
        let mut instance = synthetic_instance(vec![vertex], vec![0]);

        assert!(instance.set_transform_property(0, TransformProperty::X, 14.0));

        assert_eq!(
            instance.transform_property(0, TransformProperty::X),
            Some(14.0)
        );
        assert_eq!(instance.double_property(0, vertex_x_key), Some(14.0));
        assert_eq!(instance.double_property(0, node_x_key), None);
    }

    #[test]
    fn keyed_path_vertex_geometry_mutation_dirties_parent_path() {
        // C++ routes Vertex::xChanged through
        // PathVertex::markGeometryDirty to Path::markPathDirty; the parent
        // PointsPath owns the rebuilt RawPath (`vertex.cpp:14-15`,
        // `path_vertex.cpp:21-30`, `path.cpp:327-334`).
        let vertex_x_key = property_key_for_name("StraightVertex", "x").expect("StraightVertex.x");
        let mut path = synthetic_component(0, 0);
        path.type_name = "PointsPath";
        path.capabilities = RuntimeComponentCapabilities::default();
        let mut vertex = synthetic_component(1, 1);
        vertex.type_name = "StraightVertex";
        vertex.capabilities = RuntimeComponentCapabilities::default();
        let mut instance = synthetic_instance(vec![path, vertex], vec![0, 1]);
        synthetic_link_parent(&mut instance, 1, 0);
        instance.clear_component_dirt(0);
        instance.clear_component_dirt(1);
        instance.set_artboard_dirt_for_test(ComponentDirt::NONE);

        assert!(instance.set_keyed_double_property(1, vertex_x_key, 14.0));
        assert!(
            instance
                .component(0)
                .expect("PointsPath component")
                .dirt
                .contains(ComponentDirt::PATH)
        );
        assert!(
            !instance
                .component(1)
                .expect("StraightVertex component")
                .dirt
                .contains(ComponentDirt::PATH)
        );
    }

    #[test]
    fn inherited_parametric_and_path_callbacks_dirty_geometry_and_layout_owner() {
        let layout = synthetic_component_for_type(0, "LayoutComponent");
        let shape = synthetic_component_for_type(1, "Shape");
        let rectangle = synthetic_component_for_type(2, "Rectangle");
        let mut instance = synthetic_instance(vec![layout, shape, rectangle], vec![0, 1, 2]);
        synthetic_link_parent(&mut instance, 1, 0);
        synthetic_link_parent(&mut instance, 2, 1);

        let width = property_key_for_name("Rectangle", "width").expect("Rectangle.width");
        let is_hole = property_key_for_name("Rectangle", "isHole").expect("Rectangle.isHole");
        instance.clear_component_dirt(0);
        instance.clear_component_dirt(1);
        instance.clear_component_dirt(2);
        instance.set_artboard_dirt_for_test(ComponentDirt::NONE);
        let layout_node_revision = instance
            .component(0)
            .and_then(|component| component.concrete.layout.as_ref())
            .expect("layout owner")
            .layout_node_revision();

        assert!(instance.set_double_property(2, width, 24.0));
        assert!(
            instance
                .component(2)
                .expect("Rectangle component")
                .dirt
                .contains(ComponentDirt::PATH)
        );
        assert!(
            instance
                .component(0)
                .and_then(|component| component.concrete.layout.as_ref())
                .expect("layout owner")
                .layout_node_revision()
                > layout_node_revision
        );

        instance.clear_component_dirt(2);
        instance.set_artboard_dirt_for_test(ComponentDirt::NONE);
        assert!(instance.set_bool_property(2, is_hole, true));
        assert!(
            instance
                .component(2)
                .expect("Rectangle component")
                .dirt
                .contains(ComponentDirt::PATH)
        );
    }

    #[test]
    fn transform_property_mutation_only_recurses_world_transform_to_dependents() {
        let source = synthetic_component(0, 0);
        let dependent = synthetic_component(1, 1);
        let mut instance = synthetic_instance(vec![source, dependent], vec![0, 1]);
        synthetic_add_dependent(&mut instance, 0, 1);
        instance.clear_component_dirt(0);
        instance.clear_component_dirt(1);
        instance.set_artboard_dirt_for_test(ComponentDirt::NONE);

        assert!(instance.set_transform_property(0, TransformProperty::X, 12.0));

        let source = instance.component(0).unwrap();
        assert!(source.dirt.contains(ComponentDirt::TRANSFORM));
        assert!(source.dirt.contains(ComponentDirt::WORLD_TRANSFORM));
        let dependent = instance.component(1).unwrap();
        assert!(!dependent.dirt.contains(ComponentDirt::TRANSFORM));
        assert!(dependent.dirt.contains(ComponentDirt::WORLD_TRANSFORM));
    }

    #[test]
    fn opacity_mutation_marks_render_opacity_dirty() {
        let mut component = synthetic_component(0, 0);
        component.dirt = ComponentDirt::NONE;
        let mut instance = synthetic_instance(vec![component], vec![0]);
        instance.set_artboard_dirt_for_test(ComponentDirt::NONE);

        assert!(instance.set_transform_property(0, TransformProperty::Opacity, 0.35));
        let component = instance.component(0).unwrap();
        assert_eq!(
            instance.transform_property(0, TransformProperty::Opacity),
            Some(0.35)
        );
        assert!(component.dirt.contains(ComponentDirt::RENDER_OPACITY));
        assert!(!component.dirt.contains(ComponentDirt::TRANSFORM));
    }

    #[test]
    fn generic_artboard_opacity_mutation_marks_render_opacity_dirty() {
        let artboard_opacity_key =
            property_key_for_name("Artboard", "opacity").expect("Artboard.opacity key");
        let mut root = synthetic_component(0, 0);
        root.type_name = "Artboard";
        root.dirt = ComponentDirt::NONE;
        let mut instance = synthetic_instance(vec![root], vec![0]);
        instance.set_artboard_dirt_for_test(ComponentDirt::NONE);

        assert!(instance.set_double_property(0, artboard_opacity_key, 0.25));

        let root = instance.component(0).unwrap();
        assert_eq!(
            instance.transform_property(0, TransformProperty::Opacity),
            Some(0.25)
        );
        assert!(root.dirt.contains(ComponentDirt::RENDER_OPACITY));
        assert!(!root.dirt.contains(ComponentDirt::TRANSFORM));
    }

    #[test]
    fn host_opacity_multiplies_without_overwriting_artboard_opacity() {
        let artboard_opacity_key =
            property_key_for_name("Artboard", "opacity").expect("Artboard.opacity key");
        let mut root = synthetic_component_for_type(0, "Artboard");
        root.dirt = ComponentDirt::NONE;
        let mut instance = synthetic_instance(vec![root], vec![0]);

        assert!(instance.set_double_property(0, artboard_opacity_key, 0.4));
        instance.update_components();
        instance.clear_component_dirt(0);
        instance.set_artboard_dirt_for_test(ComponentDirt::NONE);

        assert!(instance.set_host_opacity(0.5));
        assert_eq!(instance.double_property(0, artboard_opacity_key), Some(0.4));
        assert_eq!(instance.child_opacity(), 0.2);
        assert!(
            instance
                .component(0)
                .unwrap()
                .dirt
                .contains(ComponentDirt::RENDER_OPACITY)
        );
    }

    #[test]
    fn artboard_self_transform_combines_rotation_and_scale() {
        let root = synthetic_component_for_type(0, "Artboard");
        let mut instance = synthetic_instance(vec![root], vec![0]);

        assert!(!instance.has_self_transform());
        assert!(instance.set_transform_property(
            0,
            TransformProperty::Rotation,
            std::f32::consts::FRAC_PI_2,
        ));
        assert!(instance.set_transform_property(0, TransformProperty::ScaleX, 2.0));
        assert!(instance.set_transform_property(0, TransformProperty::ScaleY, 3.0));

        assert!(instance.has_self_transform());
        let transform = instance.self_transform().0;
        assert!(transform[0].abs() < 1e-6);
        assert!((transform[1] - 2.0).abs() < 1e-6);
        assert!((transform[2] + 3.0).abs() < 1e-6);
        assert!(transform[3].abs() < 1e-6);
        assert_eq!(&transform[4..], &[0.0, 0.0]);
        assert_eq!(
            instance.mounted_root_transform(Mat2D([1.0, 0.0, 0.0, 1.0, 4.0, 5.0])),
            Mat2D([
                transform[0],
                transform[1],
                transform[2],
                transform[3],
                4.0,
                5.0,
            ])
        );
    }

    #[test]
    fn update_reads_mutated_instance_transform_state() {
        let mut component = synthetic_component(0, 0);
        component.dirt = ComponentDirt::NONE;
        let mut instance = synthetic_instance(vec![component], vec![0]);
        instance.set_artboard_dirt_for_test(ComponentDirt::NONE);

        assert!(instance.set_transform_property(0, TransformProperty::X, 9.0));
        assert!(instance.set_transform_property(0, TransformProperty::Y, 4.0));

        let report = instance.update_components();

        assert_eq!(report.updated_locals, vec![0]);
        assert_eq!(
            instance.component(0).unwrap().transform.local_transform,
            Mat2D([1.0, 0.0, -0.0, 1.0, 9.0, 4.0])
        );
    }

    #[test]
    fn builds_instance_from_graph_fixture() {
        let bytes = include_bytes!("../../../../fixtures/graph/dependency_test.riv");
        let file = read_runtime_file(bytes).expect("fixture should import");
        let graph = GraphFile::from_runtime_file(&file).expect("fixture should graph");
        let artboard = graph.artboards.first().expect("fixture has artboard");
        let instance = ArtboardInstance::from_graph(&file, artboard).expect("instance builds");

        assert_eq!(instance.slots().len(), artboard.local_objects.len());
        assert_eq!(
            instance
                .slots()
                .iter()
                .map(|slot| (slot.local_id, slot.source_global_id, slot.type_name))
                .collect::<Vec<_>>(),
            artboard
                .local_objects
                .iter()
                .map(|object| (object.local_id, object.global_id, object.type_name))
                .collect::<Vec<_>>()
        );
        assert_eq!(instance.components().len(), artboard.components.len());
        let graph_ordered_components = artboard
            .components
            .iter()
            .filter(|component| component.graph_order.is_some())
            .count();
        assert_eq!(
            instance
                .objects
                .dependency_order()
                .iter()
                .filter(|handle| {
                    matches!(
                        instance.objects.address(**handle),
                        Some(ComponentAddress::Authored(_))
                    )
                })
                .count(),
            graph_ordered_components
        );
        assert!(instance.has_dirt(ComponentDirt::COMPONENTS));
        assert!(
            instance
                .components()
                .iter()
                .all(|component| component.dirt == ComponentDirt::FILTHY)
        );
    }

    #[test]
    fn occurrence_schedule_keeps_embedded_path_composer_identity_exact() {
        let bytes = include_bytes!("../../../../fixtures/graph/clipping_and_draw_order.riv");
        let file = read_runtime_file(bytes).expect("fixture should import");
        let graph = GraphFile::from_runtime_file(&file).expect("fixture should graph");
        let artboard = graph.artboards.first().expect("fixture has artboard");
        let instance = ArtboardInstance::from_graph(&file, artboard).expect("instance builds");

        let (source_local, shape_local, dependent_local) = artboard
            .dependency_nodes
            .iter()
            .enumerate()
            .find_map(|(composer_node, node)| {
                let DependencyNodeKind::PathComposer { shape_local, .. } = node.kind else {
                    return None;
                };
                let source_local = artboard
                    .dependency_node_edges_in_insertion_order
                    .iter()
                    .find(|edge| edge.dependent_node == composer_node)
                    .and_then(|edge| artboard.dependency_nodes.get(edge.source_node))
                    .and_then(|node| match node.kind {
                        DependencyNodeKind::Component { local_id, .. } => Some(local_id),
                        _ => None,
                    })?;
                let dependent_local = artboard
                    .dependency_node_edges_in_insertion_order
                    .iter()
                    .find(|edge| edge.source_node == composer_node)
                    .and_then(|edge| artboard.dependency_nodes.get(edge.dependent_node))
                    .and_then(|node| match node.kind {
                        DependencyNodeKind::Component { local_id, .. } => Some(local_id),
                        _ => None,
                    })?;
                Some((source_local, shape_local, dependent_local))
            })
            .expect("fixture has Component -> PathComposer -> Component dependency chain");

        let source = instance
            .component_handle(source_local)
            .expect("source handle");
        let shape = instance
            .component_handle(shape_local)
            .expect("shape handle");
        let composer = instance
            .objects
            .path_composer_handle(shape_local)
            .expect("embedded PathComposer handle");
        let dependent = instance
            .component_handle(dependent_local)
            .expect("dependent handle");

        assert_ne!(shape, composer);
        assert_eq!(
            instance.objects.component_local_id(shape),
            instance.objects.component_local_id(composer)
        );
        assert!(matches!(
            instance.objects.address(composer),
            Some(ComponentAddress::PathComposer(_))
        ));
        assert!(
            instance.objects.graph_order(source).unwrap().index()
                < instance.objects.graph_order(composer).unwrap().index()
        );
        assert!(
            instance.objects.graph_order(composer).unwrap().index()
                < instance.objects.graph_order(dependent).unwrap().index()
        );
    }

    #[test]
    fn occurrence_schedule_excludes_draw_order_only_components() {
        let bytes = synthetic_riv(13_103, |bytes| {
            push_synthetic_object(bytes, "Backboard", &[]);
            push_synthetic_object(bytes, "Artboard", &[]);
            push_synthetic_object(bytes, "Shape", &[("parentId", 0)]);
            push_synthetic_object(bytes, "DrawTarget", &[("parentId", 0), ("drawableId", 1)]);
            push_synthetic_object(bytes, "DrawRules", &[("parentId", 0), ("drawTargetId", 2)]);
        });
        let instance = instance_from_riv(&bytes);
        let draw_target = instance
            .component_handle(2)
            .expect("DrawTarget remains a live Component occurrence");
        let draw_rules = instance
            .component_handle(3)
            .expect("DrawRules remains a live Component occurrence");

        assert!(
            !instance.objects.dependency_order().contains(&draw_target),
            "DrawTarget inherits empty Component::buildDependencies; its drawable edge belongs only to Artboard draw-order sorting (`src/draw_target.cpp`; `include/rive/component.hpp:50`; `src/artboard.cpp:409-435`)"
        );
        assert!(
            !instance.objects.dependency_order().contains(&draw_rules),
            "DrawRules inherits empty Component::buildDependencies; its target edge belongs only to Artboard draw-order sorting (`src/draw_rules.cpp`; `include/rive/component.hpp:50`; `src/artboard.cpp:409-435`)"
        );
    }

    #[test]
    fn follow_path_node_target_adds_no_path_source_dependency() {
        let bytes = synthetic_riv(13_102, |bytes| {
            push_synthetic_object(bytes, "Backboard", &[]);
            push_synthetic_object(bytes, "Artboard", &[]);
            push_synthetic_object(bytes, "Node", &[("parentId", 0)]);
            push_synthetic_object(bytes, "Node", &[("parentId", 0)]);
            push_synthetic_object(
                bytes,
                "FollowPathConstraint",
                &[("parentId", 1), ("targetId", 2)],
            );
        });
        let instance = instance_from_riv(&bytes);
        let target = instance.component_handle(2).expect("ordinary Node target");
        let constraint = instance
            .component_handle(3)
            .expect("FollowPathConstraint handle");
        let constrained_parent = instance
            .component_handle(1)
            .expect("constrained parent handle");

        assert!(
            !(0..instance.objects.dependent_len(target))
                .filter_map(|index| instance.objects.dependent_at(target, index))
                .any(|dependent| dependent == constraint),
            "FollowPath adds a source edge only for Shape/Path targets (`follow_path_constraint.cpp:167-186`)"
        );
        assert!(
            (0..instance.objects.dependent_len(constraint))
                .filter_map(|index| instance.objects.dependent_at(constraint, index))
                .any(|dependent| dependent == constrained_parent)
        );
    }

    #[test]
    fn occurrence_schedule_interleaves_text_helper_before_later_root_child() {
        let bytes = synthetic_riv(9589, |bytes| {
            push_synthetic_object(bytes, "Backboard", &[]);
            push_synthetic_object(bytes, "Artboard", &[]);
            push_synthetic_object(bytes, "Text", &[("parentId", 0)]);
            push_synthetic_object(bytes, "TextStyle", &[("parentId", 1)]);
            push_synthetic_object(bytes, "TextStyleAxis", &[("parentId", 2)]);
            push_synthetic_object(bytes, "Node", &[("parentId", 0)]);
        });
        let instance = instance_from_riv(&bytes);
        let root = instance.component_handle(0).expect("root handle");
        let text = instance.component_handle(1).expect("Text handle");
        let style = instance.component_handle(2).expect("TextStyle handle");
        let helper = instance
            .objects
            .text_variation_helper_handle(2)
            .expect("TextStyle owns a variation helper");
        let later_child = instance.component_handle(4).expect("later Node handle");

        assert_eq!(
            (0..instance.objects.dependent_len(root))
                .filter_map(|index| instance.objects.dependent_at(root, index))
                .collect::<Vec<_>>(),
            vec![text, helper, later_child],
            "TextStyle invokes helper buildDependencies before its own parent edge, and the later Node runs afterward in object order (`text_style.cpp:128-136`; `text_variation_helper.cpp:7-12`; `artboard.cpp:417-428`)"
        );
        assert_eq!(
            instance.objects.dependency_order(),
            &[root, later_child, helper, text, style],
            "DependencySorter front-inserts completed siblings after visiting retained insertion order (`dependency_sorter.cpp:6-48`)"
        );
    }

    #[test]
    fn clone_rebuilds_component_relations_in_clone_owned_storage() {
        let bytes = include_bytes!("../../../../fixtures/graph/clipping_and_draw_order.riv");
        let file = read_runtime_file(bytes).expect("fixture should import");
        let graph = GraphFile::from_runtime_file(&file).expect("fixture should graph");
        let artboard = graph.artboards.first().expect("fixture has artboard");
        let instance = ArtboardInstance::from_graph(&file, artboard).expect("instance builds");
        let source = instance
            .objects
            .dependency_order()
            .iter()
            .copied()
            .find(|handle| instance.objects.dependent_len(*handle) > 0)
            .expect("fixture has a dependency source");
        let source_dependent = instance
            .objects
            .dependent_at(source, 0)
            .expect("source has a dependent");
        let mut instance = instance;
        let fabricated_bind = DataBindHandle::from_index(usize::MAX);
        assert!(instance.objects.add_collapsable(source, fabricated_bind));
        let source_path = instance
            .objects
            .component_handles()
            .iter()
            .copied()
            .find(|handle| {
                instance
                    .objects
                    .component(*handle)
                    .and_then(|component| component.concrete.path.as_ref())
                    .and_then(|path| path.shape)
                    .is_some()
            })
            .expect("fixture has a Path with a retained Shape owner");
        let source_shape = instance
            .objects
            .component(source_path)
            .and_then(|component| component.concrete.path.as_ref())
            .and_then(|path| path.shape)
            .expect("source Path retained its Shape");
        let source_composer = instance
            .objects
            .component_local_id(source_shape)
            .and_then(|local| instance.objects.path_composer_handle(local))
            .expect("source Shape owns its embedded PathComposer");

        let mut cloned = instance.clone();

        assert!(!std::ptr::eq(
            instance.objects.component(source).unwrap(),
            cloned.objects.component(source).unwrap()
        ));
        assert_eq!(
            cloned.objects.dependent_at(source, 0),
            Some(source_dependent)
        );
        assert_eq!(
            cloned.objects.graph_order(source),
            instance.objects.graph_order(source)
        );
        assert_eq!(
            cloned.objects.collapsable_len(source),
            0,
            "cloneObjectDataBinds rebuilds m_collapsables from cloned authored DataBinds instead of copying the source pointer list (`artboard.cpp:1038-1057`)"
        );
        assert_eq!(
            instance.objects.collapsable_at(source, 0),
            Some(fabricated_bind)
        );
        assert_eq!(
            cloned.objects.dependency_order(),
            instance.objects.dependency_order(),
            "clone rebuild must reproduce the authored occurrence schedule"
        );
        let cloned_shape = cloned
            .objects
            .component(source_path)
            .and_then(|component| component.concrete.path.as_ref())
            .and_then(|path| path.shape)
            .expect("clone Path rebuilt its clone-owned Shape pointer");
        assert_eq!(cloned_shape, source_shape);
        assert!(
            cloned
                .objects
                .component(cloned_shape)
                .and_then(|component| component.concrete.shape.as_ref())
                .is_some_and(|shape| shape.paths.contains(&source_path))
        );
        assert_eq!(
            cloned
                .objects
                .component_local_id(cloned_shape)
                .and_then(|local| cloned.objects.path_composer_handle(local)),
            Some(source_composer),
            "Path reaches the clone-owned embedded composer only through its retained Shape (`path.cpp:76-96`; `follow_path_constraint.cpp:175-186`)"
        );
        assert!(!std::ptr::eq(
            instance.objects.component(source_shape).unwrap(),
            cloned.objects.component(cloned_shape).unwrap()
        ));

        cloned
            .objects
            .component_mut(source)
            .unwrap()
            .dependents
            .clear();
        cloned.objects.component_mut(source).unwrap().dirt = ComponentDirt::NONE;

        assert_eq!(cloned.objects.dependent_len(source), 0);
        assert!(instance.objects.dependent_len(source) > 0);
        assert_ne!(
            cloned.objects.component(source).unwrap().dirt,
            instance.objects.component(source).unwrap().dirt
        );
    }

    #[test]
    fn clone_rebuilds_parent_links_from_clone_owned_generated_parent_id() {
        let bytes = synthetic_riv(9590, |bytes| {
            push_synthetic_object(bytes, "Backboard", &[]);
            push_synthetic_object(bytes, "Artboard", &[]);
            push_synthetic_object(bytes, "Node", &[("parentId", 0)]);
            push_synthetic_object(bytes, "Node", &[("parentId", 0)]);
            push_synthetic_object(bytes, "Node", &[("parentId", 1)]);
        });
        let mut instance = instance_from_riv(&bytes);
        let parent_key = property_key_for_name("Component", "parentId").expect("parentId key");

        assert_eq!(instance.component_parent_local(3), Some(1));
        assert!(instance.set_uint_property(3, parent_key, 2));
        assert_eq!(
            instance.component_parent_local(3),
            Some(1),
            "Component.parentIdChanged is intentionally a no-op on the live occurrence"
        );

        let cloned = instance.clone();
        assert_eq!(
            cloned.component_parent_local(3),
            Some(2),
            "fresh clone onAddedDirty resolves the copied generated parentId (`src/component.cpp:19-29`)"
        );
        assert_eq!(instance.component_parent_local(3), Some(1));
    }

    #[test]
    fn targeted_constraint_retains_live_target_and_clone_reresolves_generated_target_id() {
        let bytes = synthetic_riv(9_594, |bytes| {
            push_synthetic_object(bytes, "Backboard", &[]);
            push_synthetic_object(bytes, "Artboard", &[]);
            push_synthetic_object(bytes, "Node", &[("parentId", 0)]);
            push_synthetic_object(bytes, "Node", &[("parentId", 0)]);
            push_synthetic_object(bytes, "Node", &[("parentId", 0)]);
            push_synthetic_object(
                bytes,
                "RotationConstraint",
                &[("parentId", 1), ("targetId", 2)],
            );
        });
        let mut instance = instance_from_riv(&bytes);
        let owner = instance.component_handle(1).expect("constraint owner");
        let first_target = instance.component_handle(2).expect("first target");
        let second_target = instance.component_handle(3).expect("second target");
        let constraint = instance.component_handle(4).expect("constraint");

        assert_eq!(instance.objects.constraint_len(owner), 1);
        assert_eq!(instance.objects.constraint_at(owner, 0), Some(constraint));
        assert_eq!(
            instance
                .component_at(constraint)
                .concrete
                .constraint
                .expect("retained constraint state")
                .target,
            Some(first_target)
        );
        assert_eq!(
            instance.objects.dependent_at(first_target, 0),
            Some(owner),
            "TargetedConstraint::buildDependencies adds target -> constrained parent (`src/constraints/targeted_constraint.cpp:42-49`)"
        );

        let target_key =
            property_key_for_name("TargetedConstraint", "targetId").expect("targetId key");
        instance.clear_component_dirt(1);
        assert!(instance.set_uint_property(4, target_key, 3));
        assert_eq!(
            instance
                .component_at(constraint)
                .concrete
                .constraint
                .expect("retained constraint state")
                .target,
            Some(first_target),
            "TargetedConstraintBase::targetIdChanged is intentionally empty; a live occurrence does not retarget (`generated/constraints/targeted_constraint_base.hpp`)"
        );
        assert!(
            !instance
                .component_at(owner)
                .dirt
                .contains(ComponentDirt::TRANSFORM),
            "targetIdChanged must not dirty the constrained parent"
        );

        let cloned = instance.clone();
        assert_eq!(
            cloned
                .component_at(constraint)
                .concrete
                .constraint
                .expect("clone retained constraint state")
                .target,
            Some(second_target),
            "fresh clone onAddedDirty resolves the copied generated targetId (`src/constraints/targeted_constraint.cpp:23-39`)"
        );
        assert_eq!(cloned.objects.constraint_at(owner, 0), Some(constraint));
        assert_eq!(cloned.objects.dependent_at(second_target, 0), Some(owner));
        assert_eq!(instance.objects.dependent_at(first_target, 0), Some(owner));

        instance.clear_component_dirt(1);
        instance.clear_component_dirt(4);
        assert!(instance.add_dirt(4, ComponentDirt::PATH, false));
        assert!(
            instance
                .component_at(owner)
                .dirt
                .contains(ComponentDirt::TRANSFORM),
            "Constraint::onDirty marks its retained parent for every non-empty accumulated dirt (`src/constraints/constraint.cpp:23-29`)"
        );

        instance.clear_component_dirt(1);
        let strength_key = property_key_for_name("Constraint", "strength").expect("strength key");
        assert!(instance.set_double_property(4, strength_key, 0.5));
        assert!(
            instance
                .component_at(owner)
                .dirt
                .contains(ComponentDirt::TRANSFORM),
            "ConstraintBase::strengthChanged dirties the retained constrained parent"
        );
    }

    #[test]
    fn targeted_constraint_target_validation_preserves_optional_and_required_contracts() {
        let optional = synthetic_riv(9_595, |bytes| {
            push_synthetic_object(bytes, "Backboard", &[]);
            push_synthetic_object(bytes, "Artboard", &[]);
            push_synthetic_object(bytes, "Node", &[("parentId", 0)]);
            push_synthetic_object(bytes, "RotationConstraint", &[("parentId", 1)]);
        });
        let optional = instance_from_riv(&optional);
        let optional_constraint = optional.component_handle(2).expect("optional constraint");
        assert_eq!(
            optional
                .component_at(optional_constraint)
                .concrete
                .constraint
                .expect("constraint state")
                .target,
            None
        );

        let required = synthetic_riv(9_596, |bytes| {
            push_synthetic_object(bytes, "Backboard", &[]);
            push_synthetic_object(bytes, "Artboard", &[]);
            push_synthetic_object(bytes, "Node", &[("parentId", 0)]);
            push_synthetic_object(bytes, "DistanceConstraint", &[("parentId", 1)]);
        });
        let file = read_runtime_file(&required).expect("required fixture imports");
        let graphs = GraphFile::from_runtime_file(&file).expect("required fixture graphs");
        let required = ArtboardInstance::from_graph(&file, &graphs.artboards[0])
            .expect("invalid required-target constraint is filtered before initialize");
        assert!(
            required
                .components()
                .iter()
                .all(|component| component.type_name != "DistanceConstraint"),
            "Artboard::validateObjects removes invalid Components from m_Objects before \
             initialize; MissingObject from onAddedDirty is non-fatal (`artboard.cpp:204-245, \
             264-288`; `targeted_constraint.cpp:7-38`)"
        );

        let wrong_type = synthetic_riv(9_597, |bytes| {
            push_synthetic_object(bytes, "Backboard", &[]);
            push_synthetic_object(bytes, "Artboard", &[]);
            push_synthetic_object(bytes, "Node", &[("parentId", 0)]);
            push_synthetic_object(bytes, "Skin", &[("parentId", 0)]);
            push_synthetic_object(
                bytes,
                "DistanceConstraint",
                &[("parentId", 1), ("targetId", 2)],
            );
        });
        let file = read_runtime_file(&wrong_type).expect("wrong-type fixture imports");
        let graphs = GraphFile::from_runtime_file(&file).expect("wrong-type fixture graphs");
        let wrong_type = ArtboardInstance::from_graph(&file, &graphs.artboards[0])
            .expect("wrong-type target constraint is filtered before initialize");
        assert!(
            wrong_type
                .components()
                .iter()
                .all(|component| component.type_name != "DistanceConstraint"),
            "TargetedConstraint::validate rejects a non-Transform target and the Artboard removes \
             that invalid object before lifecycle dispatch"
        );
    }

    #[test]
    fn transform_constraint_registration_is_an_unconditional_ordered_push() {
        let owner = synthetic_component_for_type(0, "Node");
        let first = synthetic_component_for_type(1, "RotationConstraint");
        let second = synthetic_component_for_type(2, "ScaleConstraint");
        let mut instance = synthetic_instance(vec![owner, first, second], vec![0, 1, 2]);
        let owner = instance.component_handle(0).unwrap();
        let first = instance.component_handle(1).unwrap();
        let second = instance.component_handle(2).unwrap();

        assert!(instance.objects.add_constraint(owner, first));
        assert!(instance.objects.add_constraint(owner, second));
        assert!(instance.objects.add_constraint(owner, first));
        assert_eq!(
            (0..instance.objects.constraint_len(owner))
                .map(|index| instance.objects.constraint_at(owner, index).unwrap())
                .collect::<Vec<_>>(),
            vec![first, second, first],
            "TransformComponent::addConstraint unconditionally push_backs in call order (`src/transform_component.cpp:123-126`)"
        );
    }

    #[test]
    fn clone_relinks_text_variation_helper_to_clone_owned_text_parent() {
        let bytes = synthetic_riv(9591, |bytes| {
            push_synthetic_object(bytes, "Backboard", &[]);
            push_synthetic_object(bytes, "Artboard", &[]);
            push_synthetic_object(bytes, "Text", &[("parentId", 0)]);
            push_synthetic_object(bytes, "TextStyle", &[("parentId", 1)]);
            push_synthetic_object(bytes, "TextStyleAxis", &[("parentId", 2)]);
            push_synthetic_object(bytes, "Text", &[("parentId", 0)]);
        });
        let mut instance = instance_from_riv(&bytes);
        let parent_key = property_key_for_name("Component", "parentId").expect("parentId key");
        let first_text = instance.component_handle(1).expect("first Text handle");
        let second_text = instance.component_handle(4).expect("second Text handle");

        assert_eq!(
            instance.objects.text_variation_helper_text_handle(2),
            Some(first_text)
        );
        assert!(instance.set_uint_property(2, parent_key, 4));

        let cloned = instance.clone();
        assert_eq!(cloned.component_parent_local(2), Some(4));
        assert_eq!(
            cloned.objects.text_variation_helper_text_handle(2),
            Some(second_text),
            "TextStyle::onAddedClean refreshes m_text before creating the clone-owned helper (`src/text/text_style.cpp:45-70`)"
        );
        let helper = cloned
            .objects
            .text_variation_helper_handle(2)
            .expect("helper handle");
        assert_eq!(
            cloned.objects.dependent_at(helper, 0),
            Some(second_text),
            "TextVariationHelper::buildDependencies reads the rebuilt TextStyle parent (`src/text/text_variation_helper.cpp:7-12`)"
        );
        assert_eq!(
            instance.objects.text_variation_helper_text_handle(2),
            Some(first_text)
        );
    }

    #[test]
    fn data_bind_collapsables_link_in_import_order_and_rebuild_on_clone_initialize() {
        let bytes = include_bytes!("../../../../fixtures/flow/data_binding_test.riv");
        let file = read_runtime_file(bytes).expect("fixture should import");
        let graph = GraphFile::from_runtime_file(&file).expect("fixture should graph");
        let artboard = graph.artboards.first().expect("fixture has artboard");
        let instance = ArtboardInstance::from_graph(&file, artboard).expect("instance builds");
        let artboard_index = file
            .artboards()
            .iter()
            .position(|candidate| candidate.id == artboard.global_id)
            .expect("graph artboard maps to runtime file");

        let mut expected = BTreeMap::<ComponentHandle, Vec<DataBindHandle>>::new();
        for (data_bind_index, data_bind) in file
            .artboard_data_binds(artboard_index)
            .into_iter()
            .enumerate()
        {
            let Some(target_local_id) = data_bind.target_local_id else {
                continue;
            };
            let Some(component) = instance.objects.component_handle(target_local_id) else {
                continue;
            };
            expected
                .entry(component)
                .or_default()
                .push(DataBindHandle::from_index(data_bind_index));
        }
        assert!(
            !expected.is_empty(),
            "fixture must exercise Component targets"
        );
        for (component, data_binds) in &expected {
            assert_eq!(
                (0..instance.objects.collapsable_len(*component))
                    .filter_map(|index| instance.objects.collapsable_at(*component, index))
                    .collect::<Vec<_>>(),
                *data_binds
            );
        }

        let mut cloned = instance.clone();
        for (component, data_binds) in expected {
            assert_eq!(
                (0..cloned.objects.collapsable_len(component))
                    .filter_map(|index| cloned.objects.collapsable_at(component, index))
                    .collect::<Vec<_>>(),
                data_binds
            );
        }
        let _ = cloned.advance_artboard_data_binds();
    }

    #[test]
    fn unattached_import_only_paths_do_not_rearm_runtime_traversal() {
        let bytes = include_bytes!("../../../../fixtures/graph/dependency_test.riv");
        let file = read_runtime_file(bytes).expect("fixture should import");
        let graph = GraphFile::from_runtime_file(&file).expect("fixture should graph");
        let artboard = graph.artboards.first().expect("fixture has artboard");
        let mut instance = ArtboardInstance::from_graph(&file, artboard).expect("instance builds");

        assert!(instance.update_components().did_update);
        assert!(
            !instance.update_components().did_update,
            "C++ leaves unattached import-only components out of the rooted runtime schedule; their cold Path owners must not re-arm Artboard dirt"
        );
    }

    #[test]
    fn construction_seeds_file_owned_external_fonts_on_the_root_instance() {
        let bytes = include_bytes!("../../../../fixtures/graph/dependency_test.riv");
        let file = read_runtime_file(bytes).expect("fixture should import");
        let graph = GraphFile::from_runtime_file(&file).expect("fixture should graph");
        let artboard = graph.artboards.first().expect("fixture has artboard");
        let font_bytes = Arc::<[u8]>::from(vec![1, 2, 3]);
        let external_fonts = BTreeMap::from([(7, Arc::clone(&font_bytes))]);

        let instance = ArtboardInstance::from_graph_with_artboards_and_external_fonts(
            &file,
            artboard,
            &graph.artboards,
            &external_fonts,
        )
        .expect("instance builds with file-owned fonts");

        assert_eq!(instance.external_font_asset_bytes(7), Some(&*font_bytes));
        assert_eq!(
            instance
                .build_context
                .as_ref()
                .and_then(|context| context.external_font_assets.get(&7))
                .map(AsRef::as_ref),
            Some(&*font_bytes)
        );
    }

    #[test]
    fn replacing_file_owned_fonts_updates_existing_nested_children() {
        let mut instance = synthetic_instance(Vec::new(), Vec::new());
        instance
            .nested_artboards
            .insert(7, synthetic_nested_artboard_instance(0));
        let font_bytes = Arc::<[u8]>::from(vec![1, 2, 3]);
        let external_fonts = BTreeMap::from([(7, Arc::clone(&font_bytes))]);

        instance.replace_external_font_asset_snapshot(&external_fonts);

        let nested = instance
            .nested_artboards
            .get(&7)
            .expect("nested child exists");
        assert_eq!(
            nested.child.external_font_asset_bytes(7),
            Some(&*font_bytes)
        );
        assert!(Arc::ptr_eq(
            &instance.external_font_assets,
            &nested.child.external_font_assets
        ));
    }

    fn push_var_uint(bytes: &mut Vec<u8>, mut value: u64) {
        loop {
            let mut byte = (value & 0x7f) as u8;
            value >>= 7;
            if value != 0 {
                byte |= 0x80;
            }
            bytes.push(byte);
            if value == 0 {
                break;
            }
        }
    }

    fn schema_type_key(type_name: &str) -> u16 {
        definition_by_name(type_name)
            .unwrap_or_else(|| panic!("missing schema definition {type_name}"))
            .type_key
            .int
    }

    fn schema_property_key(type_name: &str, property_name: &str) -> u16 {
        property_key_for_name(type_name, property_name)
            .unwrap_or_else(|| panic!("missing property {type_name}.{property_name}"))
    }

    fn push_synthetic_object(bytes: &mut Vec<u8>, type_name: &str, properties: &[(&str, u64)]) {
        push_synthetic_object_with_properties(bytes, type_name, |bytes| {
            for (property_name, value) in properties {
                push_synthetic_uint_property(bytes, type_name, property_name, *value);
            }
        });
    }

    fn push_synthetic_object_with_properties(
        bytes: &mut Vec<u8>,
        type_name: &str,
        properties: impl FnOnce(&mut Vec<u8>),
    ) {
        push_var_uint(bytes, u64::from(schema_type_key(type_name)));
        properties(bytes);
        push_var_uint(bytes, 0);
    }

    fn push_synthetic_uint_property(
        bytes: &mut Vec<u8>,
        type_name: &str,
        property_name: &str,
        value: u64,
    ) {
        push_var_uint(
            bytes,
            u64::from(schema_property_key(type_name, property_name)),
        );
        push_var_uint(bytes, value);
    }

    fn push_synthetic_string_property(
        bytes: &mut Vec<u8>,
        type_name: &str,
        property_name: &str,
        value: &str,
    ) {
        push_var_uint(
            bytes,
            u64::from(schema_property_key(type_name, property_name)),
        );
        push_var_uint(bytes, value.len() as u64);
        bytes.extend_from_slice(value.as_bytes());
    }

    fn push_synthetic_f32_property(
        bytes: &mut Vec<u8>,
        type_name: &str,
        property_name: &str,
        value: f32,
    ) {
        push_var_uint(
            bytes,
            u64::from(schema_property_key(type_name, property_name)),
        );
        bytes.extend_from_slice(&value.to_le_bytes());
    }

    fn push_synthetic_bytes_property(
        bytes: &mut Vec<u8>,
        type_name: &str,
        property_name: &str,
        value: &[u8],
    ) {
        push_var_uint(
            bytes,
            u64::from(schema_property_key(type_name, property_name)),
        );
        push_var_uint(bytes, value.len() as u64);
        bytes.extend_from_slice(value);
    }

    fn synthetic_owned_view_model_action_riv(file_id: u64, listener_action: bool) -> Vec<u8> {
        synthetic_owned_view_model_action_riv_with_options(
            file_id,
            listener_action,
            false,
            false,
            false,
        )
    }

    fn synthetic_owned_view_model_action_riv_with_options(
        file_id: u64,
        listener_action: bool,
        cross_model_trigger_action: bool,
        listener_cascade: bool,
        unrelated_two_way_bind: bool,
    ) -> Vec<u8> {
        synthetic_riv(file_id, |bytes| {
            push_synthetic_object(bytes, "FontAsset", &[("assetId", 17)]);
            push_synthetic_object(bytes, "ViewModel", &[]);
            push_synthetic_object_with_properties(bytes, "ViewModelPropertyList", |bytes| {
                push_synthetic_bytes_property(bytes, "ViewModelPropertyList", "name", b"items");
            });
            push_synthetic_object_with_properties(bytes, "ViewModelPropertyViewModel", |bytes| {
                push_synthetic_bytes_property(
                    bytes,
                    "ViewModelPropertyViewModel",
                    "name",
                    b"child",
                );
                push_synthetic_uint_property(
                    bytes,
                    "ViewModelPropertyViewModel",
                    "viewModelReferenceId",
                    1,
                );
            });
            push_synthetic_object_with_properties(bytes, "ViewModelPropertyViewModel", |bytes| {
                push_synthetic_bytes_property(
                    bytes,
                    "ViewModelPropertyViewModel",
                    "name",
                    b"other_child",
                );
                push_synthetic_uint_property(
                    bytes,
                    "ViewModelPropertyViewModel",
                    "viewModelReferenceId",
                    2,
                );
            });
            push_synthetic_object(bytes, "ViewModel", &[("viewModelType", 2)]);
            push_synthetic_object(bytes, "ViewModelPropertyNumber", &[]);
            push_synthetic_object(bytes, "ViewModelPropertyNumber", &[]);
            push_synthetic_object(bytes, "ViewModelPropertyTrigger", &[]);
            push_synthetic_object(bytes, "ViewModelPropertyAssetFont", &[]);
            // A same-shaped global model lets listener tests distinguish an
            // authored slot identity from its compatible cross-model occupant.
            push_synthetic_object(bytes, "ViewModel", &[("viewModelType", 2)]);
            push_synthetic_object(bytes, "ViewModelPropertyNumber", &[]);
            push_synthetic_object(bytes, "ViewModelPropertyNumber", &[]);
            push_synthetic_object(bytes, "ViewModelPropertyTrigger", &[]);
            push_synthetic_object(bytes, "ViewModelPropertyAssetFont", &[]);
            push_synthetic_object(bytes, "Backboard", &[]);
            push_synthetic_object(bytes, "ViewModelInstance", &[("viewModelId", 1)]);
            push_synthetic_object_with_properties(bytes, "ViewModelInstanceNumber", |bytes| {
                push_synthetic_uint_property(
                    bytes,
                    "ViewModelInstanceNumber",
                    "viewModelPropertyId",
                    0,
                );
                push_synthetic_f32_property(bytes, "ViewModelInstanceNumber", "propertyValue", 0.0);
            });
            push_synthetic_object_with_properties(bytes, "ViewModelInstanceTrigger", |bytes| {
                push_synthetic_uint_property(
                    bytes,
                    "ViewModelInstanceTrigger",
                    "viewModelPropertyId",
                    2,
                );
            });
            push_synthetic_object_with_properties(bytes, "ViewModelInstanceAssetFont", |bytes| {
                push_synthetic_uint_property(
                    bytes,
                    "ViewModelInstanceAssetFont",
                    "viewModelPropertyId",
                    3,
                );
                push_synthetic_uint_property(
                    bytes,
                    "ViewModelInstanceAssetFont",
                    "propertyValue",
                    0,
                );
            });
            push_synthetic_object_with_properties(bytes, "ViewModelInstanceNumber", |bytes| {
                push_synthetic_uint_property(
                    bytes,
                    "ViewModelInstanceNumber",
                    "viewModelPropertyId",
                    1,
                );
                push_synthetic_f32_property(bytes, "ViewModelInstanceNumber", "propertyValue", 0.0);
            });
            push_synthetic_object(bytes, "ViewModelInstance", &[("viewModelId", 2)]);
            push_synthetic_object_with_properties(bytes, "ViewModelInstanceNumber", |bytes| {
                push_synthetic_uint_property(
                    bytes,
                    "ViewModelInstanceNumber",
                    "viewModelPropertyId",
                    0,
                );
                push_synthetic_f32_property(bytes, "ViewModelInstanceNumber", "propertyValue", 0.0);
            });
            push_synthetic_object_with_properties(bytes, "ViewModelInstanceNumber", |bytes| {
                push_synthetic_uint_property(
                    bytes,
                    "ViewModelInstanceNumber",
                    "viewModelPropertyId",
                    1,
                );
                push_synthetic_f32_property(bytes, "ViewModelInstanceNumber", "propertyValue", 0.0);
            });
            push_synthetic_object_with_properties(bytes, "ViewModelInstanceTrigger", |bytes| {
                push_synthetic_uint_property(
                    bytes,
                    "ViewModelInstanceTrigger",
                    "viewModelPropertyId",
                    2,
                );
            });
            push_synthetic_object_with_properties(bytes, "ViewModelInstanceAssetFont", |bytes| {
                push_synthetic_uint_property(
                    bytes,
                    "ViewModelInstanceAssetFont",
                    "viewModelPropertyId",
                    3,
                );
            });
            push_synthetic_object(bytes, "Artboard", &[("viewModelId", 1)]);
            push_synthetic_object_with_properties(bytes, "LinearAnimation", |bytes| {
                push_synthetic_uint_property(bytes, "LinearAnimation", "duration", 1);
            });
            push_synthetic_object(bytes, "StateMachine", &[]);

            if listener_action {
                push_synthetic_object(bytes, "StateMachineNumber", &[]);
                let mut listener_path = Vec::new();
                push_var_uint(&mut listener_path, 1);
                push_var_uint(&mut listener_path, 0);
                push_synthetic_object_with_properties(
                    bytes,
                    "StateMachineListenerSingle",
                    |bytes| {
                        push_synthetic_uint_property(
                            bytes,
                            "StateMachineListenerSingle",
                            "targetId",
                            0,
                        );
                        push_synthetic_uint_property(
                            bytes,
                            "StateMachineListenerSingle",
                            "listenerTypeValue",
                            11,
                        );
                        push_synthetic_bytes_property(
                            bytes,
                            "StateMachineListenerSingle",
                            "viewModelPathIds",
                            &listener_path,
                        );
                    },
                );
                push_synthetic_object_with_properties(bytes, "ListenerNumberChange", |bytes| {
                    push_synthetic_uint_property(bytes, "ListenerNumberChange", "inputId", 0);
                    push_synthetic_f32_property(bytes, "ListenerNumberChange", "value", 7.0);
                });
                push_owned_number_change_action(bytes, 0);
                if cross_model_trigger_action {
                    push_owned_trigger_change_action(bytes, 2, 2);
                    push_owned_number_change_action_for(bytes, 2, 1, 64.0, 0);
                }
                if listener_cascade {
                    let mut cascade_path = Vec::new();
                    push_var_uint(&mut cascade_path, 1);
                    push_var_uint(&mut cascade_path, 1);
                    push_synthetic_object_with_properties(
                        bytes,
                        "StateMachineListenerSingle",
                        |bytes| {
                            push_synthetic_uint_property(
                                bytes,
                                "StateMachineListenerSingle",
                                "targetId",
                                0,
                            );
                            push_synthetic_uint_property(
                                bytes,
                                "StateMachineListenerSingle",
                                "listenerTypeValue",
                                11,
                            );
                            push_synthetic_bytes_property(
                                bytes,
                                "StateMachineListenerSingle",
                                "viewModelPathIds",
                                &cascade_path,
                            );
                        },
                    );
                    push_synthetic_object_with_properties(bytes, "ListenerNumberChange", |bytes| {
                        push_synthetic_uint_property(bytes, "ListenerNumberChange", "inputId", 0);
                        push_synthetic_f32_property(bytes, "ListenerNumberChange", "value", 9.0);
                    });
                }
            }

            push_synthetic_object(bytes, "StateMachineLayer", &[]);
            push_synthetic_object(bytes, "AnyState", &[]);
            push_synthetic_object(bytes, "EntryState", &[]);
            push_synthetic_object(bytes, "StateTransition", &[("stateToId", 2)]);
            push_synthetic_object(bytes, "AnimationState", &[("animationId", 0)]);
            if !listener_action {
                const STATE_AT_START: u64 = 2 << 1;
                push_owned_number_change_action(bytes, STATE_AT_START);
                if cross_model_trigger_action {
                    push_owned_trigger_change_action_with_flags(bytes, 2, 2, STATE_AT_START);
                }
            }
            push_owned_font_bind(bytes, unrelated_two_way_bind.then_some(1 << 1));
            push_synthetic_object(bytes, "ExitState", &[]);
        })
    }

    fn synthetic_owned_view_model_listener_chain_riv(
        file_id: u64,
        listener_count: usize,
        close_cycle: bool,
    ) -> Vec<u8> {
        synthetic_riv(file_id, |bytes| {
            push_synthetic_object(bytes, "ViewModel", &[]);
            for _ in 0..=listener_count {
                push_synthetic_object(bytes, "ViewModelPropertyNumber", &[]);
            }
            push_synthetic_object(bytes, "Backboard", &[]);
            push_synthetic_object(bytes, "ViewModelInstance", &[("viewModelId", 0)]);
            for property_index in 0..=listener_count {
                push_synthetic_object_with_properties(bytes, "ViewModelInstanceNumber", |bytes| {
                    push_synthetic_uint_property(
                        bytes,
                        "ViewModelInstanceNumber",
                        "viewModelPropertyId",
                        property_index as u64,
                    );
                    push_synthetic_f32_property(
                        bytes,
                        "ViewModelInstanceNumber",
                        "propertyValue",
                        0.0,
                    );
                });
            }
            push_synthetic_object(bytes, "Artboard", &[("viewModelId", 0)]);
            push_synthetic_object_with_properties(bytes, "LinearAnimation", |bytes| {
                push_synthetic_uint_property(bytes, "LinearAnimation", "duration", 1);
            });
            push_synthetic_object(bytes, "StateMachine", &[]);
            for property_index in 0..listener_count {
                let mut listener_path = Vec::new();
                push_var_uint(&mut listener_path, 0);
                push_var_uint(&mut listener_path, property_index as u64);
                push_synthetic_object_with_properties(
                    bytes,
                    "StateMachineListenerSingle",
                    |bytes| {
                        push_synthetic_uint_property(
                            bytes,
                            "StateMachineListenerSingle",
                            "targetId",
                            0,
                        );
                        push_synthetic_uint_property(
                            bytes,
                            "StateMachineListenerSingle",
                            "listenerTypeValue",
                            11,
                        );
                        push_synthetic_bytes_property(
                            bytes,
                            "StateMachineListenerSingle",
                            "viewModelPathIds",
                            &listener_path,
                        );
                    },
                );
                push_owned_number_change_action_for(
                    bytes,
                    0,
                    property_index.saturating_add(1) as u64,
                    1.0,
                    0,
                );
            }
            if close_cycle {
                let mut listener_path = Vec::new();
                push_var_uint(&mut listener_path, 0);
                push_var_uint(&mut listener_path, listener_count as u64);
                push_synthetic_object_with_properties(
                    bytes,
                    "StateMachineListenerSingle",
                    |bytes| {
                        push_synthetic_uint_property(
                            bytes,
                            "StateMachineListenerSingle",
                            "targetId",
                            0,
                        );
                        push_synthetic_uint_property(
                            bytes,
                            "StateMachineListenerSingle",
                            "listenerTypeValue",
                            11,
                        );
                        push_synthetic_bytes_property(
                            bytes,
                            "StateMachineListenerSingle",
                            "viewModelPathIds",
                            &listener_path,
                        );
                    },
                );
                push_owned_number_change_action_for(bytes, 0, 0, 2.0, 0);
            }
            push_synthetic_object(bytes, "StateMachineLayer", &[]);
            push_synthetic_object(bytes, "AnyState", &[]);
            push_synthetic_object(bytes, "EntryState", &[]);
            push_synthetic_object(bytes, "StateTransition", &[("stateToId", 2)]);
            push_synthetic_object(bytes, "AnimationState", &[("animationId", 0)]);
            push_synthetic_object(bytes, "ExitState", &[]);
        })
    }

    fn synthetic_owned_view_model_listener_live_cycle_riv(file_id: u64) -> Vec<u8> {
        synthetic_riv(file_id, |bytes| {
            push_synthetic_object(bytes, "ViewModel", &[]);
            push_synthetic_object(bytes, "ViewModelPropertyNumber", &[]);
            push_synthetic_object(bytes, "ViewModelPropertyNumber", &[]);
            push_synthetic_object(bytes, "Backboard", &[]);
            push_synthetic_object(bytes, "ViewModelInstance", &[("viewModelId", 0)]);
            for property_index in 0..2 {
                push_synthetic_object_with_properties(bytes, "ViewModelInstanceNumber", |bytes| {
                    push_synthetic_uint_property(
                        bytes,
                        "ViewModelInstanceNumber",
                        "viewModelPropertyId",
                        property_index,
                    );
                    push_synthetic_f32_property(
                        bytes,
                        "ViewModelInstanceNumber",
                        "propertyValue",
                        0.0,
                    );
                });
            }
            push_synthetic_object(bytes, "Artboard", &[("viewModelId", 0)]);
            push_synthetic_object_with_properties(bytes, "LinearAnimation", |bytes| {
                push_synthetic_uint_property(bytes, "LinearAnimation", "duration", 1);
            });
            push_synthetic_object(bytes, "StateMachine", &[]);

            // This listener order forms a permanent three-phase cycle:
            // (A=1, B=1) -> (A=0, B=0) -> (A=1, B=0).
            push_owned_number_listener_change(bytes, 0, 1, 1.0);
            push_owned_number_listener_change(bytes, 1, 0, 0.0);
            push_owned_number_listener_change(bytes, 0, 0, 1.0);
            push_owned_number_listener_change(bytes, 1, 1, 0.0);

            push_synthetic_object(bytes, "StateMachineLayer", &[]);
            push_synthetic_object(bytes, "AnyState", &[]);
            push_synthetic_object(bytes, "EntryState", &[]);
            push_synthetic_object(bytes, "StateTransition", &[("stateToId", 2)]);
            push_synthetic_object(bytes, "AnimationState", &[("animationId", 0)]);
            push_synthetic_object(bytes, "ExitState", &[]);
        })
    }

    fn push_owned_number_listener_change(
        bytes: &mut Vec<u8>,
        source_property_index: u64,
        target_property_index: u64,
        value: f32,
    ) {
        let mut listener_path = Vec::new();
        push_var_uint(&mut listener_path, 0);
        push_var_uint(&mut listener_path, source_property_index);
        push_synthetic_object_with_properties(bytes, "StateMachineListenerSingle", |bytes| {
            push_synthetic_uint_property(bytes, "StateMachineListenerSingle", "targetId", 0);
            push_synthetic_uint_property(
                bytes,
                "StateMachineListenerSingle",
                "listenerTypeValue",
                11,
            );
            push_synthetic_bytes_property(
                bytes,
                "StateMachineListenerSingle",
                "viewModelPathIds",
                &listener_path,
            );
        });
        push_owned_number_change_action_for(bytes, 0, target_property_index, value, 0);
    }

    fn push_owned_number_change_action(bytes: &mut Vec<u8>, flags: u64) {
        push_owned_number_change_action_for(bytes, 1, 1, 42.0, flags);
    }

    fn push_owned_number_change_action_for(
        bytes: &mut Vec<u8>,
        view_model_index: u64,
        property_index: u64,
        value: f32,
        flags: u64,
    ) {
        push_synthetic_object_with_properties(bytes, "BindablePropertyNumber", |bytes| {
            push_synthetic_f32_property(bytes, "BindablePropertyNumber", "propertyValue", value);
        });
        let mut output_path = Vec::new();
        push_var_uint(&mut output_path, view_model_index);
        push_var_uint(&mut output_path, property_index);
        push_synthetic_object_with_properties(bytes, "DataBindContext", |bytes| {
            push_synthetic_uint_property(
                bytes,
                "DataBindContext",
                "propertyKey",
                u64::from(schema_property_key(
                    "BindablePropertyNumber",
                    "propertyValue",
                )),
            );
            push_synthetic_uint_property(bytes, "DataBindContext", "flags", 1);
            push_synthetic_bytes_property(bytes, "DataBindContext", "sourcePathIds", &output_path);
        });
        push_synthetic_object(bytes, "ListenerViewModelChange", &[("flags", flags)]);
    }

    fn push_owned_trigger_change_action(
        bytes: &mut Vec<u8>,
        view_model_index: u64,
        property_index: u64,
    ) {
        push_owned_trigger_change_action_with_flags(bytes, view_model_index, property_index, 0);
    }

    fn push_owned_trigger_change_action_with_flags(
        bytes: &mut Vec<u8>,
        view_model_index: u64,
        property_index: u64,
        flags: u64,
    ) {
        push_synthetic_object(bytes, "BindablePropertyTrigger", &[("propertyValue", 1)]);
        let mut output_path = Vec::new();
        push_var_uint(&mut output_path, view_model_index);
        push_var_uint(&mut output_path, property_index);
        push_synthetic_object_with_properties(bytes, "DataBindContext", |bytes| {
            push_synthetic_uint_property(
                bytes,
                "DataBindContext",
                "propertyKey",
                u64::from(schema_property_key(
                    "BindablePropertyTrigger",
                    "propertyValue",
                )),
            );
            push_synthetic_uint_property(bytes, "DataBindContext", "flags", 1);
            push_synthetic_bytes_property(bytes, "DataBindContext", "sourcePathIds", &output_path);
        });
        push_synthetic_object(bytes, "ListenerViewModelChange", &[("flags", flags)]);
    }

    fn push_owned_font_bind(bytes: &mut Vec<u8>, flags: Option<u64>) {
        push_synthetic_object(
            bytes,
            "BindablePropertyAsset",
            &[("propertyValue", u64::from(u32::MAX))],
        );
        let mut source_path = Vec::new();
        push_var_uint(&mut source_path, 1);
        push_var_uint(&mut source_path, 3);
        push_synthetic_object_with_properties(bytes, "DataBindContext", |bytes| {
            push_synthetic_uint_property(
                bytes,
                "DataBindContext",
                "propertyKey",
                u64::from(schema_property_key(
                    "BindablePropertyAsset",
                    "propertyValue",
                )),
            );
            push_synthetic_bytes_property(bytes, "DataBindContext", "sourcePathIds", &source_path);
            if let Some(flags) = flags {
                push_synthetic_uint_property(bytes, "DataBindContext", "flags", flags);
            }
        });
    }

    fn owned_view_model_action_fixture(
        file_id: u64,
        listener_action: bool,
    ) -> (RuntimeFile, ArtboardInstance, StateMachineInstance) {
        let bytes = synthetic_owned_view_model_action_riv(file_id, listener_action);
        let file = read_runtime_file(&bytes).expect("owned ViewModel action fixture imports");
        let graph = GraphFile::from_runtime_file(&file).expect("fixture builds a graph");
        let artboard_graph = graph.artboards.first().expect("fixture has an artboard");
        let mut artboard = ArtboardInstance::from_graph(&file, artboard_graph)
            .expect("fixture artboard instantiates");
        let runtime_state_machine = artboard.state_machine(0).expect("fixture machine graph");
        assert_eq!(runtime_state_machine.bindable_numbers.len(), 1);
        if listener_action {
            assert_eq!(runtime_state_machine.listeners.len(), 1);
            assert_eq!(
                runtime_state_machine.listeners[0]
                    .view_model_path
                    .as_ref()
                    .and_then(|path| path.absolute_source_path()),
                Some((1, [0].as_slice()))
            );
            assert_eq!(runtime_state_machine.listeners[0].listener_actions.len(), 2);
        }
        let state_machine = artboard
            .state_machine_instance(0)
            .expect("fixture has a state machine");
        (file, artboard, state_machine)
    }

    fn owned_view_model_action_fixture_with_unrelated_two_way_bind(
        file_id: u64,
        listener_action: bool,
    ) -> (RuntimeFile, ArtboardInstance, StateMachineInstance) {
        let bytes = synthetic_owned_view_model_action_riv_with_options(
            file_id,
            listener_action,
            false,
            false,
            true,
        );
        let file = read_runtime_file(&bytes).expect("two-way listener fixture imports");
        let graph = GraphFile::from_runtime_file(&file).expect("fixture builds a graph");
        let artboard_graph = graph.artboards.first().expect("fixture has an artboard");
        let mut artboard = ArtboardInstance::from_graph(&file, artboard_graph)
            .expect("fixture artboard instantiates");
        let state_machine = artboard
            .state_machine_instance(0)
            .expect("fixture has a state machine");
        (file, artboard, state_machine)
    }

    #[test]
    fn component_list_occurrence_ignores_scalar_dirt_but_consumes_structural_rebind() {
        let (file, _, _) = owned_view_model_action_fixture(9713, false);
        let context = RuntimeOwnedViewModelHandle::new(
            RuntimeOwnedViewModelInstance::new(&file, 1).expect("row context"),
        );
        let row = RuntimeComponentListItemInstance {
            child: Box::new(synthetic_instance(Vec::new(), Vec::new())),
            render_resources: RefCell::new(crate::draw::RuntimeOccurrenceRenderResources::default()),
            state_machines: Vec::new(),
            context_rebind_sink: {
                let sink = crate::view_model_cell::RuntimeCellDirtSink::new();
                context.add_rebind_dependent(&sink);
                sink
            },
            draw_index_sink: None,
            context: context.clone(),
            occurrence_identity: 1,
            logical_index: 0,
            settled_layout_size: Cell::new(None),
            transform: Mat2D::IDENTITY,
            render_cache_revision: 1,
        };
        assert!(row.context_is_current(&context));

        assert!(context.borrow_mut().set_number_by_property_path(&[1], 42.0));
        assert!(row.context_is_current(&context));

        row.context_rebind_sink
            .add_dirt(crate::view_model_cell::RuntimeCellDirt::BINDINGS);
        assert!(!row.context_is_current(&context));
        row.consume_context_rebind_dirt();
        assert!(row.context_is_current(&context));
    }

    fn owned_view_model_action_fixture_with_cross_model_trigger(
        file_id: u64,
    ) -> (RuntimeFile, ArtboardInstance, StateMachineInstance) {
        let bytes =
            synthetic_owned_view_model_action_riv_with_options(file_id, true, true, false, false);
        let file = read_runtime_file(&bytes).expect("owned ViewModel action fixture imports");
        let graph = GraphFile::from_runtime_file(&file).expect("fixture builds a graph");
        let artboard_graph = graph.artboards.first().expect("fixture has an artboard");
        let mut artboard = ArtboardInstance::from_graph(&file, artboard_graph)
            .expect("fixture artboard instantiates");
        let runtime_state_machine = artboard.state_machine(0).expect("fixture machine graph");
        assert_eq!(runtime_state_machine.listeners.len(), 1);
        assert_eq!(runtime_state_machine.listeners[0].listener_actions.len(), 4);
        let state_machine = artboard
            .state_machine_instance(0)
            .expect("fixture has a state machine");
        (file, artboard, state_machine)
    }

    fn owned_view_model_listener_cascade_fixture(
        file_id: u64,
    ) -> (RuntimeFile, ArtboardInstance, StateMachineInstance) {
        let bytes =
            synthetic_owned_view_model_action_riv_with_options(file_id, true, false, true, false);
        let file = read_runtime_file(&bytes).expect("listener cascade fixture imports");
        let graph = GraphFile::from_runtime_file(&file).expect("fixture builds a graph");
        let artboard_graph = graph.artboards.first().expect("fixture has an artboard");
        let mut artboard = ArtboardInstance::from_graph(&file, artboard_graph)
            .expect("fixture artboard instantiates");
        let state_machine = artboard
            .state_machine_instance(0)
            .expect("fixture has a state machine");
        (file, artboard, state_machine)
    }

    #[test]
    fn immutable_owned_view_model_bind_dispatches_without_mutating_the_borrowed_context() {
        let (file, mut artboard, mut state_machine) = owned_view_model_action_fixture(9680, true);
        let mut context = RuntimeOwnedViewModelInstance::new(&file, 1)
            .expect("fixture has an owned ViewModel context");

        assert!(state_machine.bind_owned_view_model_context(&context));
        assert!(context.set_number_by_property_index(0, 1.0));
        state_machine.bind_owned_view_model_context(&context);
        assert_eq!(
            state_machine
                .input(0)
                .and_then(|input| input.number_value()),
            Some(0.0),
            "rebinding must not execute a queued listener action"
        );
        artboard.advance_state_machine_instance(&mut state_machine, 0.0);
        assert_eq!(
            state_machine
                .input(0)
                .and_then(|input| input.number_value()),
            Some(7.0),
            "the documented immutable low-level bind still dispatches non-context listener actions"
        );
        assert_eq!(context.number_value_by_property_path(&[1]), Some(0.0));

        assert!(context.set_number_by_property_index(0, 2.0));
        state_machine.bind_owned_view_model_context_mut(&mut context);
        assert_eq!(context.number_value_by_property_path(&[1]), Some(0.0));
        artboard.advance_state_machine_instances_with_nested_and_owned_view_model_context(
            std::slice::from_mut(&mut state_machine),
            0.0,
            &mut context,
        );
        assert_eq!(context.number_value_by_property_path(&[1]), Some(42.0));
    }

    #[test]
    fn compatibility_context_chain_listener_dispatches_on_the_next_frame() {
        let (file, mut artboard, mut state_machine) = owned_view_model_action_fixture(9682, true);
        let mut root = RuntimeOwnedViewModelInstance::new(&file, 0)
            .expect("fixture has a nested owned ViewModel context");

        assert!(state_machine.bind_owned_view_model_context_chain(&file, &root, &[&[1]]));
        assert!(root.set_number_by_property_path(&[1, 0], 1.0));
        state_machine.bind_owned_view_model_context_chain(&file, &root, &[&[1]]);
        assert_eq!(
            state_machine
                .input(0)
                .and_then(|input| input.number_value()),
            Some(0.0),
            "context-chain rebinding must only retain the listener cell"
        );

        // C++ reports cell dirt immediately but drains listener actions at
        // the next new-frame `applyEvents`, before layer advance
        // (`state_machine_instance.cpp:1374-1380,2320-2335,2555-2565`).
        artboard.advance_state_machine_instance(&mut state_machine, 0.0);
        assert_eq!(
            state_machine
                .input(0)
                .and_then(|input| input.number_value()),
            Some(7.0)
        );
        assert_eq!(
            root.number_value_by_property_path(&[1, 1]),
            Some(0.0),
            "an immutable context-chain bind cannot receive ViewModel writes"
        );
    }

    #[test]
    fn compatibility_context_chain_converter_retains_the_nested_operand_cell() {
        let (file, _, _) = owned_view_model_action_fixture(9683, true);
        let mut root = RuntimeOwnedViewModelInstance::new(&file, 0)
            .expect("fixture has a nested owned ViewModel context");
        assert!(root.set_number_by_property_path(&[1, 0], 4.0));
        let mut converter = RuntimeDataBindGraphConverter::OperationViewModel {
            operation_type: 2,
            operation_value: 0.0,
            default_operation_value: 0.0,
            source_path: Some(vec![1, 0]),
            retained_operation_value: None,
        };

        assert!(
            crate::data_bind_graph::runtime_data_bind_graph_refresh_operation_view_model_converter_for_owned_context(
                &mut converter,
                &root,
                &[&[1]],
            )
        );
        let RuntimeDataBindGraphConverter::OperationViewModel {
            retained_operation_value: Some(retained),
            ..
        } = &converter
        else {
            panic!("nested operation operand must retain its exact cell")
        };
        assert!(retained.ptr_eq(&root.cell_by_property_path(&[1, 0]).expect("nested cell")));
        assert_eq!(
            crate::data_bind_graph::runtime_data_bind_graph_convert_value(
                &converter,
                &RuntimeDataBindGraphValue::Number(3.0),
            ),
            Some(RuntimeDataBindGraphValue::Number(12.0))
        );
    }

    #[test]
    fn compatibility_mutable_listener_cascade_drains_in_one_apply_events_frame() {
        let (file, mut artboard, mut state_machine) =
            owned_view_model_listener_cascade_fixture(9689);
        let mut context = RuntimeOwnedViewModelInstance::new(&file, 1)
            .expect("fixture has an owned ViewModel context");

        assert!(state_machine.bind_owned_view_model_context_mut(&mut context));
        assert!(context.set_number_by_property_index(0, 1.0));
        state_machine.bind_owned_view_model_context_mut(&mut context);

        // C++ applies the report present at new-frame start, then loops the
        // ViewModel write's chained report to completion before layer advance
        // (`state_machine_instance.cpp:2320-2343,2555-2565`).
        artboard.advance_state_machine_instances_with_nested_and_owned_view_model_context(
            std::slice::from_mut(&mut state_machine),
            0.0,
            &mut context,
        );
        assert_eq!(
            state_machine
                .input(0)
                .and_then(|input| input.number_value()),
            Some(9.0),
            "the chained listener must finish in the same applyEvents frame",
        );
    }

    #[test]
    fn composite_owned_view_model_bind_dispatches_view_model_listeners() {
        let (file, mut artboard, mut state_machine) = owned_view_model_action_fixture(9684, true);
        let main = RuntimeOwnedViewModelHandle::new(
            RuntimeOwnedViewModelInstance::new(&file, 1)
                .expect("fixture has an owned ViewModel context"),
        );
        let context = RuntimeOwnedViewModelContext::from_main_handle(main.clone());

        assert!(state_machine.bind_owned_view_model_contexts(&context));
        assert!(main.borrow_mut().set_number_by_property_index(0, 1.0));
        artboard.advance_state_machine_instance(&mut state_machine, 0.0);
        assert_eq!(
            state_machine
                .input(0)
                .and_then(|input| input.number_value()),
            Some(7.0),
            "the composite artboard context must dispatch its ViewModel listener"
        );
        assert_eq!(
            main.borrow().number_value_by_property_path(&[1]),
            Some(42.0),
            "listener ViewModel writes must reach the retained composite main context"
        );
    }

    #[test]
    fn composite_listener_cascade_drains_in_one_apply_events_frame() {
        let (file, mut artboard, mut state_machine) =
            owned_view_model_listener_cascade_fixture(9700);
        let main = RuntimeOwnedViewModelHandle::new(
            RuntimeOwnedViewModelInstance::new(&file, 1)
                .expect("fixture has an owned ViewModel context"),
        );
        let context = RuntimeOwnedViewModelContext::from_main_handle(main.clone());

        assert!(state_machine.bind_owned_view_model_contexts(&context));
        assert!(main.borrow_mut().set_number_by_property_index(0, 1.0));
        artboard.advance_state_machine_instance(&mut state_machine, 0.0);
        assert_eq!(
            state_machine
                .input(0)
                .and_then(|input| input.number_value()),
            Some(9.0),
            "applyEvents must drain listener A then its chained listener B before layer advance",
        );
    }

    #[test]
    fn retained_handle_listener_cascade_drains_in_one_apply_events_frame() {
        let (file, mut artboard, mut state_machine) =
            owned_view_model_listener_cascade_fixture(9701);
        let context = RuntimeOwnedViewModelHandle::new(
            RuntimeOwnedViewModelInstance::new(&file, 1)
                .expect("fixture has an owned ViewModel context"),
        );

        assert!(state_machine.bind_owned_view_model_handle(&context));
        assert!(context.borrow_mut().set_number_by_property_index(0, 1.0));
        artboard.advance_state_machine_instance(&mut state_machine, 0.0);
        assert_eq!(
            state_machine
                .input(0)
                .and_then(|input| input.number_value()),
            Some(9.0),
            "retained listener reports must preserve the chained FIFO order",
        );
    }

    #[test]
    fn retained_data_context_listener_cascade_drains_in_one_apply_events_frame() {
        let (file, mut artboard, mut state_machine) =
            owned_view_model_listener_cascade_fixture(9702);
        let context = RuntimeOwnedViewModelHandle::new(
            RuntimeOwnedViewModelInstance::new(&file, 1)
                .expect("fixture has an owned ViewModel context"),
        );
        assert!(state_machine.bind_owned_view_model_handle(&context));
        assert!(context.borrow_mut().set_number_by_property_index(0, 1.0));
        artboard.advance_state_machine_instance(&mut state_machine, 0.0);
        assert_eq!(
            state_machine
                .input(0)
                .and_then(|input| input.number_value()),
            Some(9.0),
            "the retained DataContext must drain the same applyEvents queue",
        );
    }

    #[test]
    fn retained_state_action_does_not_rebind_an_unrelated_two_way_data_bind() {
        let (file, mut artboard, mut state_machine) =
            owned_view_model_action_fixture_with_unrelated_two_way_bind(9730, false);
        let context = RuntimeOwnedViewModelHandle::new(
            RuntimeOwnedViewModelInstance::new(&file, 1)
                .expect("fixture has an owned ViewModel context"),
        );

        assert!(state_machine.bind_owned_view_model_handle(&context));
        let bind_count = state_machine.owned_data_bind_context_bind_count();
        assert!(artboard.advance_state_machine_instance(&mut state_machine, 0.0));
        assert_eq!(
            context.borrow().number_value_by_property_path(&[1]),
            Some(42.0),
            "the state-entry ListenerViewModelChange must still reach its exact source",
        );
        assert_eq!(
            state_machine.owned_data_bind_context_bind_count(),
            bind_count,
            "an exact listener write must not reconcile the fixture's unrelated two-way bind",
        );
    }

    #[test]
    fn retained_listener_report_does_not_rebind_an_unrelated_two_way_data_bind() {
        let (file, mut artboard, mut state_machine) =
            owned_view_model_action_fixture_with_unrelated_two_way_bind(9731, true);
        let context = RuntimeOwnedViewModelHandle::new(
            RuntimeOwnedViewModelInstance::new(&file, 1)
                .expect("fixture has an owned ViewModel context"),
        );

        assert!(state_machine.bind_owned_view_model_handle(&context));
        let bind_count = state_machine.owned_data_bind_context_bind_count();
        assert!(context.borrow_mut().set_number_by_property_index(0, 1.0));
        assert!(artboard.advance_state_machine_instance(&mut state_machine, 0.0));
        assert_eq!(
            context.borrow().number_value_by_property_path(&[1]),
            Some(42.0),
            "the queued ListenerViewModelChange must still reach its exact source",
        );
        assert_eq!(
            state_machine.owned_data_bind_context_bind_count(),
            bind_count,
            "a queued exact listener write must not reconcile the fixture's unrelated two-way bind",
        );
    }

    #[test]
    fn retained_data_context_listener_reads_the_live_bindable_occurrence() {
        let (file, mut artboard, mut state_machine) = owned_view_model_action_fixture(9732, true);
        let context = RuntimeOwnedViewModelHandle::new(
            RuntimeOwnedViewModelInstance::new(&file, 1)
                .expect("fixture has an owned ViewModel context"),
        );

        assert!(state_machine.bind_owned_view_model_handle(&context));
        assert!(state_machine.set_bindable_number_for_data_bind(0, 9.0));
        assert!(context.borrow_mut().set_number_by_property_index(0, 1.0));
        assert!(artboard.advance_state_machine_instance(&mut state_machine, 0.0));
        assert_eq!(
            context.borrow().number_value_by_property_path(&[1]),
            Some(9.0),
            "ListenerViewModelChange must read the mutable cloned BindableProperty at perform time, not its imported default",
        );
    }

    #[test]
    fn retained_data_context_listener_queues_each_mutation_until_next_frame() {
        let (file, mut artboard, mut state_machine) = owned_view_model_action_fixture(9720, true);
        let context = RuntimeOwnedViewModelHandle::new(
            RuntimeOwnedViewModelInstance::new(&file, 1)
                .expect("fixture has an owned ViewModel context"),
        );

        // Binding the DataContext registers the listener condition as a
        // dependent on the retained cell it reads (#RB-1 e4).
        assert!(state_machine.bind_owned_view_model_handle(&context));
        let condition_cell = context
            .borrow()
            .cell_by_property_path(&[0])
            .expect("condition property has a retained cell");
        let bound_cell = state_machine
            .view_model_listener_condition_cell(0)
            .expect("DataContext bind migrates the scalar condition");
        assert!(
            bound_cell.ptr_eq(&condition_cell),
            "the listener must observe the SAME retained cell the context owns"
        );

        // A slot write reports the listener immediately, but C++ performs its
        // actions only from next-frame applyEvents
        // (`state_machine_instance.cpp:2320-2335,3021-3025`).
        assert!(context.borrow_mut().set_number_by_property_index(0, 1.0));
        assert_eq!(
            state_machine.pending_listener_view_model_report_count(),
            1,
            "the cell cascade must append one listener report"
        );
        state_machine.bind_owned_view_model_handle(&context);
        assert_eq!(
            state_machine
                .input(0)
                .and_then(|input| input.number_value()),
            Some(0.0),
            "rebinding must not execute a queued listener action"
        );
        artboard.advance_state_machine_instance(&mut state_machine, 0.0);
        assert_eq!(
            state_machine
                .input(0)
                .and_then(|input| input.number_value()),
            Some(7.0),
            "next-frame applyEvents must dispatch the listener actions"
        );
        assert_eq!(
            context.borrow().number_value_by_property_path(&[1]),
            Some(42.0)
        );

        // C++ deliberately preserves duplicates instead of collapsing a
        // transient 1→2→1 into a net-equal observed copy.
        assert!(context.borrow_mut().set_number_by_property_path(&[1], 0.0));
        assert!(context.borrow_mut().set_number_by_property_index(0, 2.0));
        assert!(context.borrow_mut().set_number_by_property_index(0, 1.0));
        assert_eq!(
            state_machine.pending_listener_view_model_report_count(),
            2,
            "both genuine mutations must remain queued in order"
        );
        artboard.advance_state_machine_instance(&mut state_machine, 0.0);
        assert_eq!(state_machine.pending_listener_view_model_report_count(), 0);
        assert_eq!(
            context.borrow().number_value_by_property_path(&[1]),
            Some(42.0),
            "the transient reports must execute instead of disappearing behind a net diff"
        );
    }

    #[test]
    fn retained_data_context_listener_apply_events_cap_leaves_batch_101_pending() {
        const LISTENER_CAP: usize = 100;
        let bytes = synthetic_owned_view_model_listener_chain_riv(9705, LISTENER_CAP + 1, false);
        let file = read_runtime_file(&bytes).expect("listener boundary fixture imports");
        let graph = GraphFile::from_runtime_file(&file).expect("fixture builds a graph");
        let artboard_graph = graph.artboards.first().expect("fixture has an artboard");
        let mut artboard = ArtboardInstance::from_graph(&file, artboard_graph)
            .expect("fixture artboard instantiates");
        let mut state_machine = artboard
            .state_machine_instance(0)
            .expect("fixture has a state machine");
        let context = RuntimeOwnedViewModelHandle::new(
            RuntimeOwnedViewModelInstance::new(&file, 0)
                .expect("fixture has an owned ViewModel context"),
        );
        let data_context = RuntimeOwnedDataContext::from_root_handle(context.clone());

        assert!(state_machine.bind_owned_view_model_data_context(&data_context));
        assert!(context.borrow_mut().set_number_by_property_index(0, 1.0));
        artboard.advance_state_machine_instance(&mut state_machine, 0.0);
        assert_eq!(
            context
                .borrow()
                .number_value_by_property_index(LISTENER_CAP),
            Some(1.0),
        );
        assert_eq!(
            context
                .borrow()
                .number_value_by_property_index(LISTENER_CAP + 1),
            Some(0.0),
            "the applyEvents batch cap must stop before listener 101",
        );
        assert_eq!(state_machine.pending_listener_view_model_report_count(), 1);

        artboard.advance_state_machine_instance(&mut state_machine, 0.0);
        assert_eq!(
            context
                .borrow()
                .number_value_by_property_index(LISTENER_CAP + 1),
            Some(1.0),
        );
        assert_eq!(state_machine.pending_listener_view_model_report_count(), 0);
    }

    #[test]
    fn retained_data_context_listener_cycle_settles_without_replaying() {
        let bytes = synthetic_owned_view_model_listener_chain_riv(9706, 2, true);
        let file = read_runtime_file(&bytes).expect("listener cycle fixture imports");
        let graph = GraphFile::from_runtime_file(&file).expect("fixture builds a graph");
        let artboard_graph = graph.artboards.first().expect("fixture has an artboard");
        let mut artboard = ArtboardInstance::from_graph(&file, artboard_graph)
            .expect("fixture artboard instantiates");
        let mut state_machine = artboard
            .state_machine_instance(0)
            .expect("fixture has a state machine");
        let context = RuntimeOwnedViewModelHandle::new(
            RuntimeOwnedViewModelInstance::new(&file, 0)
                .expect("fixture has an owned ViewModel context"),
        );
        assert!(state_machine.bind_owned_view_model_handle(&context));
        assert!(context.borrow_mut().set_number_by_property_index(0, 1.0));
        assert!(artboard.advance_state_machine_instance(&mut state_machine, 0.0));
        assert_eq!(
            context.borrow().number_value_by_property_index(0),
            Some(2.0)
        );
        assert_eq!(
            context.borrow().number_value_by_property_index(1),
            Some(1.0)
        );
        assert_eq!(
            context.borrow().number_value_by_property_index(2),
            Some(1.0)
        );
        assert_eq!(state_machine.pending_listener_view_model_report_count(), 0);
    }

    #[test]
    fn retained_data_context_listener_live_cycle_stays_pending_at_apply_events_cap() {
        let bytes = synthetic_owned_view_model_listener_live_cycle_riv(9707);
        let file = read_runtime_file(&bytes).expect("listener live-cycle fixture imports");
        let graph = GraphFile::from_runtime_file(&file).expect("fixture builds a graph");
        let artboard_graph = graph.artboards.first().expect("fixture has an artboard");
        let mut artboard = ArtboardInstance::from_graph(&file, artboard_graph)
            .expect("fixture artboard instantiates");
        let mut state_machine = artboard
            .state_machine_instance(0)
            .expect("fixture has a state machine");
        let context = RuntimeOwnedViewModelHandle::new(
            RuntimeOwnedViewModelInstance::new(&file, 0)
                .expect("fixture has an owned ViewModel context"),
        );

        assert!(state_machine.bind_owned_view_model_handle(&context));
        assert!(context.borrow_mut().set_number_by_property_index(0, 1.0));

        artboard.advance_state_machine_instance(&mut state_machine, 0.0);
        assert!(state_machine.has_pending_listener_view_model_reports());

        artboard.advance_state_machine_instance(&mut state_machine, 0.0);
        assert!(state_machine.has_pending_listener_view_model_reports());
    }

    #[test]
    fn retained_scoped_context_refresh_dispatches_listener_actions_to_the_scope() {
        let (file, mut artboard, mut state_machine) = owned_view_model_action_fixture(9688, true);
        let root = RuntimeOwnedViewModelHandle::new(
            RuntimeOwnedViewModelInstance::new(&file, 0)
                .expect("fixture has a nested owned ViewModel context"),
        );
        let scoped = RuntimeOwnedViewModelContextHandle::root(&file, root.clone())
            .scoped(vec![1])
            .expect("fixture child scope resolves");

        assert!(state_machine.bind_owned_view_model_context_handle(&scoped));
        assert!(root.borrow_mut().set_number_by_property_path(&[1, 0], 1.0));
        artboard.advance_state_machine_instance(&mut state_machine, 0.0);
        assert_eq!(
            state_machine
                .input(0)
                .and_then(|input| input.number_value()),
            Some(7.0),
            "automatic retained-handle refresh must observe the scoped listener source"
        );
        assert_eq!(
            root.borrow().number_value_by_property_path(&[1, 1]),
            Some(42.0),
            "automatic retained-handle refresh must route listener writes back into the scope"
        );
    }

    #[test]
    fn composite_listener_preserves_authored_view_model_identity() {
        let (file, mut artboard, mut state_machine) = owned_view_model_action_fixture(9689, true);
        let same_shaped_main = RuntimeOwnedViewModelHandle::new(
            RuntimeOwnedViewModelInstance::new(&file, 2)
                .expect("fixture has a same-shaped non-global ViewModel"),
        );
        let authored_global = RuntimeOwnedViewModelHandle::new(
            RuntimeOwnedViewModelInstance::new(&file, 1)
                .expect("fixture has the listener's authored global ViewModel"),
        );
        let mut context = RuntimeOwnedViewModelContext::from_main_handle(same_shaped_main.clone());
        assert!(context.set_global_slot_handle(&file, 1, authored_global.clone()));
        assert!(
            same_shaped_main
                .borrow_mut()
                .set_trigger_by_property_index(2, 9)
        );
        assert!(
            authored_global
                .borrow_mut()
                .set_trigger_by_property_index(2, 3)
        );

        assert!(state_machine.bind_owned_view_model_contexts(&context));
        assert!(
            authored_global
                .borrow_mut()
                .set_number_by_property_index(0, 1.0)
        );
        artboard.advance_state_machine_instance(&mut state_machine, 0.0);
        assert_eq!(
            state_machine
                .input(0)
                .and_then(|input| input.number_value()),
            Some(7.0),
            "a same-shaped main model must not mask a listener authored against global slot 1"
        );
        assert_eq!(
            authored_global.borrow().number_value_by_property_path(&[1]),
            Some(42.0)
        );
        assert_eq!(
            same_shaped_main
                .borrow()
                .number_value_by_property_path(&[1]),
            Some(0.0),
            "listener observation and writes must retain their authored ViewModel identity"
        );
    }

    #[test]
    fn listener_write_rejects_cross_model_global_slot_occupant() {
        let (file, mut artboard, mut state_machine) = owned_view_model_action_fixture(9690, true);
        let main = RuntimeOwnedViewModelHandle::new(
            RuntimeOwnedViewModelInstance::new(&file, 0)
                .expect("fixture has a main ViewModel context"),
        );
        let override_instance = RuntimeOwnedViewModelHandle::new(
            RuntimeOwnedViewModelInstance::new(&file, 2)
                .expect("fixture has a compatible cross-model override"),
        );
        let mut context = RuntimeOwnedViewModelContext::from_main_handle(main);
        assert!(context.set_global_slot_handle(&file, 1, override_instance.clone()));
        assert!(
            override_instance
                .borrow_mut()
                .set_trigger_by_property_index(2, 3)
        );
        assert!(
            override_instance
                .borrow_mut()
                .set_font_asset_index_by_property_index(3, 7)
        );

        assert!(state_machine.bind_owned_view_model_contexts(&context));
        assert_eq!(
            state_machine.bindable_asset_value_for_data_bind(1),
            Some(RuntimeFontAssetValue::MISSING_FILE_ASSET_INDEX),
            "font synchronization must reject the same wrong-model occupant as the data-bind graph"
        );
        assert!(
            override_instance
                .borrow_mut()
                .set_number_by_property_index(0, 1.0)
        );
        artboard.advance_state_machine_instance(&mut state_machine, 0.0);
        assert_eq!(
            state_machine
                .input(0)
                .and_then(|input| input.number_value()),
            Some(0.0),
            "C++ DataContext rejects the wrong-model occupant before listener dispatch (data_context.cpp:397-506)"
        );
        assert_eq!(
            state_machine.default_view_model_number_source_value_for_data_bind(0),
            Some(0.0),
            "the authored source remains unresolved against a wrong-model slot occupant"
        );
        assert_eq!(
            override_instance
                .borrow()
                .number_value_by_property_path(&[1]),
            Some(0.0),
            "listener writes must not be redirected through the slot key"
        );
    }

    #[test]
    fn cross_model_listener_trigger_does_not_fire_default_view_model_transition_trigger() {
        let (file, mut artboard, mut state_machine) =
            owned_view_model_action_fixture_with_cross_model_trigger(9694);
        let main = RuntimeOwnedViewModelHandle::new(
            RuntimeOwnedViewModelInstance::new(&file, 1)
                .expect("fixture has the listener's default ViewModel"),
        );
        let cross_model = RuntimeOwnedViewModelHandle::new(
            RuntimeOwnedViewModelInstance::new(&file, 2)
                .expect("fixture has the compatible cross-model trigger target"),
        );
        let mut context = RuntimeOwnedViewModelContext::from_main_handle(main.clone());
        assert!(context.set_global_slot_handle(&file, 2, cross_model.clone()));

        assert!(state_machine.bind_owned_view_model_contexts(&context));
        assert_eq!(
            state_machine.default_view_model_trigger_source_value_for_data_bind(1),
            Some(0),
            "cross-model trigger source must be represented in the bound graph"
        );
        assert_eq!(
            state_machine.bindable_trigger_value_for_data_bind(1),
            Some(1),
            "listener trigger bindable retains its authored action value"
        );
        assert!(main.borrow_mut().set_number_by_property_index(0, 1.0));
        artboard.advance_state_machine_instance(&mut state_machine, 0.0);
        assert_eq!(
            cross_model.borrow().trigger_value_by_property_path(&[2]),
            Some(1),
            "the listener action must still fire its declared cross-model trigger target"
        );
        assert_eq!(
            cross_model.borrow().number_value_by_property_path(&[1]),
            Some(64.0),
            "a non-default global number action must retain its schema-backed source and reach the declared slot"
        );
    }

    #[test]
    fn switching_from_retained_handle_to_composite_clears_stale_refresh_source() {
        let (file, _artboard, mut state_machine) = owned_view_model_action_fixture(9696, false);
        let stale = RuntimeOwnedViewModelHandle::new(
            RuntimeOwnedViewModelInstance::new(&file, 1)
                .expect("fixture has the original retained ViewModel"),
        );
        let replacement = RuntimeOwnedViewModelHandle::new(
            RuntimeOwnedViewModelInstance::new(&file, 1)
                .expect("fixture has the replacement composite ViewModel"),
        );
        assert!(stale.borrow_mut().set_number_by_property_index(1, 5.0));
        assert!(
            replacement
                .borrow_mut()
                .set_number_by_property_index(1, 8.0)
        );

        assert!(state_machine.bind_owned_view_model_handle(&stale));
        let contexts = RuntimeOwnedViewModelContext::from_main_handle(replacement);
        assert!(state_machine.bind_owned_view_model_contexts(&contexts));
        assert_eq!(
            state_machine.default_view_model_number_source_value_for_data_bind(0),
            Some(8.0)
        );

        assert!(stale.borrow_mut().set_number_by_property_index(1, 9.0));
        let _ = state_machine.advance_data_context();
        assert_eq!(
            state_machine.default_view_model_number_source_value_for_data_bind(0),
            Some(8.0),
            "advance_data_context must not resurrect the previously retained single handle after a composite bind"
        );
    }

    #[test]
    fn retained_composite_does_not_route_authored_path_through_cross_model_slot() {
        let (file, mut artboard, state_machine) = owned_view_model_action_fixture(9697, false);
        let main = RuntimeOwnedViewModelHandle::new(
            RuntimeOwnedViewModelInstance::new(&file, 0)
                .expect("fixture has a distinct main ViewModel"),
        );
        let global_override = RuntimeOwnedViewModelHandle::new(
            RuntimeOwnedViewModelInstance::new(&file, 2)
                .expect("fixture has a compatible cross-model global override"),
        );
        let mut contexts = RuntimeOwnedViewModelContext::from_main_handle(main);
        assert!(contexts.set_global_slot_handle(&file, 1, global_override.clone()));
        let mut state_machines = vec![state_machine];

        assert!(state_machines[0].bind_owned_view_model_contexts(&contexts));
        assert!(artboard.advance_state_machine_instances_with_nested(&mut state_machines, 0.0));
        assert_eq!(
            global_override.borrow().number_value_by_property_path(&[1]),
            Some(0.0),
            "slot keys only place globals; C++ lookup compares the actual occupant viewModelId (data_context.cpp:397-506)"
        );

        assert!(
            global_override
                .borrow_mut()
                .set_number_by_property_index(1, 17.0)
        );
        let _ = artboard.advance_state_machine_instances_with_nested(&mut state_machines, 0.0);
        assert_eq!(
            state_machines[0].default_view_model_number_source_value_for_data_bind(0),
            Some(0.0),
            "the unresolved authored source must remain at its default"
        );
        assert_eq!(
            global_override.borrow().number_value_by_property_path(&[1]),
            Some(17.0),
            "the one-shot state action must not replay on the unchanged second advance"
        );
    }

    #[test]
    fn retained_state_action_rejects_slot_without_matching_actual_model() {
        let bytes =
            synthetic_owned_view_model_action_riv_with_options(9704, false, true, false, false);
        let file = read_runtime_file(&bytes).expect("state trigger action fixture imports");
        let graph = GraphFile::from_runtime_file(&file).expect("fixture builds a graph");
        let artboard_graph = graph.artboards.first().expect("fixture has an artboard");
        let mut artboard = ArtboardInstance::from_graph(&file, artboard_graph)
            .expect("fixture artboard instantiates");
        let state_machine = artboard
            .state_machine_instance(0)
            .expect("fixture has a state machine");
        let main = RuntimeOwnedViewModelHandle::new(
            RuntimeOwnedViewModelInstance::new(&file, 1)
                .expect("fixture has the default ViewModel"),
        );
        let slot_two_occupant = RuntimeOwnedViewModelHandle::new(
            RuntimeOwnedViewModelInstance::new(&file, 1)
                .expect("fixture has a same-type global-slot occupant"),
        );
        let mut context = RuntimeOwnedViewModelContext::from_main_handle(main.clone());
        assert!(context.set_global_slot_handle(&file, 2, slot_two_occupant.clone()));
        let mut state_machines = vec![state_machine];

        assert!(state_machines[0].bind_owned_view_model_contexts(&context));
        assert!(artboard.advance_state_machine_instances_with_nested(&mut state_machines, 0.0));
        assert_eq!(
            slot_two_occupant
                .borrow()
                .trigger_value_by_property_path(&[2]),
            Some(0),
            "same-model locals are resolved in DataContext order, not by slot key (data_context.cpp:397-506)",
        );
        assert_eq!(main.borrow().trigger_value_by_property_path(&[2]), Some(0));
    }

    #[test]
    fn artboard_created_machine_rejects_cross_model_global_occupant() {
        let (file, mut artboard, _) = owned_view_model_action_fixture(9698, false);
        let main = RuntimeOwnedViewModelHandle::new(
            RuntimeOwnedViewModelInstance::new(&file, 0)
                .expect("fixture has a distinct main ViewModel"),
        );
        let global_override = RuntimeOwnedViewModelHandle::new(
            RuntimeOwnedViewModelInstance::new(&file, 2)
                .expect("fixture has a compatible cross-model global override"),
        );
        let mut contexts = RuntimeOwnedViewModelContext::from_main_handle(main);
        assert!(contexts.set_global_slot_handle(&file, 1, global_override.clone()));
        let _ = artboard.bind_owned_view_model_artboard_contexts(&file, &contexts);
        let mut state_machine = artboard
            .state_machine_instance(0)
            .expect("bound artboard creates its state machine");

        assert!(artboard.advance_state_machine_instance(&mut state_machine, 0.0));
        assert_eq!(
            global_override.borrow().number_value_by_property_path(&[1]),
            Some(0.0),
            "artboard-created machines inherit actual-id DataContext resolution (data_context.cpp:397-506)",
        );

        assert!(
            global_override
                .borrow_mut()
                .set_number_by_property_index(1, 17.0)
        );
        let _ = artboard.advance_state_machine_instance(&mut state_machine, 0.0);
        assert_eq!(
            state_machine.default_view_model_number_source_value_for_data_bind(0),
            Some(0.0),
            "the wrong-model global occupant must remain unresolved",
        );
        assert_eq!(
            global_override.borrow().number_value_by_property_path(&[1]),
            Some(17.0),
            "the one-shot state action must not replay during alias refresh",
        );
    }

    #[test]
    fn retained_scoped_context_routes_state_entry_action_into_scope() {
        let (file, mut artboard, mut state_machine) = owned_view_model_action_fixture(9699, false);
        let root = RuntimeOwnedViewModelHandle::new(
            RuntimeOwnedViewModelInstance::new(&file, 0)
                .expect("fixture has a nested owned ViewModel context"),
        );
        let scoped = RuntimeOwnedViewModelContextHandle::root(&file, root.clone())
            .scoped(vec![1])
            .expect("fixture child scope resolves");

        assert!(state_machine.bind_owned_view_model_context_handle(&scoped));
        assert!(artboard.advance_state_machine_instance(&mut state_machine, 0.0));
        assert_eq!(
            root.borrow().number_value_by_property_path(&[1, 1]),
            Some(42.0),
            "scheduled state actions must resolve through the retained scope path",
        );
        assert_eq!(
            root.borrow().number_value_by_property_path(&[1]),
            None,
            "the scoped write must not be redirected to the root object",
        );
    }

    #[test]
    fn scoped_data_context_bind_dispatches_view_model_listeners() {
        let (file, mut artboard, mut state_machine) = owned_view_model_action_fixture(9685, true);
        let main = RuntimeOwnedViewModelHandle::new(
            RuntimeOwnedViewModelInstance::new(&file, 0)
                .expect("fixture has a nested owned ViewModel context"),
        );
        let scoped = RuntimeOwnedViewModelContextHandle::root(&file, main.clone())
            .scoped(vec![1])
            .expect("fixture child scope resolves");
        let data_context = RuntimeOwnedDataContext::from_context_handle(&scoped);

        assert!(state_machine.bind_owned_view_model_data_context(&data_context));
        assert!(main.borrow_mut().set_number_by_property_path(&[1, 0], 1.0));
        artboard.advance_state_machine_instance(&mut state_machine, 0.0);
        assert_eq!(
            state_machine
                .input(0)
                .and_then(|input| input.number_value()),
            Some(7.0),
            "scoped DataContexts must dispatch their ViewModel listener"
        );
        assert_eq!(
            main.borrow().number_value_by_property_path(&[1, 1]),
            Some(42.0),
            "listener ViewModel writes must reach the retained scoped path"
        );
    }

    #[test]
    fn later_local_data_context_instance_owns_listener_observation_and_writes() {
        let (file, mut artboard, mut state_machine) = owned_view_model_action_fixture(9686, true);
        let root = RuntimeOwnedViewModelHandle::new(
            RuntimeOwnedViewModelInstance::new(&file, 0)
                .expect("fixture has nested DataContext instances"),
        );
        let invalid_first = RuntimeOwnedViewModelContextHandle::root(&file, root.clone())
            .scoped(vec![2])
            .expect("fixture has a same-shaped wrong-model child");
        let resolved_later = RuntimeOwnedViewModelContextHandle::root(&file, root.clone())
            .scoped(vec![1])
            .expect("fixture has the matching child");
        let data_context = RuntimeOwnedDataContext::with_local_context_handles(
            [invalid_first, resolved_later],
            None,
        );

        assert!(state_machine.bind_owned_view_model_data_context(&data_context));
        assert!(root.borrow_mut().set_number_by_property_path(&[1, 0], 1.0));
        artboard.advance_state_machine_instance(&mut state_machine, 0.0);
        assert_eq!(
            state_machine
                .input(0)
                .and_then(|input| input.number_value()),
            Some(7.0),
            "listener observation must fall through an invalid first local instance"
        );
        assert_eq!(
            root.borrow().number_value_by_property_path(&[1, 1]),
            Some(42.0),
            "listener writes must follow the data-bind source into the later local instance"
        );
        assert_eq!(
            root.borrow().number_value_by_property_path(&[2, 1]),
            Some(0.0),
            "the unresolved first local instance must remain untouched"
        );
    }

    #[test]
    fn composite_context_listener_falls_through_main_to_global_slot() {
        let (file, mut artboard, mut state_machine) = owned_view_model_action_fixture(9687, true);
        let main = RuntimeOwnedViewModelHandle::new(
            RuntimeOwnedViewModelInstance::new(&file, 0)
                .expect("fixture has a main ViewModel context"),
        );
        let global = RuntimeOwnedViewModelHandle::new(
            RuntimeOwnedViewModelInstance::new(&file, 1)
                .expect("fixture has a global ViewModel context"),
        );
        let mut context = RuntimeOwnedViewModelContext::from_main_handle(main.clone());
        assert!(context.set_global_slot_handle(&file, 1, global.clone()));

        assert!(state_machine.bind_owned_view_model_contexts(&context));
        assert!(global.borrow_mut().set_number_by_property_index(0, 1.0));
        artboard.advance_state_machine_instance(&mut state_machine, 0.0);
        assert_eq!(
            state_machine
                .input(0)
                .and_then(|input| input.number_value()),
            Some(7.0),
            "listener observation must follow composite main-to-global ordering"
        );
        assert_eq!(
            global.borrow().number_value_by_property_path(&[1]),
            Some(42.0),
            "listener writes must reach the global slot that resolved the source"
        );
        assert_eq!(
            main.borrow().number_value_by_property_path(&[1, 1]),
            Some(0.0),
            "the composite main context must not receive the global source write"
        );
    }

    #[test]
    fn component_list_advance_writes_state_actions_to_the_item_context() {
        let (file, mut child, mut state_machine) = owned_view_model_action_fixture(9681, false);
        child.update_pass();
        let context = RuntimeOwnedViewModelInstance::new(&file, 1)
            .expect("fixture has an owned ViewModel context");

        let mut root_context = RuntimeOwnedViewModelInstance::new(&file, 0)
            .expect("fixture has a root owned ViewModel context");
        let list_source = root_context
            .list_source_handle_by_property_name("items")
            .expect("fixture root exposes its item list");
        assert_eq!(
            root_context.replace_list_items_by_source_handle(&list_source, vec![context.clone()]),
            Some(true)
        );
        let list = root_context
            .list_handle_by_property_path(list_source.path())
            .expect("fixture root retains its item list");
        let row = list
            .item_entries()
            .into_iter()
            .next()
            .expect("fixture list has one retained row");
        state_machine.bind_owned_view_model_handle(&row.instance);

        let mut parent = synthetic_instance(
            vec![synthetic_component_for_type(1, "ArtboardComponentList")],
            vec![1],
        );
        parent.set_component_list_source(1, Some(list.clone()));
        parent.component_list_state_mut(1).unwrap().items =
            vec![RuntimeComponentListItemInstance {
                child: Box::new(child),
                render_resources: RefCell::new(
                    crate::draw::RuntimeOccurrenceRenderResources::default(),
                ),
                state_machines: vec![state_machine],
                context_rebind_sink: {
                    let sink = crate::view_model_cell::RuntimeCellDirtSink::new();
                    row.instance.add_rebind_dependent(&sink);
                    sink
                },
                draw_index_sink: None,
                context: row.instance,
                occurrence_identity: row.occurrence_identity,
                logical_index: 0,
                settled_layout_size: Cell::new(None),
                transform: Mat2D::IDENTITY,
                render_cache_revision: row.occurrence_identity,
            }];

        let cache_epoch = parent.cache_epoch();
        let prepared_epoch = parent.prepared_epoch();
        let retained_path_generation = parent.path_epoch();
        let layout_revision = parent.layout_revision();
        assert!(parent.advance_nested_artboards(0.0));
        let context = &parent.component_list_items(1).unwrap()[0].context;
        assert_eq!(
            context.borrow().number_value_by_property_path(&[1]),
            Some(42.0)
        );
        assert_eq!(
            list.items()[0].borrow().number_value_by_property_path(&[1]),
            Some(42.0),
            "the retained list source must observe the item-owned write"
        );
        // C++ only dirties the component-list host when the mounted child
        // retains Components dirt after its advance. This fixture's scalar is
        // unprojected, so the row write stays local
        // (`artboard_component_list.cpp:827-885`, especially 870-881).
        assert_eq!(parent.cache_epoch(), cache_epoch);
        assert_eq!(parent.prepared_epoch(), prepared_epoch);
        assert_eq!(parent.path_epoch(), retained_path_generation);
        assert_eq!(parent.layout_revision(), layout_revision);
    }

    #[test]
    fn component_list_machine_rejects_inherited_cross_model_global_slot() {
        let (file, child, state_machine) = owned_view_model_action_fixture(9703, false);
        let row = RuntimeOwnedViewModelHandle::new(
            RuntimeOwnedViewModelInstance::new(&file, 2)
                .expect("fixture has a row that does not occupy declared slot 1"),
        );
        let main = RuntimeOwnedViewModelHandle::new(
            RuntimeOwnedViewModelInstance::new(&file, 0)
                .expect("fixture has the parent main ViewModel"),
        );
        let global_override = RuntimeOwnedViewModelHandle::new(
            RuntimeOwnedViewModelInstance::new(&file, 2)
                .expect("fixture has a compatible cross-model global override"),
        );
        let mut contexts = RuntimeOwnedViewModelContext::from_main_handle(main);
        assert!(contexts.set_global_slot_handle(&file, 1, global_override.clone()));

        let mut parent = synthetic_instance(
            vec![synthetic_component_for_type(1, "ArtboardComponentList")],
            vec![1],
        );
        parent.component_list_state_mut(1).unwrap().items =
            vec![RuntimeComponentListItemInstance {
                child: Box::new(child),
                render_resources: RefCell::new(
                    crate::draw::RuntimeOccurrenceRenderResources::default(),
                ),
                state_machines: vec![state_machine],
                context_rebind_sink: {
                    let sink = crate::view_model_cell::RuntimeCellDirtSink::new();
                    row.add_rebind_dependent(&sink);
                    sink
                },
                draw_index_sink: None,
                context: row.clone(),
                occurrence_identity: 1,
                logical_index: 0,
                settled_layout_size: Cell::new(None),
                transform: Mat2D::IDENTITY,
                render_cache_revision: 1,
            }];
        let _ = parent.bind_owned_view_model_artboard_contexts(&file, &contexts);

        assert!(parent.advance_nested_artboards(0.0));
        assert_eq!(
            global_override.borrow().number_value_by_property_path(&[1]),
            Some(0.0),
            "row parent fallback still resolves by actual viewModelId, never the inherited slot key (data_context.cpp:397-506)",
        );
        assert_eq!(
            row.borrow().number_value_by_property_path(&[1]),
            Some(0.0),
            "a same-shaped row of another ViewModel type must not steal the declared global action",
        );

        assert!(
            global_override
                .borrow_mut()
                .set_number_by_property_index(1, 17.0)
        );
        let _ = parent.advance_nested_artboards(0.0);
        assert_eq!(
            global_override.borrow().number_value_by_property_path(&[1]),
            Some(17.0),
            "the retained inherited alias must refresh without replaying the one-shot state action",
        );
    }

    #[test]
    fn component_list_reverse_writes_target_the_exact_repeated_occurrence() {
        let (file, child, mut state_machine) = owned_view_model_action_fixture(9682, false);
        let context = RuntimeOwnedViewModelInstance::new(&file, 1)
            .expect("fixture has an owned ViewModel context");

        let mut root_context = RuntimeOwnedViewModelInstance::new(&file, 0)
            .expect("fixture has a root owned ViewModel context");
        let list_source = root_context
            .list_source_handle_by_property_name("items")
            .expect("fixture root exposes its item list");
        assert_eq!(
            root_context.replace_list_items_by_source_handle(
                &list_source,
                vec![context.clone(), context.clone()],
            ),
            Some(true)
        );
        let list = root_context
            .list_handle_by_property_path(list_source.path())
            .expect("fixture root retains its item list");
        let rows = list.item_entries();
        let first = rows[0].clone();
        let second = rows[1].clone();
        state_machine.bind_owned_view_model_handle(&second.instance);

        let mut parent = synthetic_instance(
            vec![synthetic_component_for_type(1, "ArtboardComponentList")],
            vec![1],
        );
        parent.set_component_list_source(1, Some(list.clone()));
        parent.component_list_state_mut(1).unwrap().items = vec![
            RuntimeComponentListItemInstance {
                child: Box::new(synthetic_instance(Vec::new(), Vec::new())),
                render_resources: RefCell::new(
                    crate::draw::RuntimeOccurrenceRenderResources::default(),
                ),
                state_machines: Vec::new(),
                context_rebind_sink: {
                    let sink = crate::view_model_cell::RuntimeCellDirtSink::new();
                    first.instance.add_rebind_dependent(&sink);
                    sink
                },
                draw_index_sink: None,
                context: first.instance,
                occurrence_identity: first.occurrence_identity,
                logical_index: 0,
                settled_layout_size: Cell::new(None),
                transform: Mat2D::IDENTITY,
                render_cache_revision: first.occurrence_identity,
            },
            RuntimeComponentListItemInstance {
                child: Box::new(child),
                render_resources: RefCell::new(
                    crate::draw::RuntimeOccurrenceRenderResources::default(),
                ),
                state_machines: vec![state_machine],
                context_rebind_sink: {
                    let sink = crate::view_model_cell::RuntimeCellDirtSink::new();
                    second.instance.add_rebind_dependent(&sink);
                    sink
                },
                draw_index_sink: None,
                context: second.instance,
                occurrence_identity: second.occurrence_identity,
                logical_index: 1,
                settled_layout_size: Cell::new(None),
                transform: Mat2D::IDENTITY,
                render_cache_revision: second.occurrence_identity,
            },
        ];

        assert!(parent.advance_nested_artboards(0.0));
        let source_items = list.items();
        assert_eq!(
            source_items[0].borrow().number_value_by_property_path(&[1]),
            Some(0.0)
        );
        assert_eq!(
            source_items[1].borrow().number_value_by_property_path(&[1]),
            Some(42.0)
        );
    }

    fn synthetic_riv(file_id: u64, object_stream: impl FnOnce(&mut Vec<u8>)) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(b"RIVE");
        push_var_uint(&mut bytes, 7);
        push_var_uint(&mut bytes, 0);
        push_var_uint(&mut bytes, file_id);
        push_var_uint(&mut bytes, 0);
        object_stream(&mut bytes);
        bytes
    }

    fn instance_from_riv(bytes: &[u8]) -> ArtboardInstance {
        let file = read_runtime_file(bytes).expect("synthetic riv should import");
        let graph = GraphFile::from_runtime_file(&file).expect("synthetic riv should graph");
        let artboard = graph.artboards.first().expect("synthetic riv has artboard");
        ArtboardInstance::from_graph(&file, artboard).expect("instance builds")
    }

    #[test]
    fn nested_state_machine_rejects_artboard_list_map_rule_parent() {
        let bytes = synthetic_riv(9599, |bytes| {
            push_synthetic_object(bytes, "Backboard", &[]);
            push_synthetic_object(bytes, "Artboard", &[]);
            push_synthetic_object(bytes, "ArtboardListMapRule", &[("parentId", 0)]);
            push_synthetic_object(bytes, "NestedStateMachine", &[("parentId", 1)]);
        });
        let file = read_runtime_file(&bytes).expect("synthetic riv should import");
        let graph = GraphFile::from_runtime_file(&file).expect("synthetic riv should graph");
        let artboard = graph.artboards.first().expect("synthetic riv has artboard");

        let error = match ArtboardInstance::from_graph(&file, artboard) {
            Ok(_) => panic!("non-container Component parent must be rejected"),
            Err(error) => error,
        };
        assert_eq!(
            error.to_string(),
            "Component NestedStateMachine local 2 parent local 1 type ArtboardListMapRule is not a ContainerComponent"
        );
    }

    fn assert_collapsed(instance: &ArtboardInstance, local_id: usize, collapsed: bool) {
        assert_eq!(
            instance
                .component(local_id)
                .unwrap_or_else(|| panic!("missing component {local_id}"))
                .is_collapsed(),
            collapsed,
            "component {local_id} collapse mismatch"
        );
    }

    #[test]
    fn instantiating_an_artboard_without_solos_skips_solo_mapping_analysis() {
        let bytes = synthetic_riv(9600, |bytes| {
            push_synthetic_object(bytes, "Backboard", &[]);
            push_synthetic_object(bytes, "Artboard", &[]);
            push_synthetic_object(bytes, "Node", &[("parentId", 0)]);
            push_synthetic_object(bytes, "Shape", &[("parentId", 1)]);
        });

        reset_solo_mapping_work();
        let instance = instance_from_riv(&bytes);

        assert_collapsed(&instance, 1, false);
        assert_collapsed(&instance, 2, false);
        assert_eq!(solo_mapping_work(), SoloMappingWork::default());
    }

    #[test]
    fn imported_solo_mapping_preserves_a_null_slot_before_the_active_child() {
        let bytes = synthetic_riv(9604, |bytes| {
            push_synthetic_object(bytes, "Backboard", &[]);
            push_synthetic_object(bytes, "Artboard", &[]);
            // Local 1: abstract runtime objects become indexed null slots in
            // C++ Artboard::objects(). They must not compact later local ids.
            push_synthetic_object(bytes, "BindableProperty", &[]);
            // Local 2: solo; local 4 is active across the null slot.
            push_synthetic_object(bytes, "Solo", &[("parentId", 0), ("activeComponentId", 4)]);
            push_synthetic_object(bytes, "Node", &[("parentId", 2)]);
            push_synthetic_object(bytes, "Node", &[("parentId", 2)]);
        });

        reset_solo_mapping_work();
        let instance = instance_from_riv(&bytes);

        assert_collapsed(&instance, 3, true);
        assert_collapsed(&instance, 4, false);
        let solo = instance
            .component(2)
            .and_then(|component| component.concrete.solo.as_ref())
            .expect("Solo occurrence owns its retained state");
        assert_eq!(solo.cpp_local_ids, [3, 4]);
        assert_eq!(
            solo_mapping_work(),
            SoloMappingWork {
                analyses: 1,
                batch_queries: 1,
                visited_slots: 5,
            }
        );
    }

    fn imported_solo_mapping_work(child_count: usize) -> SoloMappingWork {
        let bytes = synthetic_riv(9605 + child_count as u64, |bytes| {
            push_synthetic_object(bytes, "Backboard", &[]);
            push_synthetic_object(bytes, "Artboard", &[]);
            push_synthetic_object(bytes, "Solo", &[("parentId", 0), ("activeComponentId", 2)]);
            for _ in 0..child_count {
                push_synthetic_object(bytes, "Node", &[("parentId", 1)]);
            }
        });

        reset_solo_mapping_work();
        let instance = instance_from_riv(&bytes);
        assert_eq!(
            instance
                .components()
                .iter()
                .filter(|component| component.concrete.solo.is_some())
                .count(),
            1
        );
        solo_mapping_work()
    }

    #[test]
    fn solo_mapping_analysis_is_one_batched_linear_pass() {
        let small_child_count = 8;
        let large_child_count = 64;

        let small = imported_solo_mapping_work(small_child_count);
        let large = imported_solo_mapping_work(large_child_count);

        assert_eq!(small.analyses, 1);
        assert_eq!(large.analyses, 1);
        assert_eq!(small.batch_queries, 1);
        assert_eq!(large.batch_queries, 1);
        assert_eq!(small.visited_slots, small_child_count + 2);
        assert_eq!(large.visited_slots, large_child_count + 2);
    }

    #[test]
    fn imported_solos_retain_children_on_each_occurrence_without_behavior_drift() {
        let bytes = synthetic_riv(9670, |bytes| {
            push_synthetic_object(bytes, "Backboard", &[]);
            push_synthetic_object(bytes, "Artboard", &[]);
            // Local 1 with active/inactive children 2 and 3.
            push_synthetic_object(bytes, "Solo", &[("parentId", 0), ("activeComponentId", 2)]);
            push_synthetic_object(bytes, "Node", &[("parentId", 1)]);
            push_synthetic_object(bytes, "Node", &[("parentId", 1)]);
            // Local 4 with active/inactive children 5 and 6.
            push_synthetic_object(bytes, "Solo", &[("parentId", 0), ("activeComponentId", 5)]);
            push_synthetic_object(bytes, "Node", &[("parentId", 4)]);
            push_synthetic_object(bytes, "Node", &[("parentId", 4)]);
        });

        reset_solo_mapping_work();
        let mut instance = instance_from_riv(&bytes);

        let first = instance
            .component(1)
            .and_then(|component| component.concrete.solo.as_ref())
            .expect("first Solo occurrence");
        let second = instance
            .component(4)
            .and_then(|component| component.concrete.solo.as_ref())
            .expect("second Solo occurrence");
        assert_eq!(first.cpp_local_ids, [2, 3]);
        assert_eq!(second.cpp_local_ids, [5, 6]);
        assert_eq!(
            solo_mapping_work(),
            SoloMappingWork {
                analyses: 1,
                batch_queries: 1,
                visited_slots: 7,
            }
        );

        assert_collapsed(&instance, 2, false);
        assert_collapsed(&instance, 3, true);
        assert_collapsed(&instance, 5, false);
        assert_collapsed(&instance, 6, true);

        assert!(instance.set_solo_active_child_by_index(1, 1.0));
        assert_collapsed(&instance, 2, true);
        assert_collapsed(&instance, 3, false);
        assert_collapsed(&instance, 5, false);
        assert_collapsed(&instance, 6, true);

        assert!(instance.set_solo_active_child_by_index(4, 1.0));
        assert_collapsed(&instance, 2, true);
        assert_collapsed(&instance, 3, false);
        assert_collapsed(&instance, 5, true);
        assert_collapsed(&instance, 6, false);
    }

    #[test]
    fn solo_index_and_name_selection_skip_property_like_children() {
        let bytes = synthetic_riv(0, |bytes| {
            push_synthetic_object(bytes, "Backboard", &[]);
            push_synthetic_object(bytes, "Artboard", &[]);
            push_synthetic_object(bytes, "Solo", &[("parentId", 0), ("activeComponentId", 4)]);
            push_synthetic_object(bytes, "ClippingShape", &[("parentId", 1), ("sourceId", 4)]);
            push_synthetic_object(bytes, "TranslationConstraint", &[("parentId", 1)]);
            push_synthetic_object_with_properties(bytes, "Shape", |bytes| {
                push_synthetic_uint_property(bytes, "Shape", "parentId", 1);
                push_synthetic_string_property(bytes, "Shape", "name", "Blue");
            });
            push_synthetic_object(bytes, "FocusData", &[("parentId", 1)]);
            push_synthetic_object_with_properties(bytes, "Shape", |bytes| {
                push_synthetic_uint_property(bytes, "Shape", "parentId", 1);
                push_synthetic_string_property(bytes, "Shape", "name", "Green");
            });
            push_synthetic_object(bytes, "SemanticData", &[("parentId", 1)]);
            push_synthetic_object_with_properties(bytes, "Shape", |bytes| {
                push_synthetic_uint_property(bytes, "Shape", "parentId", 1);
                push_synthetic_string_property(bytes, "Shape", "name", "Red");
            });
        });
        let file = read_runtime_file(&bytes).expect("solo member fixture imports");
        let graph = GraphFile::from_runtime_file(&file)
            .expect("solo member graph builds")
            .artboards
            .remove(0);
        let mut instance = ArtboardInstance::from_graph(&file, &graph).expect("artboard builds");

        assert!(instance.set_solo_active_child_by_index(1, 1.0));
        assert_collapsed(&instance, 4, true);
        assert_collapsed(&instance, 6, false);
        assert_collapsed(&instance, 8, true);
        for property_like in [2, 3, 5, 7] {
            assert_collapsed(&instance, property_like, false);
        }

        assert!(instance.set_solo_active_child_by_name(1, b"Red"));
        assert_collapsed(&instance, 4, true);
        assert_collapsed(&instance, 6, true);
        assert_collapsed(&instance, 8, false);
        assert!(!instance.set_solo_active_child_by_index(1, 3.0));
        assert!(!instance.set_solo_active_child_by_name(1, b""));
    }

    // Regression for the M8 audit finding: apply_initial_solo_collapses only
    // flagged DIRECT solo children, so Solo -> Group -> Shape left the Shape
    // un-collapsed (and drawing) on a fresh instance without a state machine.
    // C++ Solo::onAddedClean recurses the full subtree (src/solo.cpp).
    #[test]
    fn initial_solo_collapse_propagates_to_deep_descendants() {
        let bytes = synthetic_riv(9601, |bytes| {
            push_synthetic_object(bytes, "Backboard", &[]);
            push_synthetic_object(bytes, "Artboard", &[]);
            // Local 1: solo with the first group active.
            push_synthetic_object(bytes, "Solo", &[("parentId", 0), ("activeComponentId", 2)]);
            // Local 2/3: active branch.
            push_synthetic_object(bytes, "Node", &[("parentId", 1)]);
            push_synthetic_object(bytes, "Shape", &[("parentId", 2)]);
            // Local 4/5: statically-inactive branch.
            push_synthetic_object(bytes, "Node", &[("parentId", 1)]);
            push_synthetic_object(bytes, "Shape", &[("parentId", 4)]);
        });
        let instance = instance_from_riv(&bytes);
        assert_collapsed(&instance, 2, false);
        assert_collapsed(&instance, 3, false);
        assert_collapsed(&instance, 4, true);
        // The deep descendant was left un-collapsed before the fix.
        assert_collapsed(&instance, 5, true);
        let inactive_composer = instance
            .objects
            .path_composer_handle(5)
            .expect("inactive Shape owns its PathComposer");
        assert!(
            instance
                .objects
                .component(inactive_composer)
                .expect("inactive PathComposer remains live")
                .is_collapsed(),
            "Shape::collapse must keep its embedded PathComposer collapsed"
        );
    }

    // Regression for the M8 audit finding: collapse propagation from a
    // display:none layout recursed only into Artboard|LayoutComponent
    // children, so display:none -> Node -> Shape still drew. C++
    // LayoutComponent::propagateCollapse recurses through
    // ContainerComponent::collapse (src/layout_component.cpp).
    #[test]
    fn initial_display_none_collapse_propagates_to_deep_descendants() {
        let bytes = synthetic_riv(9602, |bytes| {
            push_synthetic_object(bytes, "Backboard", &[]);
            push_synthetic_object(bytes, "Artboard", &[]);
            // Local 1: hidden layout; local 2: its style with display:none.
            push_synthetic_object(bytes, "LayoutComponent", &[("parentId", 0), ("styleId", 2)]);
            push_synthetic_object(bytes, "LayoutComponentStyle", &[("displayValue", 1)]);
            // Local 3/4: plain-node chain under the hidden layout.
            push_synthetic_object(bytes, "Node", &[("parentId", 1)]);
            push_synthetic_object(bytes, "Shape", &[("parentId", 3)]);
        });
        let instance = instance_from_riv(&bytes);

        assert_collapsed(&instance, 3, true);
        // The deep descendant was left un-collapsed before the fix.
        assert_collapsed(&instance, 4, true);
    }

    // Regression for the M8 audit finding: collapse_component_tree_with_ancestor
    // blindly un-collapsed descendants, clobbering a nested solo's
    // re-collapsed inactive children. C++ Solo::collapse skips the blind
    // container child walk (src/solo.cpp).
    #[test]
    fn solo_switch_preserves_nested_solo_inactive_collapse() {
        let bytes = synthetic_riv(9603, |bytes| {
            push_synthetic_object(bytes, "Backboard", &[]);
            push_synthetic_object(bytes, "Artboard", &[]);
            // Local 1: outer solo, group A (local 2) active.
            push_synthetic_object(bytes, "Solo", &[("parentId", 0), ("activeComponentId", 2)]);
            push_synthetic_object(bytes, "Node", &[("parentId", 1)]);
            // Local 3: inactive group B holding a nested solo.
            push_synthetic_object(bytes, "Node", &[("parentId", 1)]);
            // Local 4: inner solo, group C (local 5) active.
            push_synthetic_object(bytes, "Solo", &[("parentId", 3), ("activeComponentId", 5)]);
            push_synthetic_object(bytes, "Node", &[("parentId", 4)]);
            push_synthetic_object(bytes, "Shape", &[("parentId", 5)]);
            // Local 7/8: inner solo's inactive branch.
            push_synthetic_object(bytes, "Node", &[("parentId", 4)]);
            push_synthetic_object(bytes, "Shape", &[("parentId", 7)]);
        });
        let mut instance = instance_from_riv(&bytes);

        // Fresh instance: the whole inactive outer branch is collapsed.
        for local_id in 3..=8 {
            assert_collapsed(&instance, local_id, true);
        }

        // Switch the outer solo to group B (child index 1).
        assert!(instance.set_solo_active_child_by_index(1, 1.0));

        assert_collapsed(&instance, 2, true);
        assert_collapsed(&instance, 3, false);
        assert_collapsed(&instance, 4, false);
        assert_collapsed(&instance, 5, false);
        assert_collapsed(&instance, 6, false);
        // The nested solo's inactive branch must stay collapsed; the blind
        // descendant walk un-collapsed it before the fix.
        assert_collapsed(&instance, 7, true);
        assert_collapsed(&instance, 8, true);
    }

    #[test]
    fn component_list_children_select_the_cpp_default_state_machine_index() {
        assert_eq!(component_list_default_state_machine_index(Some(1), 3), 1);
        assert_eq!(component_list_default_state_machine_index(None, 3), 0);
        assert_eq!(component_list_default_state_machine_index(Some(3), 3), 0);
        assert_eq!(
            component_list_default_state_machine_index(Some(u64::MAX), 3),
            0
        );
    }

    #[test]
    fn state_machine_frame_preserves_deep_nested_artboard_opacity() {
        let typed_component = |local_id: usize, graph_order: usize, type_name: &'static str| {
            let mut component = synthetic_component(local_id, graph_order);
            component.type_name = type_name;
            component.transform_property_keys =
                crate::components::TransformPropertyKeys::for_type(type_name);
            component
        };

        let mut leaf_root = typed_component(0, 0, "Artboard");
        leaf_root.transform.render_opacity = 0.0;
        let mut leaf = synthetic_instance(vec![leaf_root], vec![0]);
        let opacity_key = property_key_for_name("Artboard", "opacity").expect("opacity key");
        assert!(leaf.set_double_property(0, opacity_key, 0.0));
        leaf.clear_component_dirt(0);
        leaf.set_artboard_dirt_for_test(ComponentDirt::NONE);

        let mut middle_root = typed_component(0, 0, "Artboard");
        middle_root.transform.render_opacity = 1.0;
        let mut middle_host = typed_component(1, 1, "NestedArtboard");
        middle_host.dirt = ComponentDirt::RENDER_OPACITY;
        let mut middle = synthetic_instance(vec![middle_root, middle_host], vec![1]);
        synthetic_link_parent(&mut middle, 1, 0);
        let mut leaf_mount = synthetic_nested_artboard_instance(2);
        leaf_mount.child = Box::new(leaf);
        middle.nested_artboards.insert(1, leaf_mount);
        middle.nested_artboard_locals.push(1);
        middle.set_artboard_dirt_for_test(ComponentDirt::COMPONENTS);

        let mut root_component = typed_component(0, 0, "Artboard");
        root_component.transform.render_opacity = 1.0;
        let mut root_host = typed_component(1, 1, "NestedArtboard");
        root_host.transform.render_opacity = 1.0;
        root_host.dirt = ComponentDirt::COMPONENTS;
        let mut root = synthetic_instance(vec![root_component, root_host], vec![1]);
        synthetic_link_parent(&mut root, 1, 0);
        let mut middle_mount = synthetic_nested_artboard_instance(1);
        middle_mount.child = Box::new(middle);
        root.nested_artboards.insert(1, middle_mount);
        root.nested_artboard_locals.push(1);

        root.update_pass();

        let middle = root
            .nested_artboards
            .values()
            .next()
            .expect("middle occurrence");
        let leaf = middle
            .child
            .nested_artboards
            .values()
            .next()
            .expect("leaf occurrence");
        let leaf_root = leaf.child.component(0).expect("leaf root component");
        assert_eq!(leaf_root.transform.render_opacity, 0.0);
        assert_eq!(leaf.child.host_opacity, 1.0);
        assert_eq!(leaf.child.child_opacity(), 0.0);
        assert!(!leaf_root.dirt.contains(ComponentDirt::RENDER_OPACITY));

        root.settle_state_machine_update_passes();

        let middle = root
            .nested_artboards
            .values()
            .next()
            .expect("middle occurrence");
        let leaf = middle
            .child
            .nested_artboards
            .values()
            .next()
            .expect("leaf occurrence");
        let leaf_root = leaf.child.component(0).expect("leaf root component");
        assert_eq!(leaf_root.transform.render_opacity, 0.0);
        assert_eq!(leaf.child.host_opacity, 1.0);
        assert_eq!(leaf.child.child_opacity(), 0.0);
        assert!(!leaf_root.dirt.contains(ComponentDirt::RENDER_OPACITY));
    }

    #[test]
    fn component_list_row_state_machine_uses_parent_focus_domain_for_all_input_channels() {
        let bytes = synthetic_riv(9703, |bytes| {
            push_synthetic_object(bytes, "ViewModel", &[]);
            push_synthetic_object(bytes, "ViewModelPropertyList", &[]);
            push_synthetic_object(bytes, "ViewModel", &[]);
            push_synthetic_object(bytes, "Backboard", &[]);
            push_synthetic_object(bytes, "ViewModelInstance", &[("viewModelId", 0)]);
            push_synthetic_object(
                bytes,
                "ViewModelInstanceList",
                &[("viewModelPropertyId", 0)],
            );
            push_synthetic_object(bytes, "ViewModelInstance", &[("viewModelId", 1)]);
            push_synthetic_object(
                bytes,
                "ViewModelInstanceListItem",
                &[("viewModelId", 1), ("viewModelInstanceId", 0)],
            );
            push_synthetic_object(bytes, "Artboard", &[("viewModelId", 0)]);
            push_synthetic_object_with_properties(bytes, "ArtboardComponentList", |bytes| {
                push_synthetic_uint_property(bytes, "ArtboardComponentList", "parentId", 0);
                push_synthetic_f32_property(bytes, "ArtboardComponentList", "opacity", 1.0);
            });
            // The parent StateMachine owns the shared FocusManager before the
            // row is mounted.
            push_synthetic_object(bytes, "StateMachine", &[]);
            push_synthetic_object(bytes, "Artboard", &[("viewModelId", 1)]);
            push_synthetic_object_with_properties(bytes, "Node", |bytes| {
                push_synthetic_uint_property(bytes, "Node", "parentId", 0);
                push_synthetic_f32_property(bytes, "Node", "opacity", 1.0);
            });
            push_synthetic_object(bytes, "FocusData", &[("parentId", 1), ("focusFlags", 7)]);
            push_synthetic_object(bytes, "StateMachine", &[]);
            push_synthetic_object(bytes, "StateMachineBool", &[]);
            push_synthetic_object(bytes, "StateMachineListener", &[("targetId", 1)]);
            push_synthetic_object(
                bytes,
                "ListenerInputTypeKeyboard",
                &[("listenerTypeValue", RuntimeListenerType::Keyboard as u64)],
            );
            push_synthetic_object(
                bytes,
                "ListenerInputTypeText",
                &[("listenerTypeValue", RuntimeListenerType::TextInput as u64)],
            );
            push_synthetic_object(
                bytes,
                "ListenerInputTypeGamepad",
                &[("listenerTypeValue", RuntimeListenerType::Gamepad as u64)],
            );
            // Toggle once for each delivered channel so order/duplication is
            // observable without relying on a handled return.
            push_synthetic_object(bytes, "ListenerBoolChange", &[("inputId", 0), ("value", 2)]);
        });
        let file = read_runtime_file(&bytes).expect("component-list focus fixture imports");
        let graph = GraphFile::from_runtime_file(&file).expect("component-list focus graphs");
        let mut parent = ArtboardInstance::from_graph_with_artboards(
            &file,
            &graph.artboards[0],
            &graph.artboards,
        )
        .expect("parent artboard instance");
        // The ordinary runtime constructs focus topology after the initial
        // component update has published render opacity. This low-level
        // fixture bypasses the facade initialization, so mirror that C++
        // ordering explicitly.
        parent.update_components();
        let mut parent_machine = parent
            .state_machine_instance(0)
            .expect("parent focus-manager owner");

        let list_local_id = graph.artboards[0].component_lists[0].local_id;
        let row_context = RuntimeOwnedViewModelHandle::new(
            RuntimeOwnedViewModelInstance::from_instance(&file, 1, 0)
                .expect("component-list row context"),
        );
        assert!(parent.sync_component_list_items(&file, list_local_id, vec![row_context],));

        // Refresh the parent's retained projection after the dynamic row was
        // linked, then focus through the row machine's shared manager handle.
        parent_machine.sync_focus_for_test(&parent);
        let row_target_local = graph.artboards[1]
            .components
            .iter()
            .find(|component| component.type_name == "Node")
            .expect("row focus target")
            .local_id;
        {
            let row = parent
                .component_list_items_mut(list_local_id)
                .and_then(|items| items.first_mut())
                .expect("mounted row");
            assert!(
                row.state_machines[0].set_focus_target_for_test(row_target_local),
                "C++ setExternalFocusManager makes the row target visible in the parent domain"
            );
        }

        assert!(!parent_machine.key_input(&mut parent, 65, 0, true, false));
        assert_eq!(
            parent.component_list_items(list_local_id).unwrap()[0].state_machines[0]
                .input(0)
                .and_then(StateMachineInputInstance::bool_value),
            Some(true)
        );
        assert!(!parent_machine.text_input(&mut parent, "owned"));
        assert_eq!(
            parent.component_list_items(list_local_id).unwrap()[0].state_machines[0]
                .input(0)
                .and_then(StateMachineInputInstance::bool_value),
            Some(false)
        );
        assert!(!parent_machine.gamepad_dispatch(
            &mut parent,
            ScriptListenerInvocation::GamepadConnected {
                snapshot: ScriptGamepadSnapshot {
                    device_id: 7,
                    button_mask: 0,
                    button_values: Vec::new(),
                    axes: Vec::new(),
                    mapping: ScriptGamepadMappingKind::Standard,
                },
            },
        ));
        assert_eq!(
            parent.component_list_items(list_local_id).unwrap()[0].state_machines[0]
                .input(0)
                .and_then(StateMachineInputInstance::bool_value),
            Some(true),
            "key, text, and gamepad each reach the same list-row occurrence exactly once"
        );
    }

    #[test]
    fn component_list_mount_settles_context_without_advancing_the_row_state_machine() {
        let bytes = synthetic_riv(9702, |bytes| {
            push_synthetic_object(bytes, "ViewModel", &[]);
            push_synthetic_object(bytes, "ViewModelPropertyList", &[]);
            push_synthetic_object(bytes, "ViewModel", &[]);
            push_synthetic_object(bytes, "Backboard", &[]);
            push_synthetic_object(bytes, "ViewModelInstance", &[("viewModelId", 0)]);
            push_synthetic_object(
                bytes,
                "ViewModelInstanceList",
                &[("viewModelPropertyId", 0)],
            );
            push_synthetic_object(bytes, "ViewModelInstance", &[("viewModelId", 1)]);
            push_synthetic_object(
                bytes,
                "ViewModelInstanceListItem",
                &[("viewModelId", 1), ("viewModelInstanceId", 0)],
            );
            push_synthetic_object(bytes, "Artboard", &[("viewModelId", 0)]);
            push_synthetic_object(bytes, "ArtboardComponentList", &[("parentId", 0)]);
            push_synthetic_object(bytes, "Artboard", &[("viewModelId", 1)]);
            push_synthetic_object(bytes, "StateMachine", &[]);
            push_synthetic_object(bytes, "StateMachineLayer", &[]);
            push_synthetic_object(bytes, "AnyState", &[]);
            push_synthetic_object(bytes, "EntryState", &[]);
            push_synthetic_object(bytes, "StateTransition", &[("stateToId", 2)]);
            push_synthetic_object(bytes, "ExitState", &[]);
        });
        let file = read_runtime_file(&bytes).expect("component-list mount fixture imports");
        let graph = GraphFile::from_runtime_file(&file).expect("component-list fixture graphs");
        let mut parent = ArtboardInstance::from_graph_with_artboards(
            &file,
            &graph.artboards[0],
            &graph.artboards,
        )
        .expect("parent artboard instance");

        let list_local_id = graph.artboards[0].component_lists[0].local_id;
        let row_context = RuntimeOwnedViewModelHandle::new(
            RuntimeOwnedViewModelInstance::from_instance(&file, 1, 0)
                .expect("component-list row context"),
        );
        assert!(parent.sync_component_list_items(&file, list_local_id, vec![row_context.clone()],));
        let mounted = parent
            .component_list_items(list_local_id)
            .and_then(|items| items.first())
            .expect("mounted component-list row");
        assert!(mounted.context.ptr_eq(&row_context));
        assert!(
            mounted
                .child
                .owned_view_model_context()
                .and_then(RuntimeOwnedViewModelContext::main_handle)
                .is_some_and(|context| context.ptr_eq(&row_context)),
            "the mounted child must publicly expose its occurrence-scoped row context"
        );
        assert_eq!(
            mounted
                .state_machines
                .first()
                .expect("row default state machine")
                .changed_state_count(),
            0,
            "mount links and settles the row context but leaves state advancement to the normal list pass"
        );

        parent.advance_nested_artboards(0.0);
        let advanced = parent.component_list_items(list_local_id).unwrap()[0]
            .state_machines
            .first()
            .expect("row default state machine");
        assert_eq!(advanced.changed_state_count(), 1);

        let pooled_identity = parent.component_list_items(list_local_id).unwrap()[0]
            .child
            .instance_identity();
        parent.component_list_items(list_local_id).unwrap()[0]
            .settled_layout_size
            .set(Some((119.666_664, 58.0)));
        let source_global_id = parent.component_list_items(list_local_id).unwrap()[0]
            .child
            .graph_global_id;
        assert!(parent.remove_component_list_virtualizable(list_local_id, 0));
        assert_eq!(
            parent
                .component_list_resource_pools
                .count(list_local_id, source_global_id),
            1
        );
        assert!(parent.add_component_list_virtualizable(&file, list_local_id, 0));
        let remounted = &parent.component_list_items(list_local_id).unwrap()[0];
        assert_eq!(remounted.child.instance_identity(), pooled_identity);
        assert_eq!(
            remounted.settled_layout_size.get(),
            Some((119.666_664, 58.0)),
            "pool reuse keeps the transferred root layout result until the hosting parent solves it again"
        );
        assert_eq!(
            remounted.state_machines[0].changed_state_count(),
            0,
            "safe recorder restoration resets pooled mutable state before rebinding"
        );
        assert!(remounted.context.ptr_eq(&row_context));
        assert_eq!(
            parent
                .component_list_resource_pools
                .count(list_local_id, source_global_id),
            0
        );
    }

    #[test]
    fn component_list_override_without_mounted_rows_keeps_the_host_clean() {
        let bytes = synthetic_riv(9703, |bytes| {
            push_synthetic_object(bytes, "Backboard", &[]);
            push_synthetic_object(bytes, "Artboard", &[]);
            push_synthetic_object(bytes, "ArtboardComponentList", &[("parentId", 0)]);
            push_synthetic_object(
                bytes,
                "ArtboardComponentListOverride",
                &[("parentId", 1)],
            );
        });
        let file = read_runtime_file(&bytes).expect("component-list override fixture imports");
        let graph = GraphFile::from_runtime_file(&file).expect("component-list fixture graphs");
        let mut instance =
            ArtboardInstance::from_graph(&file, &graph.artboards[0]).expect("artboard instance");
        instance.update_pass();

        let instance_height =
            property_key_for_name("ArtboardComponentListOverride", "instanceHeight")
                .expect("override height property");
        assert!(instance.set_double_property(2, instance_height, 100.0));
        assert!(
            !instance.has_dirt(ComponentDirt::COMPONENTS),
            "C++ walks only the override's mounted m_artboards; an empty host has no layout invalidation to bubble (`artboard_component_list_override.cpp:7-44`)"
        );
    }

    #[test]
    fn node_hosted_component_list_keeps_mounted_artboard_base_transform_identity() {
        let bytes = synthetic_riv(9703, |bytes| {
            push_synthetic_object(bytes, "ViewModel", &[]);
            push_synthetic_object(bytes, "ViewModelPropertyList", &[]);
            push_synthetic_object(bytes, "ViewModel", &[]);
            push_synthetic_object(bytes, "Backboard", &[]);
            push_synthetic_object(bytes, "ViewModelInstance", &[("viewModelId", 0)]);
            push_synthetic_object(
                bytes,
                "ViewModelInstanceList",
                &[("viewModelPropertyId", 0)],
            );
            push_synthetic_object(bytes, "ViewModelInstance", &[("viewModelId", 1)]);
            push_synthetic_object(
                bytes,
                "ViewModelInstanceListItem",
                &[("viewModelId", 1), ("viewModelInstanceId", 0)],
            );
            push_synthetic_object(bytes, "Artboard", &[("viewModelId", 0)]);
            push_synthetic_object(bytes, "Node", &[("parentId", 0)]);
            push_synthetic_object(bytes, "ArtboardComponentList", &[("parentId", 1)]);
            push_synthetic_object_with_properties(bytes, "Artboard", |bytes| {
                push_synthetic_uint_property(bytes, "Artboard", "viewModelId", 1);
                push_synthetic_f32_property(bytes, "Artboard", "x", 1121.0);
                push_synthetic_f32_property(bytes, "Artboard", "y", 259.0);
            });
        });
        let file = read_runtime_file(&bytes).expect("component-list transform fixture imports");
        let graph = GraphFile::from_runtime_file(&file).expect("component-list fixture graphs");
        let mut parent = ArtboardInstance::from_graph_with_artboards(
            &file,
            &graph.artboards[0],
            &graph.artboards,
        )
        .expect("parent artboard instance");
        let list_local_id = graph.artboards[0].component_lists[0].local_id;
        let row_context = RuntimeOwnedViewModelHandle::new(
            RuntimeOwnedViewModelInstance::from_instance(&file, 1, 0)
                .expect("component-list row context"),
        );

        assert!(parent.sync_component_list_items(&file, list_local_id, vec![row_context],));

        assert_eq!(
            runtime_component_list_item_base_transforms(&parent, list_local_id),
            vec![Mat2D::IDENTITY],
            "a flowless mounted Artboard does not project its authored root x/y into the list transform (`artboard_component_list.cpp:1306-1329,1453-1483`)"
        );
    }

    #[test]
    fn component_list_rows_apply_their_item_index_to_child_data_binds() {
        let file = read_runtime_file(include_bytes!(
            "../../../../fixtures/graph/clipping_and_draw_order.riv"
        ))
        .expect("clipping fixture imports");
        let graphs = GraphFile::from_runtime_file(&file).expect("clipping fixture graphs");
        let graph = graphs.artboards.first().expect("fixture root artboard");
        let mut parent =
            ArtboardInstance::from_graph_with_artboards(&file, graph, &graphs.artboards)
                .expect("parent artboard instance");
        let view_model_index = usize::try_from(
            file.object(graph.global_id as usize)
                .and_then(|artboard| artboard.uint_property("viewModelId"))
                .expect("root artboard view model"),
        )
        .expect("view-model index fits usize");
        let mut context = RuntimeOwnedViewModelContext::from_main(
            RuntimeOwnedViewModelInstance::new(&file, view_model_index)
                .expect("root view-model instance"),
        );
        context.complete_for_artboard(&file, 0);

        assert!(parent.bind_owned_view_model_artboard_contexts(&file, &context));
        let mut state_machine = parent
            .state_machine_instance(0)
            .expect("root state machine instance");
        state_machine.bind_owned_view_model_contexts(&context);
        state_machine.advance_data_context();
        for elapsed in [0.0, 0.5] {
            parent.advance_state_machine_instance(&mut state_machine, elapsed);
            parent
                .advance_frame_components_with_state_machine_report(elapsed, &mut state_machine)
                .expect("component-list frame advance");
            parent
                .settle_state_machine_update_passes_after_main_advance_with_script_errors(
                    std::slice::from_mut(&mut state_machine),
                )
                .expect("component-list settle passes");
        }

        let list_local_id = graph.component_lists[0].local_id;
        let rows = parent
            .component_list_items(list_local_id)
            .expect("component-list rows mount");
        let rotation_key =
            property_key_for_name("TransformComponent", "rotation").expect("rotation property key");
        assert_eq!(rows.len(), 3);
        assert_eq!(
            rows[1]
                .context
                .borrow()
                .symbol_list_index_value_by_property_path(&[0]),
            Some(1)
        );
        assert_eq!(
            rows[1].child.artboard_data_bind_values.get(&[0, 0][..]),
            Some(&RuntimeDataBindGraphValue::SymbolListIndex(1))
        );
        assert_eq!(
            rows[1].child.double_property(1, rotation_key),
            Some(1.0),
            "the row's synthetic itemIndex is a numeric converter input for child Artboard DataBinds (`data_converter_operation.cpp:9-16`; `artboard_component_list.cpp:715-814,1492-1543`)"
        );
    }

    #[test]
    fn retained_layout_size_change_publishes_path_before_world() {
        let root = synthetic_component_for_type(0, "Artboard");
        let layout = synthetic_component_for_type(1, "LayoutComponent");
        let style = synthetic_component_for_type(2, "LayoutComponentStyle");
        let mut instance = synthetic_instance(vec![root, layout, style], vec![0, 1, 2]);
        synthetic_link_parent(&mut instance, 1, 0);
        synthetic_link_parent(&mut instance, 2, 1);

        instance.retain_runtime_layout_component_bounds(
            1,
            RuntimeLayoutBounds {
                x: 0.0,
                y: 0.0,
                width: 100.0,
                height: 80.0,
            },
            None,
        );
        instance.clear_component_dirt(1);
        instance.retain_runtime_layout_component_bounds(
            1,
            RuntimeLayoutBounds {
                x: 0.0,
                y: 0.0,
                width: 100.0,
                height: 120.0,
            },
            None,
        );

        let dirt = instance.component(1).expect("layout component").dirt;
        assert!(dirt.contains(ComponentDirt::PATH));
        assert!(dirt.contains(ComponentDirt::WORLD_TRANSFORM));
    }
