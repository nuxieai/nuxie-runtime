use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;

use crate::{
    ArtboardInstance, NoopScriptHost, ScriptError, ScriptInstance, ScriptInterpolatorMethod,
    ScriptOptionalNumberResult,
};

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
    instances: HashMap<u32, Box<dyn ScriptInstance>>,
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
        if !self.instances.contains_key(&instance_key) {
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
                    self.instances.insert(instance_key, instance);
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
            .get_mut(&instance_key)
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
    use std::cell::Cell;

    use super::*;
    use crate::{ScriptMethod, ScriptOptionalMethodResult, ScriptValue};

    struct TestInstance {
        calls: Rc<Cell<u32>>,
        result: Result<ScriptOptionalNumberResult, ScriptError>,
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
