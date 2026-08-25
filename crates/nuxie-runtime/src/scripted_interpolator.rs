use std::fmt;
use std::rc::Rc;

use crate::artboard_data_bind::RuntimeOwnedDataContext;
use crate::scripting::RuntimeScriptInstanceHandle;
use crate::state_machine::{
    RuntimeScriptedListenerActionBindingDefinition, RuntimeScriptedListenerActionBindingOccurrence,
    RuntimeScriptedListenerBoundValue, RuntimeScriptedListenerDataConverterBindStep,
    runtime_scripted_object_binding_definition,
};
use crate::{
    ArtboardInstance, NoopScriptHost, RuntimeOwnedViewModelHandle, ScriptError, ScriptHost,
    ScriptInstance, ScriptInterpolatorMethod, ScriptListenerActionHydration,
    ScriptListenerInputSnapshot, ScriptListenerInputSnapshotValue, ScriptOptionalNumberResult,
};
use nuxie_binary::RuntimeFile;

/// Immutable recipe for the ScriptInput properties, DataBinds, and converters
/// cloned with one imported `ScriptedInterpolator`.
///
/// Pinned C++ shares `ScriptedObject::cloneProperties` with listener actions
/// and transition conditions. This wrapper deliberately reuses the same Rust
/// lifecycle implementation so every keyframe occurrence receives fresh
/// target properties, retained source edges, and converter state.
#[doc(hidden)]
#[derive(Debug, Clone)]
pub struct RuntimeScriptedInterpolatorBindingDefinition {
    inner: RuntimeScriptedListenerActionBindingDefinition,
}

impl RuntimeScriptedInterpolatorBindingDefinition {
    pub fn from_imported(
        file: &RuntimeFile,
        graph_global_id: u32,
        interpolator_local_id: usize,
        interpolator_global_id: u32,
    ) -> Option<Self> {
        let interpolator = file.object(interpolator_global_id as usize)?;
        if interpolator.type_name != "ScriptedInterpolator" {
            return None;
        }
        let start = graph_global_id as usize;
        let end = ((start + 1)..file.object_count())
            .find(|global_id| {
                file.object(*global_id)
                    .is_some_and(|object| object.type_name == "Artboard")
            })
            .unwrap_or_else(|| file.object_count());
        let inputs = (start..end)
            .filter_map(|global_id| file.object(global_id))
            .filter(|input| {
                input.type_name.starts_with("ScriptInput")
                    && input.uint_property("parentId") == Some(interpolator_local_id as u64)
            })
            .collect::<Vec<_>>();
        runtime_scripted_object_binding_definition(file, interpolator, &inputs)
            .map(|inner| Self { inner })
    }

    pub fn instantiate(&self) -> RuntimeScriptedInterpolatorBindingOccurrence {
        RuntimeScriptedInterpolatorBindingOccurrence {
            inner: self.inner.instantiate(),
            data_context: RuntimeOwnedDataContext::default(),
        }
    }
}

/// Clone-owned ScriptInput/DataBind/converter state for one lazy keyframe
/// script table.
///
/// This value must be dropped before the table it targets. Its `Drop` path
/// unregisters every retained source and converter operand before releasing
/// clone-owned state, matching `LinearAnimationInstance`'s explicit C++
/// teardown order.
#[doc(hidden)]
#[derive(Debug)]
pub struct RuntimeScriptedInterpolatorBindingOccurrence {
    inner: RuntimeScriptedListenerActionBindingOccurrence,
    data_context: RuntimeOwnedDataContext,
}

/// C++-ordered binding work for the DataBinds cloned with one lazy
/// `ScriptedInterpolator` occurrence.
///
/// The sequence is intentionally occurrence-local. In pinned C++, adding the
/// clone's DataBind to the live Artboard interleaves outer-source binding,
/// nested converter binding/reinitialization, and final-input rebinding before
/// the interpolator table is hydrated.
#[doc(hidden)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuntimeScriptedInterpolatorDataConverterBindStep {
    BindInput {
        input_global_id: u32,
    },
    BindConverter {
        input_global_id: u32,
        converter_path: Vec<usize>,
    },
    Rehydrate {
        input_global_id: u32,
        converter_path: Vec<usize>,
        converter_global_id: u32,
        inits: bool,
    },
    RebindFinalInput {
        input_global_id: u32,
        converter_path: Vec<usize>,
        converter_input_index: usize,
        data_bind_index: usize,
    },
    FinalizeInput {
        input_global_id: u32,
    },
}

impl RuntimeScriptedInterpolatorBindingOccurrence {
    /// Install the concrete Artboard DataContext before walking cloned binds.
    #[doc(hidden)]
    pub fn install_data_context(
        &mut self,
        artboard: &ArtboardInstance,
        fallback_root: Option<&RuntimeOwnedViewModelHandle>,
    ) {
        self.data_context = artboard.scripted_interpolator_owned_data_context(fallback_root);
    }

    #[doc(hidden)]
    pub fn scripted_data_converter_bind_steps(
        &self,
    ) -> Vec<RuntimeScriptedInterpolatorDataConverterBindStep> {
        self.inner
            .scripted_converter_bind_steps()
            .into_iter()
            .map(|step| match step {
                RuntimeScriptedListenerDataConverterBindStep::BindListenerInput {
                    listener_input_global_id,
                    ..
                } => RuntimeScriptedInterpolatorDataConverterBindStep::BindInput {
                    input_global_id: listener_input_global_id,
                },
                RuntimeScriptedListenerDataConverterBindStep::BindConverter {
                    listener_input_global_id,
                    converter_path,
                    ..
                } => RuntimeScriptedInterpolatorDataConverterBindStep::BindConverter {
                    input_global_id: listener_input_global_id,
                    converter_path,
                },
                RuntimeScriptedListenerDataConverterBindStep::Rehydrate {
                    listener_input_global_id,
                    converter_path,
                    converter_global_id,
                    inits,
                    ..
                } => RuntimeScriptedInterpolatorDataConverterBindStep::Rehydrate {
                    input_global_id: listener_input_global_id,
                    converter_path,
                    converter_global_id,
                    inits,
                },
                RuntimeScriptedListenerDataConverterBindStep::RebindFinalInput {
                    listener_input_global_id,
                    converter_path,
                    converter_input_index,
                    data_bind_index,
                    ..
                } => RuntimeScriptedInterpolatorDataConverterBindStep::RebindFinalInput {
                    input_global_id: listener_input_global_id,
                    converter_path,
                    converter_input_index,
                    data_bind_index,
                },
                RuntimeScriptedListenerDataConverterBindStep::FinalizeListenerInput {
                    listener_input_global_id,
                    ..
                } => RuntimeScriptedInterpolatorDataConverterBindStep::FinalizeInput {
                    input_global_id: listener_input_global_id,
                },
            })
            .collect()
    }

    #[doc(hidden)]
    pub fn bind_input_source(&mut self, file: &RuntimeFile, input_global_id: u32) -> bool {
        self.inner.bind_listener_input_source_from_data_context(
            file,
            &self.data_context,
            input_global_id,
            false,
        )
    }

    #[doc(hidden)]
    pub fn bind_converter_own_sources(
        &mut self,
        file: &RuntimeFile,
        input_global_id: u32,
        converter_path: &[usize],
    ) -> bool {
        self.inner
            .bind_converter_own_sources_from_data_context_at_path(
                file,
                &self.data_context,
                input_global_id,
                converter_path,
                false,
            )
    }

    #[doc(hidden)]
    pub fn rebind_scripted_converter_final_input(
        &mut self,
        file: &RuntimeFile,
        input_global_id: u32,
        converter_path: &[usize],
        converter_input_index: usize,
        data_bind_index: usize,
    ) -> bool {
        self.inner.rebind_scripted_converter_final_input(
            file,
            None,
            Some(&self.data_context),
            input_global_id,
            converter_path,
            converter_input_index,
            data_bind_index,
        )
    }

    #[doc(hidden)]
    pub fn finalize_input_sources(&mut self, input_global_id: u32) -> bool {
        self.inner.finalize_listener_input_sources(input_global_id)
    }

    #[doc(hidden)]
    pub fn has_scripted_data_converter_instance(
        &self,
        input_global_id: u32,
        converter_path: &[usize],
    ) -> bool {
        self.inner
            .has_scripted_converter_instance_at_path(input_global_id, converter_path)
    }

    #[doc(hidden)]
    pub fn scripted_data_converter_input_snapshots(
        &self,
        input_global_id: u32,
        converter_path: &[usize],
    ) -> Option<Vec<ScriptListenerInputSnapshot>> {
        self.inner
            .scripted_converter_input_snapshots(input_global_id, converter_path)
    }

    #[doc(hidden)]
    pub fn set_scripted_data_converter_instance(
        &mut self,
        input_global_id: u32,
        converter_path: &[usize],
        converter_global_id: u32,
        instance: Box<dyn ScriptInstance>,
    ) -> Result<(), ScriptError> {
        let handle = RuntimeScriptInstanceHandle::new(instance);
        if !self.inner.attach_scripted_converter_instance_at_path(
            input_global_id,
            converter_path,
            &handle,
        ) {
            return Err(ScriptError::new(format!(
                "ScriptedInterpolator input global {input_global_id} has no ScriptedDataConverter occurrence {converter_path:?} (global {converter_global_id})",
            )));
        }
        Ok(())
    }

    /// Hydrate and initialize one retained converter table at the exact point
    /// where C++ `ScriptedDataConverter::bindFromContext` calls `reinit`.
    #[doc(hidden)]
    pub fn hydrate_and_initialize_scripted_data_converter_instance<F>(
        &mut self,
        input_global_id: u32,
        converter_path: &[usize],
        context: ScriptListenerActionHydration,
        inits: bool,
        prepare_hydration: F,
    ) -> Result<bool, ScriptError>
    where
        F: FnOnce(&Self) -> Result<ScriptListenerActionHydration, ScriptError>,
    {
        let handle = self
            .inner
            .scripted_converter_instance_at_path(input_global_id, converter_path)
            .ok_or_else(|| {
                ScriptError::new(format!(
                    "ScriptedInterpolator input global {input_global_id} has no attached ScriptedDataConverter occurrence {converter_path:?}",
                ))
            })?;
        {
            let mut instance = handle.borrow_mut();
            context.install_context(&mut **instance)?;
            instance.prepare_init_retry()?;
        }
        let hydration = prepare_hydration(self)?;
        let mut instance = handle.borrow_mut();
        hydration.apply_inputs(&mut **instance, &mut NoopScriptHost)?;
        let hydrated = if !inits || !instance.user_init_pending()? {
            true
        } else {
            instance.call_init(&mut NoopScriptHost)?
        };
        drop(instance);
        if hydrated {
            let marked = self
                .inner
                .mark_scripted_converter_hydrated(input_global_id, converter_path);
            debug_assert!(
                marked,
                "the hydrated ScriptedDataConverter must retain its interpolator DataBind"
            );
        }
        Ok(hydrated)
    }

    #[doc(hidden)]
    pub fn input_snapshots(&self) -> Vec<ScriptListenerInputSnapshot> {
        self.inner.input_snapshots()
    }

    /// Hydrate an occurrence whose DataContext sources were already bound by
    /// the ordered converter walk above.
    #[doc(hidden)]
    pub fn hydrate_bound_inputs(
        &mut self,
        file: &RuntimeFile,
        script: &mut dyn ScriptInstance,
    ) -> Result<(), ScriptError> {
        for input in self.inner.input_snapshots() {
            if let Some(ScriptListenerInputSnapshotValue::Value(value)) = input.value {
                script.set_input_core(&input.name, value)?;
            }
        }
        self.refresh_inputs(file, script)
    }

    /// Clone authored properties, bind their occurrence-local DataBinds, and
    /// synchronously project source values through cloned converters. C++
    /// performs the same synchronous update when `cloneProperties` adds a bind
    /// to an Artboard whose DataContext is already live.
    pub fn hydrate_inputs(
        &mut self,
        file: &RuntimeFile,
        artboard: &ArtboardInstance,
        fallback_root: Option<&RuntimeOwnedViewModelHandle>,
        script: &mut dyn ScriptInstance,
    ) -> Result<(), ScriptError> {
        for input in self.inner.input_snapshots() {
            if let Some(ScriptListenerInputSnapshotValue::Value(value)) = input.value {
                script.set_input_core(&input.name, value)?;
            }
        }
        self.install_data_context(artboard, fallback_root);
        self.inner
            .bind_sources_from_data_context(file, &self.data_context, false);
        self.refresh_inputs(file, script)
    }

    /// Apply source changes observed after the clone was hydrated. The source
    /// edges and converter state remain occurrence-local for the lifetime of
    /// the keyframe table.
    pub fn refresh_inputs(
        &mut self,
        file: &RuntimeFile,
        script: &mut dyn ScriptInstance,
    ) -> Result<(), ScriptError> {
        self.inner.collect_source_dirt();
        let updates = self
            .inner
            .resolve_runtime_table_updates_from_data_context(file, &self.data_context)?;
        for update in updates {
            if let RuntimeScriptedListenerBoundValue::Value(value) = update.value {
                script.set_input_core(&update.input_name, value)?;
            }
        }
        Ok(())
    }

    pub fn advance_stateful_converters(
        &mut self,
        elapsed_seconds: f32,
        host: &mut dyn ScriptHost,
    ) -> Result<bool, ScriptError> {
        let advance = self
            .inner
            .advance_stateful_converters(elapsed_seconds, host)?;
        Ok(advance.changed || advance.keep_going)
    }
}

/// Creates the stateful Lua table used by one scripted-interpolator keyframe.
///
/// The runtime facade supplies this factory after authenticating and compiling
/// the referenced ScriptAsset. Each `LinearAnimationInstance` invokes it at
/// most once per keyframe, matching C++ `statefulInterpolator` cloning the
/// shared file prototype into the animation-instance cache.
#[derive(Clone)]
pub struct RuntimeScriptedInterpolatorFactory {
    create: Rc<ScriptedInterpolatorCreate>,
}

type ScriptedInterpolatorCreate =
    dyn Fn(Option<&ArtboardInstance>) -> Result<Box<dyn ScriptInstance>, ScriptError>;

impl RuntimeScriptedInterpolatorFactory {
    pub fn new(
        create: impl Fn(&ArtboardInstance) -> Result<Box<dyn ScriptInstance>, ScriptError> + 'static,
    ) -> Self {
        Self {
            create: Rc::new(move |artboard| {
                create(artboard.ok_or_else(|| {
                    ScriptError::new("scripted interpolator factory has no Artboard occurrence")
                })?)
            }),
        }
    }

    #[cfg(test)]
    fn new_for_test(
        create: impl Fn() -> Result<Box<dyn ScriptInstance>, ScriptError> + 'static,
    ) -> Self {
        Self {
            create: Rc::new(move |_| create()),
        }
    }

    fn create(
        &self,
        artboard: Option<&ArtboardInstance>,
    ) -> Result<Box<dyn ScriptInstance>, ScriptError> {
        (self.create)(artboard)
    }
}

impl fmt::Debug for RuntimeScriptedInterpolatorFactory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RuntimeScriptedInterpolatorFactory")
            .finish_non_exhaustive()
    }
}

/// One fallback-worthy scripted-interpolator failure observed while applying
/// an animation instance.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeScriptedInterpolatorDiagnostic {
    key_frame_global_id: u32,
    interpolator_global_id: u32,
    method: ScriptInterpolatorMethod,
    error: ScriptError,
}

impl RuntimeScriptedInterpolatorDiagnostic {
    pub fn key_frame_global_id(&self) -> u32 {
        self.key_frame_global_id
    }

    pub fn interpolator_global_id(&self) -> u32 {
        self.interpolator_global_id
    }

    pub fn method(&self) -> ScriptInterpolatorMethod {
        self.method
    }

    pub fn error(&self) -> &ScriptError {
        &self.error
    }
}

#[derive(Default)]
pub(crate) struct RuntimeScriptedInterpolatorState {
    // C++ parks clones on the owning Artboard in authored discovery order.
    // Converter advancement is observable, so retain that order explicitly.
    instances: Vec<(u32, Box<dyn ScriptInstance>)>,
    diagnostics: Vec<RuntimeScriptedInterpolatorDiagnostic>,
}

impl fmt::Debug for RuntimeScriptedInterpolatorState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RuntimeScriptedInterpolatorState")
            .field("instance_count", &self.instances.len())
            .field("diagnostics", &self.diagnostics)
            .finish()
    }
}

impl RuntimeScriptedInterpolatorState {
    pub(crate) fn advance_stateful_converters(&mut self, elapsed_seconds: f32) -> bool {
        let mut keep_going = false;
        for (_, instance) in &mut self.instances {
            keep_going |=
                instance.advance_scripted_data_binds(elapsed_seconds, &mut NoopScriptHost);
        }
        keep_going
    }

    pub(crate) fn evaluate(
        &mut self,
        artboard: Option<&ArtboardInstance>,
        factory: Option<&RuntimeScriptedInterpolatorFactory>,
        instance_key: u32,
        key_frame_global_id: u32,
        interpolator_global_id: u32,
        method: ScriptInterpolatorMethod,
        arguments: &[f32],
        fallback: f32,
    ) -> f32 {
        if !self.instances.iter().any(|(key, _)| *key == instance_key) {
            let Some(factory) = factory else {
                self.record(
                    key_frame_global_id,
                    interpolator_global_id,
                    method,
                    ScriptError::new("scripted interpolator has no registered ScriptAsset factory"),
                );
                return fallback;
            };
            match factory.create(artboard) {
                Ok(instance) => {
                    self.instances.push((instance_key, instance));
                }
                Err(error) => {
                    self.record(
                        key_frame_global_id,
                        interpolator_global_id,
                        method,
                        error.with_context("scripted interpolator initialization failed"),
                    );
                    return fallback;
                }
            }
        }

        let result = self
            .instances
            .iter_mut()
            .find_map(|(key, instance)| (*key == instance_key).then_some(instance))
            .expect("scripted interpolator inserted above")
            .call_interpolator(method, arguments, &mut NoopScriptHost);
        match result {
            Ok(ScriptOptionalNumberResult::Returned(value)) => value,
            Ok(ScriptOptionalNumberResult::Missing)
                if method == ScriptInterpolatorMethod::Transform =>
            {
                // C++ treats an absent transform callback as identity.
                fallback
            }
            Ok(ScriptOptionalNumberResult::Missing) => {
                self.record(
                    key_frame_global_id,
                    interpolator_global_id,
                    method,
                    ScriptError::new(format!(
                        "scripted interpolator is missing {} callback",
                        method.as_str()
                    )),
                );
                fallback
            }
            Err(error) => {
                self.record(
                    key_frame_global_id,
                    interpolator_global_id,
                    method,
                    error.with_context(format!(
                        "scripted interpolator {} callback failed",
                        method.as_str()
                    )),
                );
                fallback
            }
        }
    }

    pub(crate) fn diagnostics(&self) -> Vec<RuntimeScriptedInterpolatorDiagnostic> {
        self.diagnostics.clone()
    }

    fn record(
        &mut self,
        key_frame_global_id: u32,
        interpolator_global_id: u32,
        method: ScriptInterpolatorMethod,
        error: ScriptError,
    ) {
        const MAX_DIAGNOSTICS: usize = 64;
        let diagnostic = RuntimeScriptedInterpolatorDiagnostic {
            key_frame_global_id,
            interpolator_global_id,
            method,
            error,
        };
        if self.diagnostics.contains(&diagnostic) {
            return;
        }
        if self.diagnostics.len() == MAX_DIAGNOSTICS {
            self.diagnostics.remove(0);
        }
        self.diagnostics.push(diagnostic);
    }
}

#[cfg(test)]
mod tests {
    use std::cell::{Cell, RefCell};

    use super::*;
    use crate::{ScriptMethod, ScriptOptionalMethodResult, ScriptValue};

    struct TestInstance {
        calls: Rc<Cell<u32>>,
        result: Result<ScriptOptionalNumberResult, ScriptError>,
    }

    struct AdvancingInstance {
        clone_ordinal: usize,
        advances: Rc<RefCell<Vec<(usize, f32)>>>,
    }

    impl ScriptInstance for AdvancingInstance {
        fn advance_scripted_data_binds(
            &mut self,
            elapsed_seconds: f32,
            _host: &mut dyn ScriptHost,
        ) -> bool {
            self.advances
                .borrow_mut()
                .push((self.clone_ordinal, elapsed_seconds));
            true
        }

        fn has_method(&self, _method: ScriptMethod) -> Result<bool, ScriptError> {
            Ok(true)
        }

        fn call_method(
            &mut self,
            _method: ScriptMethod,
            _args: &[ScriptValue],
            _host: &mut dyn ScriptHost,
        ) -> Result<ScriptValue, ScriptError> {
            Ok(ScriptValue::Nil)
        }

        fn get_input(&self, _name: &str) -> Result<ScriptValue, ScriptError> {
            Ok(ScriptValue::Nil)
        }

        fn set_input(&mut self, _name: &str, _value: ScriptValue) -> Result<(), ScriptError> {
            Ok(())
        }
    }

    impl ScriptInstance for TestInstance {
        fn has_method(&self, _method: ScriptMethod) -> Result<bool, ScriptError> {
            Ok(true)
        }

        fn call_method(
            &mut self,
            _method: ScriptMethod,
            _args: &[ScriptValue],
            _host: &mut dyn crate::ScriptHost,
        ) -> Result<ScriptValue, ScriptError> {
            Ok(ScriptValue::Number(0.0))
        }

        fn call_optional_method(
            &mut self,
            _method: ScriptMethod,
            _args: &[ScriptValue],
            _host: &mut dyn crate::ScriptHost,
        ) -> Result<ScriptOptionalMethodResult, ScriptError> {
            Ok(ScriptOptionalMethodResult::Missing)
        }

        fn call_interpolator(
            &mut self,
            _method: ScriptInterpolatorMethod,
            _args: &[f32],
            _host: &mut dyn crate::ScriptHost,
        ) -> Result<ScriptOptionalNumberResult, ScriptError> {
            self.calls.set(self.calls.get() + 1);
            self.result.clone()
        }

        fn get_input(&self, _name: &str) -> Result<ScriptValue, ScriptError> {
            Ok(ScriptValue::Number(0.0))
        }

        fn set_input(&mut self, _name: &str, _value: ScriptValue) -> Result<(), ScriptError> {
            Ok(())
        }
    }

    #[test]
    fn caches_one_script_table_per_keyframe() {
        let creations = Rc::new(Cell::new(0));
        let calls = Rc::new(Cell::new(0));
        let factory = RuntimeScriptedInterpolatorFactory::new_for_test({
            let creations = Rc::clone(&creations);
            let calls = Rc::clone(&calls);
            move || {
                creations.set(creations.get() + 1);
                Ok(Box::new(TestInstance {
                    calls: Rc::clone(&calls),
                    result: Ok(ScriptOptionalNumberResult::Returned(0.25)),
                }))
            }
        });
        let mut state = RuntimeScriptedInterpolatorState::default();

        assert_eq!(
            state.evaluate(
                None,
                Some(&factory),
                10,
                10,
                3,
                ScriptInterpolatorMethod::Transform,
                &[0.5],
                0.5,
            ),
            0.25
        );
        assert_eq!(
            state.evaluate(
                None,
                Some(&factory),
                10,
                10,
                3,
                ScriptInterpolatorMethod::Transform,
                &[0.75],
                0.75,
            ),
            0.25
        );
        state.evaluate(
            None,
            Some(&factory),
            11,
            11,
            3,
            ScriptInterpolatorMethod::Transform,
            &[0.5],
            0.5,
        );

        assert_eq!(creations.get(), 2);
        assert_eq!(calls.get(), 3);
    }

    #[test]
    fn advances_new_lazy_keyframe_clones_once_in_authored_order() {
        let advances = Rc::new(RefCell::new(Vec::new()));
        let next_ordinal = Rc::new(Cell::new(0));
        let factory = RuntimeScriptedInterpolatorFactory::new_for_test({
            let advances = Rc::clone(&advances);
            let next_ordinal = Rc::clone(&next_ordinal);
            move || {
                let clone_ordinal = next_ordinal.get();
                next_ordinal.set(clone_ordinal + 1);
                Ok(Box::new(AdvancingInstance {
                    clone_ordinal,
                    advances: Rc::clone(&advances),
                }))
            }
        });
        let mut state = RuntimeScriptedInterpolatorState::default();
        for keyframe in [10, 11] {
            state.evaluate(
                None,
                Some(&factory),
                keyframe,
                keyframe,
                3,
                ScriptInterpolatorMethod::Transform,
                &[0.5],
                0.5,
            );
        }

        assert!(state.advance_stateful_converters(0.25));
        assert_eq!(
            *advances.borrow(),
            [(0, 0.25), (1, 0.25)],
            "new per-keyframe clones advance this frame in authored discovery order",
        );
    }

    #[test]
    fn missing_and_erroring_transform_value_callbacks_fall_back_with_diagnostics() {
        let missing = RuntimeScriptedInterpolatorFactory::new_for_test(|| {
            Ok(Box::new(TestInstance {
                calls: Rc::new(Cell::new(0)),
                result: Ok(ScriptOptionalNumberResult::Missing),
            }))
        });
        let erroring = RuntimeScriptedInterpolatorFactory::new_for_test(|| {
            Ok(Box::new(TestInstance {
                calls: Rc::new(Cell::new(0)),
                result: Err(ScriptError::new("boom")),
            }))
        });
        let mut state = RuntimeScriptedInterpolatorState::default();

        assert_eq!(
            state.evaluate(
                None,
                Some(&missing),
                9,
                9,
                3,
                ScriptInterpolatorMethod::Transform,
                &[0.5],
                0.5,
            ),
            0.5
        );
        assert!(
            state.diagnostics().is_empty(),
            "C++ silently treats an absent transform callback as identity"
        );
        assert_eq!(
            state.evaluate(
                None,
                Some(&missing),
                10,
                10,
                3,
                ScriptInterpolatorMethod::TransformValue,
                &[10.0, 30.0, 0.5],
                20.0,
            ),
            20.0
        );
        assert_eq!(
            state.evaluate(
                None,
                Some(&erroring),
                11,
                11,
                4,
                ScriptInterpolatorMethod::TransformValue,
                &[10.0, 30.0, 0.5],
                20.0,
            ),
            20.0
        );
        assert_eq!(
            state.evaluate(
                None,
                Some(&erroring),
                11,
                11,
                4,
                ScriptInterpolatorMethod::TransformValue,
                &[10.0, 30.0, 0.5],
                20.0,
            ),
            20.0
        );

        let diagnostics = state.diagnostics();
        assert_eq!(diagnostics.len(), 2);
        assert!(
            diagnostics[0]
                .error()
                .message()
                .contains("missing transformValue")
        );
        assert!(diagnostics[1].error().message().contains("boom"));
    }

    #[test]
    fn diagnostics_are_deduplicated_and_bounded() {
        let mut state = RuntimeScriptedInterpolatorState::default();
        for id in 0..100 {
            state.record(
                id,
                3,
                ScriptInterpolatorMethod::TransformValue,
                ScriptError::new(format!("failure {id}")),
            );
        }
        let diagnostics = state.diagnostics();
        assert_eq!(diagnostics.len(), 64);
        assert_eq!(diagnostics[0].key_frame_global_id(), 36);

        state.record(
            99,
            3,
            ScriptInterpolatorMethod::TransformValue,
            ScriptError::new("failure 99"),
        );
        assert_eq!(state.diagnostics().len(), 64);
    }
}
