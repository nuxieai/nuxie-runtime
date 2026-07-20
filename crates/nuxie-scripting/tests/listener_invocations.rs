#![cfg(feature = "luau")]

use luaur_rt::Table;
use nuxie_runtime::{
    NoopScriptHost, ScriptInstance, ScriptListenerActionMethod, ScriptListenerInvocation,
    ScriptPointerEventKind, ScriptValue,
};
use nuxie_scripting::vm::ScriptVm;

#[test]
fn perform_action_receives_pointer_invocation_userdata() {
    let vm = ScriptVm::new();
    let table: Table = vm
        .eval(
            r#"
                return {
                    performAction = function(self, invocation)
                        self.isPointer = invocation:isPointerEvent()
                        self.isReported = invocation:isReportedEvent()
                        self.isNone = invocation:isNone()
                        local pointer = invocation:asPointerEvent()
                        self.pointerId = pointer.id
                        self.x = pointer.position.x
                        self.y = pointer.position.y
                        self.previousX = pointer.previousPosition.x
                        self.previousY = pointer.previousPosition.y
                        self.eventType = pointer.type
                        self.timeStamp = pointer.timeStamp
                    end,
                }
            "#,
        )
        .unwrap();
    let mut instance = vm.script_instance_from_table(table);

    instance
        .call_listener_action(
            ScriptListenerActionMethod::PerformAction,
            &ScriptListenerInvocation::Pointer {
                x: 12.5,
                y: 34.25,
                previous_x: 8.0,
                previous_y: 13.0,
                pointer_id: 7,
                event: ScriptPointerEventKind::Click,
                timestamp_seconds: 42.75,
            },
            &mut NoopScriptHost,
        )
        .unwrap();

    assert_eq!(
        instance.get_input("isPointer").unwrap(),
        ScriptValue::Bool(true)
    );
    assert_eq!(
        instance.get_input("isReported").unwrap(),
        ScriptValue::Bool(false)
    );
    assert_eq!(
        instance.get_input("isNone").unwrap(),
        ScriptValue::Bool(false)
    );
    assert_eq!(
        instance.get_input("pointerId").unwrap(),
        ScriptValue::Number(7.0)
    );
    assert_eq!(instance.get_input("x").unwrap(), ScriptValue::Number(12.5));
    assert_eq!(instance.get_input("y").unwrap(), ScriptValue::Number(34.25));
    assert_eq!(
        instance.get_input("previousX").unwrap(),
        ScriptValue::Number(8.0)
    );
    assert_eq!(
        instance.get_input("previousY").unwrap(),
        ScriptValue::Number(13.0)
    );
    assert_eq!(
        instance.get_input("eventType").unwrap(),
        ScriptValue::String("click".to_owned())
    );
    assert_eq!(
        instance.get_input("timeStamp").unwrap(),
        ScriptValue::Number(42.75)
    );
}

#[test]
fn perform_action_distinguishes_reported_event_and_none_invocations() {
    let vm = ScriptVm::new();
    let table: Table = vm
        .eval(
            r#"
                return {
                    performAction = function(self, invocation)
                        self.isReported = invocation:isReportedEvent()
                        self.isNone = invocation:isNone()
                        self.pointerIsNil = invocation:asPointerEvent() == nil
                        self.keyboardIsFalse = not invocation:isKeyboardEvent()
                        self.keyboardIsNil = invocation:asKeyboardEvent() == nil
                        local reported = invocation:asReportedEvent()
                        self.delay = reported == nil and -1 or reported.delaySeconds
                        self.noneIsPresent = invocation:asNone() ~= nil
                    end,
                }
            "#,
        )
        .unwrap();
    let mut instance = vm.script_instance_from_table(table);

    instance
        .call_listener_action(
            ScriptListenerActionMethod::PerformAction,
            &ScriptListenerInvocation::ReportedEvent {
                event_local_index: 3,
                seconds_delay: 0.75,
            },
            &mut NoopScriptHost,
        )
        .unwrap();
    assert_eq!(
        instance.get_input("isReported").unwrap(),
        ScriptValue::Bool(true)
    );
    assert_eq!(
        instance.get_input("isNone").unwrap(),
        ScriptValue::Bool(false)
    );
    assert_eq!(
        instance.get_input("pointerIsNil").unwrap(),
        ScriptValue::Bool(true)
    );
    assert_eq!(
        instance.get_input("keyboardIsFalse").unwrap(),
        ScriptValue::Bool(true)
    );
    assert_eq!(
        instance.get_input("keyboardIsNil").unwrap(),
        ScriptValue::Bool(true)
    );
    assert_eq!(
        instance.get_input("delay").unwrap(),
        ScriptValue::Number(0.75)
    );
    assert_eq!(
        instance.get_input("noneIsPresent").unwrap(),
        ScriptValue::Bool(false)
    );

    instance
        .call_listener_action(
            ScriptListenerActionMethod::PerformAction,
            &ScriptListenerInvocation::None,
            &mut NoopScriptHost,
        )
        .unwrap();
    assert_eq!(
        instance.get_input("isReported").unwrap(),
        ScriptValue::Bool(false)
    );
    assert_eq!(
        instance.get_input("isNone").unwrap(),
        ScriptValue::Bool(true)
    );
    assert_eq!(
        instance.get_input("delay").unwrap(),
        ScriptValue::Number(-1.0)
    );
    assert_eq!(
        instance.get_input("noneIsPresent").unwrap(),
        ScriptValue::Bool(true)
    );
}

#[test]
fn legacy_perform_receives_a_pointer_event_or_the_upstream_placeholder() {
    let vm = ScriptVm::new();
    let table: Table = vm
        .eval(
            r#"
                return {
                    perform = function(self, pointer)
                        self.pointerId = pointer.id
                        self.x = pointer.position.x
                        self.y = pointer.position.y
                        self.eventType = pointer.type
                    end,
                }
            "#,
        )
        .unwrap();
    let mut instance = vm.script_instance_from_table(table);

    instance
        .call_listener_action(
            ScriptListenerActionMethod::Perform,
            &ScriptListenerInvocation::Pointer {
                x: 8.0,
                y: 13.0,
                previous_x: 8.0,
                previous_y: 13.0,
                pointer_id: 5,
                event: ScriptPointerEventKind::Drag,
                timestamp_seconds: 0.0,
            },
            &mut NoopScriptHost,
        )
        .unwrap();
    assert_eq!(
        instance.get_input("pointerId").unwrap(),
        ScriptValue::Number(5.0)
    );
    assert_eq!(instance.get_input("x").unwrap(), ScriptValue::Number(8.0));
    assert_eq!(instance.get_input("y").unwrap(), ScriptValue::Number(13.0));
    assert_eq!(
        instance.get_input("eventType").unwrap(),
        ScriptValue::String("pointerDrag".to_owned())
    );

    instance
        .call_listener_action(
            ScriptListenerActionMethod::Perform,
            &ScriptListenerInvocation::None,
            &mut NoopScriptHost,
        )
        .unwrap();
    assert_eq!(
        instance.get_input("pointerId").unwrap(),
        ScriptValue::Number(0.0)
    );
    assert_eq!(instance.get_input("x").unwrap(), ScriptValue::Number(0.0));
    assert_eq!(instance.get_input("y").unwrap(), ScriptValue::Number(0.0));
    assert_eq!(
        instance.get_input("eventType").unwrap(),
        ScriptValue::String("unknown".to_owned())
    );
}
