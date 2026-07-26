//! Scripted transition-condition definition and occurrence dispatch.
//!
//! Mirrors pinned C++ `src/animation/scripted_transition_condition.cpp`.

use super::RuntimeScheduledListenerActionExecutor;
use crate::scripting::RuntimeScriptInstanceHandle;
use crate::{NoopScriptHost, ScriptMethod, ScriptValue};
use nuxie_binary::RuntimeObject;
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy)]
pub(super) struct RuntimeScriptedTransitionCondition {
    global_id: u32,
}

impl RuntimeScriptedTransitionCondition {
    pub(super) fn from_object(object: &RuntimeObject) -> Self {
        Self::new(object.id)
    }

    pub(super) fn new(global_id: u32) -> Self {
        Self { global_id }
    }

    pub(super) fn evaluate(&self, executor: &dyn RuntimeScheduledListenerActionExecutor) -> bool {
        // Pinned C++ resolves the per-instance scripted clone before invoking
        // `evaluateStateful`; the executor owns that occurrence lookup.
        executor.evaluate_scripted_condition(self.global_id)
    }
}

pub(super) fn evaluate_scripted_condition(
    global_id: u32,
    scripted_instances: &BTreeMap<u32, RuntimeScriptInstanceHandle>,
) -> bool {
    scripted_instances
        .get(&global_id)
        .and_then(|instance| {
            instance
                .borrow_mut()
                .call_method(ScriptMethod::Evaluate, &[], &mut NoopScriptHost)
                .ok()
        })
        .is_some_and(|value| value == ScriptValue::Bool(true))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ScriptError, ScriptHost, ScriptInstance};

    struct ConditionScript {
        result: Result<ScriptValue, ScriptError>,
    }

    impl ScriptInstance for ConditionScript {
        fn has_method(&self, method: ScriptMethod) -> Result<bool, ScriptError> {
            Ok(method == ScriptMethod::Evaluate)
        }

        fn call_method(
            &mut self,
            method: ScriptMethod,
            _args: &[ScriptValue],
            _host: &mut dyn ScriptHost,
        ) -> Result<ScriptValue, ScriptError> {
            assert_eq!(method, ScriptMethod::Evaluate);
            self.result.clone()
        }

        fn get_input(&self, _name: &str) -> Result<ScriptValue, ScriptError> {
            Ok(ScriptValue::Nil)
        }

        fn set_input(&mut self, _name: &str, _value: ScriptValue) -> Result<(), ScriptError> {
            Ok(())
        }
    }

    fn instances_with(
        result: Result<ScriptValue, ScriptError>,
    ) -> BTreeMap<u32, RuntimeScriptInstanceHandle> {
        BTreeMap::from([(
            7,
            RuntimeScriptInstanceHandle::new(Box::new(ConditionScript { result })),
        )])
    }

    #[test]
    fn scripted_transition_requires_an_exact_true_boolean() {
        assert!(evaluate_scripted_condition(
            7,
            &instances_with(Ok(ScriptValue::Bool(true)))
        ));
        assert!(!evaluate_scripted_condition(
            7,
            &instances_with(Ok(ScriptValue::Bool(false)))
        ));
        assert!(!evaluate_scripted_condition(
            7,
            &instances_with(Ok(ScriptValue::Number(1.0)))
        ));
        assert!(!evaluate_scripted_condition(
            7,
            &instances_with(Err(ScriptError::new("evaluate failed")))
        ));
        assert!(!evaluate_scripted_condition(7, &BTreeMap::new()));
    }
}
