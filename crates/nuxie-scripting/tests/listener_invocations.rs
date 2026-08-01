#![cfg(feature = "luau")]

use luaur_rt::Table;
use nuxie_runtime::{
    NoopScriptHost, ScriptGamepadInputChange, ScriptGamepadMappingKind, ScriptGamepadSnapshot,
    ScriptInstance, ScriptListenerActionMethod, ScriptListenerInvocation, ScriptMethod,
    ScriptPointerEventKind, ScriptValue, ScriptedDrawablePointerHit,
};
use nuxie_scripting::vm::ScriptVm;

fn call_listener(instance: &mut impl ScriptInstance, invocation: ScriptListenerInvocation) {
    instance
        .call_listener_action(
            ScriptListenerActionMethod::PerformAction,
            &invocation,
            &mut NoopScriptHost,
        )
        .unwrap();
}

#[test]
fn pointer_event_can_be_constructed_with_the_pinned_defaults() {
    let vm = ScriptVm::new();
    vm.install_rive_globals().unwrap();

    let (id, event_type, previous_x, previous_y, timestamp): (u32, String, f32, f32, f32) = vm
        .eval(
            r#"
                local origin = PointerEvent.new(4, Vector.origin())
                return origin.id,
                    origin.type,
                    origin.previousPosition.x,
                    origin.previousPosition.y,
                    origin.timeStamp
            "#,
        )
        .unwrap();

    assert_eq!(id, 4);
    assert_eq!(event_type, "pointerEnter");
    assert_eq!((previous_x, previous_y), (0.0, 0.0));
    assert_eq!(timestamp, 0.0);
}

#[test]
fn pointer_event_can_be_constructed_with_a_specified_position() {
    let vm = ScriptVm::new();
    vm.install_rive_globals().unwrap();

    let (id, x, y): (u32, f32, f32) = vm
        .eval(
            r#"
                local event = PointerEvent.new(260, Vector.xy(11, 12))
                return event.id, event.position.x, event.position.y
            "#,
        )
        .unwrap();

    assert_eq!(id, 4);
    assert_eq!((x, y), (11.0, 12.0));
}

#[test]
fn scripted_drawable_pointer_callback_returns_the_lua_hit_result() {
    let vm = ScriptVm::new();
    let table: Table = vm
        .eval(
            r#"
                return {
                    pointerDown = function(self, event)
                        self.id = event.id
                        self.x = event.position.x
                        self.y = event.position.y
                        self.previousX = event.previousPosition.x
                        self.eventType = event.type
                        self.timeStamp = event.timeStamp
                        event:hit(true)
                    end,
                }
            "#,
        )
        .unwrap();
    let mut instance = vm.script_instance_from_table(table);

    let result = instance
        .call_scripted_drawable_pointer(
            ScriptMethod::PointerDown,
            260,
            11.0,
            12.0,
            &mut NoopScriptHost,
        )
        .unwrap();

    assert!(result.invoked);
    assert_eq!(result.hit, ScriptedDrawablePointerHit::Hit);
    assert_eq!(instance.get_input("id").unwrap(), ScriptValue::Number(4.0));
    assert_eq!(instance.get_input("x").unwrap(), ScriptValue::Number(11.0));
    assert_eq!(instance.get_input("y").unwrap(), ScriptValue::Number(12.0));
    assert_eq!(
        instance.get_input("previousX").unwrap(),
        ScriptValue::Number(0.0)
    );
    assert_eq!(
        instance.get_input("eventType").unwrap(),
        ScriptValue::String("pointerEnter".to_owned())
    );
    assert_eq!(
        instance.get_input("timeStamp").unwrap(),
        ScriptValue::Number(0.0)
    );
}

#[test]
fn text_and_gamepad_intent_payloads_match_cpp_c_string_and_unknown_enum_semantics() {
    let vm = ScriptVm::new();
    let table: Table = vm
        .eval(
            r#"
                return {
                    performAction = function(self, invocation)
                        if invocation:isTextInput() then
                            self.text = invocation:asTextInput().text
                        elseif invocation:isGamepadEvent() then
                            local event = invocation:asGamepadEvent()
                            self.intentButton = event.intentButton
                            self.intentAxis = event.intentAxis
                            self.hasButton = event.hasStandardButtonIntent
                            self.hasAxis = event.hasStandardAxisIntent
                        end
                    end,
                }
            "#,
        )
        .unwrap();
    let mut instance = vm.script_instance_from_table(table);

    instance
        .call_listener_action(
            ScriptListenerActionMethod::PerformAction,
            &ScriptListenerInvocation::TextInput {
                text: "before\0after".to_owned(),
            },
            &mut NoopScriptHost,
        )
        .unwrap();
    assert_eq!(
        instance.get_input("text").unwrap(),
        ScriptValue::String("before".to_owned())
    );

    instance
        .call_listener_action(
            ScriptListenerActionMethod::PerformAction,
            &ScriptListenerInvocation::GamepadEvent {
                full_state: ScriptGamepadSnapshot {
                    device_id: 1,
                    button_mask: 0,
                    button_values: Vec::new(),
                    axes: Vec::new(),
                    mapping: ScriptGamepadMappingKind::Standard,
                },
                change: ScriptGamepadInputChange::Button {
                    index: 0,
                    value: 1.0,
                },
                standard_button_intent: Some(99),
                standard_axis_intent: Some(99),
            },
            &mut NoopScriptHost,
        )
        .unwrap();
    assert_eq!(
        instance.get_input("intentButton").unwrap(),
        ScriptValue::String("unknown".to_owned())
    );
    assert_eq!(
        instance.get_input("intentAxis").unwrap(),
        ScriptValue::String("unknown".to_owned())
    );
    assert_eq!(
        instance.get_input("hasButton").unwrap(),
        ScriptValue::Bool(true)
    );
    assert_eq!(
        instance.get_input("hasAxis").unwrap(),
        ScriptValue::Bool(true)
    );
}

#[test]
fn keyboard_focus_reported_view_model_and_none_payload_scenario_matches_cpp() {
    let vm = ScriptVm::new();
    let table: Table = vm
        .eval(
            r#"
                return {
                    performAction = function(self, invocation)
                        self.isPointer = invocation:isPointerEvent()
                        self.isKeyboard = invocation:isKeyboardEvent()
                        self.isText = invocation:isTextInput()
                        self.isFocus = invocation:isFocus()
                        self.isReported = invocation:isReportedEvent()
                        self.isViewModel = invocation:isViewModelChange()
                        self.isNone = invocation:isNone()
                        self.isConnected = invocation:isGamepadConnected()
                        self.isGamepadEvent = invocation:isGamepadEvent()
                        self.isDisconnected = invocation:isGamepadDisconnected()
                        self.kindCount =
                            (self.isPointer and 1 or 0) +
                            (self.isKeyboard and 1 or 0) +
                            (self.isText and 1 or 0) +
                            (self.isFocus and 1 or 0) +
                            (self.isReported and 1 or 0) +
                            (self.isViewModel and 1 or 0) +
                            (self.isNone and 1 or 0) +
                            (self.isConnected and 1 or 0) +
                            (self.isGamepadEvent and 1 or 0) +
                            (self.isDisconnected and 1 or 0)
                        self.payloadCount =
                            (invocation:asPointerEvent() ~= nil and 1 or 0) +
                            (invocation:asKeyboardEvent() ~= nil and 1 or 0) +
                            (invocation:asTextInput() ~= nil and 1 or 0) +
                            (invocation:asFocus() ~= nil and 1 or 0) +
                            (invocation:asReportedEvent() ~= nil and 1 or 0) +
                            (invocation:asViewModelChange() ~= nil and 1 or 0) +
                            (invocation:asNone() ~= nil and 1 or 0) +
                            (invocation:asGamepadConnected() ~= nil and 1 or 0) +
                            (invocation:asGamepadEvent() ~= nil and 1 or 0) +
                            (invocation:asGamepadDisconnected() ~= nil and 1 or 0)
                        if self.isKeyboard then
                            local event = invocation:asKeyboardEvent()
                            self.key = event.key
                            self.shift = event.shift
                            self.control = event.control
                            self.alt = event.alt
                            self.meta = event.meta
                            self.phase = event.phase
                        elseif self.isFocus then
                            self.focus = invocation:asFocus().isFocus
                        elseif self.isReported then
                            self.delay = invocation:asReportedEvent().delaySeconds
                        elseif self.isViewModel then
                            self.hasViewModelPayload = invocation:asViewModelChange() ~= nil
                        elseif self.isNone then
                            self.hasNonePayload = invocation:asNone() ~= nil
                        end
                    end,
                }
            "#,
        )
        .unwrap();
    let mut instance = vm.script_instance_from_table(table);

    call_listener(
        &mut instance,
        ScriptListenerInvocation::Keyboard {
            key: 66,
            modifiers: 0b1111,
            is_pressed: true,
            is_repeat: true,
        },
    );
    assert_eq!(
        instance.get_input("key").unwrap(),
        ScriptValue::Number(66.0)
    );
    for field in ["shift", "control", "alt", "meta"] {
        assert_eq!(instance.get_input(field).unwrap(), ScriptValue::Bool(true));
    }
    assert_eq!(
        instance.get_input("phase").unwrap(),
        ScriptValue::String("repeat".to_owned())
    );
    assert_eq!(
        instance.get_input("kindCount").unwrap(),
        ScriptValue::Number(1.0)
    );
    assert_eq!(
        instance.get_input("payloadCount").unwrap(),
        ScriptValue::Number(1.0)
    );

    call_listener(
        &mut instance,
        ScriptListenerInvocation::Keyboard {
            key: 65,
            modifiers: 0,
            is_pressed: false,
            is_repeat: true,
        },
    );
    assert_eq!(
        instance.get_input("phase").unwrap(),
        ScriptValue::String("up".to_owned())
    );

    call_listener(
        &mut instance,
        ScriptListenerInvocation::Focus {
            listener_index: 17,
            is_focus: true,
        },
    );
    assert_eq!(
        instance.get_input("focus").unwrap(),
        ScriptValue::Bool(true)
    );

    call_listener(
        &mut instance,
        ScriptListenerInvocation::ReportedEvent {
            event_local_index: 4,
            seconds_delay: 0.75,
        },
    );
    assert_eq!(
        instance.get_input("delay").unwrap(),
        ScriptValue::Number(0.75)
    );

    call_listener(
        &mut instance,
        ScriptListenerInvocation::ViewModelChange { listener_index: 9 },
    );
    assert_eq!(
        instance.get_input("hasViewModelPayload").unwrap(),
        ScriptValue::Bool(true)
    );

    call_listener(&mut instance, ScriptListenerInvocation::None);
    assert_eq!(
        instance.get_input("hasNonePayload").unwrap(),
        ScriptValue::Bool(true)
    );
    assert_eq!(
        instance.get_input("isNone").unwrap(),
        ScriptValue::Bool(true)
    );
    assert_eq!(
        instance.get_input("isKeyboard").unwrap(),
        ScriptValue::Bool(false)
    );
    assert_eq!(
        instance.get_input("isGamepadEvent").unwrap(),
        ScriptValue::Bool(false)
    );
}

#[test]
fn scripted_listener_context_fixture_payload_sequence_matches_upstream_oracle() {
    let vm = ScriptVm::new();
    let table: Table = vm
        .eval(
            r#"
                return {
                    performAction = function(self, invocation)
                        if invocation:isPointerEvent() then
                            local event = invocation:asPointerEvent()
                            self.pointerType = event.type
                            self.posX = event.position.x
                            self.posY = event.position.y
                        elseif invocation:isFocus() then
                            self.focused = invocation:asFocus().isFocus
                        elseif invocation:isKeyboardEvent() then
                            local event = invocation:asKeyboardEvent()
                            self.keyInput = string.format(
                                "%d, %s shift, %s meta, %s control, %s alt, phase: %s",
                                event.key,
                                event.shift and "with" or "no",
                                event.meta and "with" or "no",
                                event.control and "with" or "no",
                                event.alt and "with" or "no",
                                event.phase
                            )
                        elseif invocation:isTextInput() then
                            self.stringInput = invocation:asTextInput().text
                        elseif invocation:isReportedEvent() then
                            self.eventReported = invocation:asReportedEvent().delaySeconds == 0.25
                        elseif invocation:isViewModelChange() then
                            self.viewModelChanged = invocation:asViewModelChange() ~= nil
                        end
                    end,
                }
            "#,
        )
        .unwrap();
    let mut instance = vm.script_instance_from_table(table);

    // This is the payload sequence from the pinned
    // `scripted_listener_context.riv` TEST_CASE, kept at the binding seam so
    // failures identify the Lua wrapper rather than renderer silver output.
    for invocation in [
        ScriptListenerInvocation::Pointer {
            x: 200.0,
            y: 210.0,
            previous_x: 0.0,
            previous_y: 0.0,
            pointer_id: 0,
            event: ScriptPointerEventKind::Enter,
            timestamp_seconds: 0.0,
        },
        ScriptListenerInvocation::Pointer {
            x: 250.0,
            y: 251.0,
            previous_x: 200.0,
            previous_y: 210.0,
            pointer_id: 0,
            event: ScriptPointerEventKind::Click,
            timestamp_seconds: 0.0,
        },
        ScriptListenerInvocation::Focus {
            listener_index: 0,
            is_focus: true,
        },
        ScriptListenerInvocation::Keyboard {
            key: 65,
            modifiers: 0,
            is_pressed: false,
            is_repeat: false,
        },
        ScriptListenerInvocation::TextInput {
            text: "With text input".to_owned(),
        },
        ScriptListenerInvocation::Keyboard {
            key: 66,
            modifiers: 0b1001,
            is_pressed: true,
            is_repeat: false,
        },
        ScriptListenerInvocation::Focus {
            listener_index: 0,
            is_focus: false,
        },
        ScriptListenerInvocation::ReportedEvent {
            event_local_index: 0,
            seconds_delay: 0.25,
        },
        ScriptListenerInvocation::ViewModelChange { listener_index: 0 },
    ] {
        call_listener(&mut instance, invocation);
    }

    assert_eq!(
        instance.get_input("pointerType").unwrap(),
        ScriptValue::String("click".to_owned())
    );
    assert_eq!(
        instance.get_input("posX").unwrap(),
        ScriptValue::Number(250.0)
    );
    assert_eq!(
        instance.get_input("posY").unwrap(),
        ScriptValue::Number(251.0)
    );
    assert_eq!(
        instance.get_input("keyInput").unwrap(),
        ScriptValue::String(
            "66, with shift, with meta, no control, no alt, phase: down".to_owned()
        )
    );
    assert_eq!(
        instance.get_input("stringInput").unwrap(),
        ScriptValue::String("With text input".to_owned())
    );
    assert_eq!(
        instance.get_input("focused").unwrap(),
        ScriptValue::Bool(false)
    );
    assert_eq!(
        instance.get_input("eventReported").unwrap(),
        ScriptValue::Bool(true)
    );
    assert_eq!(
        instance.get_input("viewModelChanged").unwrap(),
        ScriptValue::Bool(true)
    );
}

#[test]
fn listener_action_fixture_pointer_sequences_match_upstream_oracles() {
    let vm = ScriptVm::new();
    let table: Table = vm
        .eval(
            r#"
                return {
                    amount = 5,
                    label = "authored",
                    performAction = function(self, invocation)
                        local event = invocation:asPointerEvent()
                        local item = string.format(
                            "%d@%d,%d:%s:%d:%s",
                            event.id,
                            event.position.x,
                            event.position.y,
                            event.type,
                            self.amount,
                            self.label
                        )
                        self.trace = self.trace == nil and item or self.trace .. ";" .. item
                    end,
                }
            "#,
        )
        .unwrap();
    let mut instance = vm.script_instance_from_table(table);

    // `scripted_listener_action.riv` clicks x=200/300/400 with ids 1/2/3.
    for (pointer_id, x) in [(1, 200.0), (2, 300.0), (3, 400.0)] {
        call_listener(
            &mut instance,
            ScriptListenerInvocation::Pointer {
                x,
                y: 20.0,
                previous_x: x,
                previous_y: 20.0,
                pointer_id,
                event: ScriptPointerEventKind::Click,
                timestamp_seconds: 0.016,
            },
        );
    }
    // `listener_action_inputs.riv` uses pointer id 3 at the artboard center;
    // this extra call pins that event together with hydrated scalar/string
    // inputs, which are owned by the listener table before callback dispatch.
    call_listener(
        &mut instance,
        ScriptListenerInvocation::Pointer {
            x: 50.0,
            y: 50.0,
            previous_x: 50.0,
            previous_y: 50.0,
            pointer_id: 3,
            event: ScriptPointerEventKind::Click,
            timestamp_seconds: 0.016,
        },
    );

    assert_eq!(
        instance.get_input("trace").unwrap(),
        ScriptValue::String(
            "1@200,20:click:5:authored;2@300,20:click:5:authored;3@400,20:click:5:authored;3@50,50:click:5:authored"
                .to_owned()
        )
    );
}

#[test]
fn gamepad_snapshot_and_change_payload_scenario_matches_cpp() {
    let vm = ScriptVm::new();
    let table: Table = vm
        .eval(
            r#"
                local function readSnapshot(self, gamepad)
                    self.deviceId = gamepad.deviceId
                    self.buttonMask = gamepad.buttonMask
                    self.button1 = gamepad.buttons[1]
                    self.button2 = gamepad.buttons[2]
                    self.axis1 = gamepad.axes[1]
                    self.axis6 = gamepad.axes[6]
                    self.gamepadMapping = gamepad.gamepadMapping
                    self.mapping = gamepad.mapping
                    self.isStandardMapping = gamepad.isStandardMapping
                    self.standardButtonCount =
                        (gamepad.south and 1 or 0) +
                        (gamepad.east and 1 or 0) +
                        (gamepad.west and 1 or 0) +
                        (gamepad.north and 1 or 0) +
                        (gamepad.leftShoulder and 1 or 0) +
                        (gamepad.rightShoulder and 1 or 0) +
                        (gamepad.leftTriggerPressed and 1 or 0) +
                        (gamepad.rightTriggerPressed and 1 or 0) +
                        (gamepad.gamepadBack and 1 or 0) +
                        (gamepad.gamepadForward and 1 or 0) +
                        (gamepad.leftStickButton and 1 or 0) +
                        (gamepad.rightStickButton and 1 or 0) +
                        (gamepad.dpadUp and 1 or 0) +
                        (gamepad.dpadDown and 1 or 0) +
                        (gamepad.dpadLeft and 1 or 0) +
                        (gamepad.dpadRight and 1 or 0) +
                        (gamepad.start and 1 or 0)
                    self.leftX = gamepad.leftStick.x
                    self.leftY = gamepad.leftStick.y
                    self.rightX = gamepad.rightStick.x
                    self.rightY = gamepad.rightStick.y
                    self.leftTrigger = gamepad.leftTrigger
                    self.rightTrigger = gamepad.rightTrigger
                    self.pressed1 = gamepad:buttonPressed(1)
                    self.pressed64 = gamepad:buttonPressed(64)
                    self.pressed0 = gamepad:buttonPressed(0)
                    self.value2 = gamepad:buttonValue(2)
                    self.valueMissing = gamepad:buttonValue(99)
                    self.valueMinimum = gamepad:buttonValue(-9223372036854775808)
                    self.axisViaMethod = gamepad:axis(1)
                    self.axisMissing = gamepad:axis(99)
                    self.axisMinimum = gamepad:axis(-9223372036854775808)
                end

                return {
                    performAction = function(self, invocation)
                        if invocation:isGamepadConnected() then
                            readSnapshot(self, invocation:asGamepadConnected())
                        elseif invocation:isGamepadEvent() then
                            local event = invocation:asGamepadEvent()
                            readSnapshot(self, event)
                            self.changeKind = event.changeKind
                            self.changeIndex = event.changeIndex
                            self.changeValue = event.changeValue
                            self.hasStandardButtonIntent = event.hasStandardButtonIntent
                            self.hasStandardAxisIntent = event.hasStandardAxisIntent
                            self.intentButton = event.intentButton
                            self.intentAxis = event.intentAxis
                        elseif invocation:isGamepadDisconnected() then
                            self.disconnectedId = invocation:asGamepadDisconnected().deviceId
                        end
                    end,
                }
            "#,
        )
        .unwrap();
    let mut instance = vm.script_instance_from_table(table);
    let snapshot = ScriptGamepadSnapshot {
        device_id: 42,
        button_mask: (1_u64 << 17) - 1,
        button_values: vec![0.5, 1.0],
        axes: vec![0.25, -0.5, 0.75, -1.0, 0.6, 0.2],
        mapping: ScriptGamepadMappingKind::Standard,
    };

    instance
        .call_listener_action(
            ScriptListenerActionMethod::PerformAction,
            &ScriptListenerInvocation::GamepadConnected {
                snapshot: snapshot.clone(),
            },
            &mut NoopScriptHost,
        )
        .unwrap();
    assert_eq!(
        instance.get_input("deviceId").unwrap(),
        ScriptValue::Number(42.0)
    );
    assert_eq!(
        instance.get_input("standardButtonCount").unwrap(),
        ScriptValue::Number(17.0)
    );
    assert_eq!(
        instance.get_input("mapping").unwrap(),
        ScriptValue::String("standard".to_owned())
    );
    assert_eq!(
        instance.get_input("gamepadMapping").unwrap(),
        ScriptValue::Number(0.0)
    );
    assert_eq!(
        instance.get_input("leftX").unwrap(),
        ScriptValue::Number(0.25)
    );
    assert_eq!(
        instance.get_input("rightTrigger").unwrap(),
        ScriptValue::Number(f64::from(0.2_f32))
    );
    assert_eq!(
        instance.get_input("buttonMask").unwrap(),
        ScriptValue::Number(((1_u64 << 17) - 1) as f64)
    );
    assert_eq!(
        instance.get_input("button1").unwrap(),
        ScriptValue::Number(0.5)
    );
    assert_eq!(
        instance.get_input("button2").unwrap(),
        ScriptValue::Number(1.0)
    );
    assert_eq!(
        instance.get_input("axis1").unwrap(),
        ScriptValue::Number(0.25)
    );
    assert_eq!(
        instance.get_input("axis6").unwrap(),
        ScriptValue::Number(f64::from(0.2_f32))
    );
    assert_eq!(
        instance.get_input("isStandardMapping").unwrap(),
        ScriptValue::Bool(true)
    );
    assert_eq!(
        instance.get_input("leftY").unwrap(),
        ScriptValue::Number(-0.5)
    );
    assert_eq!(
        instance.get_input("rightX").unwrap(),
        ScriptValue::Number(0.75)
    );
    assert_eq!(
        instance.get_input("rightY").unwrap(),
        ScriptValue::Number(-1.0)
    );
    assert_eq!(
        instance.get_input("leftTrigger").unwrap(),
        ScriptValue::Number(f64::from(0.6_f32))
    );
    assert_eq!(
        instance.get_input("pressed1").unwrap(),
        ScriptValue::Bool(true)
    );
    assert_eq!(
        instance.get_input("pressed64").unwrap(),
        ScriptValue::Bool(false)
    );
    assert_eq!(
        instance.get_input("pressed0").unwrap(),
        ScriptValue::Bool(false)
    );
    assert_eq!(
        instance.get_input("value2").unwrap(),
        ScriptValue::Number(1.0)
    );
    assert_eq!(
        instance.get_input("valueMissing").unwrap(),
        ScriptValue::Number(0.0)
    );
    assert_eq!(
        instance.get_input("valueMinimum").unwrap(),
        ScriptValue::Number(0.0)
    );
    assert_eq!(
        instance.get_input("axisViaMethod").unwrap(),
        ScriptValue::Number(0.25)
    );
    assert_eq!(
        instance.get_input("axisMissing").unwrap(),
        ScriptValue::Number(0.0)
    );
    assert_eq!(
        instance.get_input("axisMinimum").unwrap(),
        ScriptValue::Number(0.0)
    );

    instance
        .call_listener_action(
            ScriptListenerActionMethod::PerformAction,
            &ScriptListenerInvocation::GamepadEvent {
                full_state: snapshot,
                change: ScriptGamepadInputChange::Axis {
                    index: 3,
                    value: -1.0,
                },
                standard_button_intent: Some(1),
                standard_axis_intent: Some(3),
            },
            &mut NoopScriptHost,
        )
        .unwrap();
    assert_eq!(
        instance.get_input("changeKind").unwrap(),
        ScriptValue::String("axis".to_owned())
    );
    assert_eq!(
        instance.get_input("changeIndex").unwrap(),
        ScriptValue::Number(4.0)
    );
    assert_eq!(
        instance.get_input("changeValue").unwrap(),
        ScriptValue::Number(-1.0)
    );
    assert_eq!(
        instance.get_input("intentButton").unwrap(),
        ScriptValue::String("east".to_owned())
    );
    assert_eq!(
        instance.get_input("intentAxis").unwrap(),
        ScriptValue::String("rightY".to_owned())
    );

    instance
        .call_listener_action(
            ScriptListenerActionMethod::PerformAction,
            &ScriptListenerInvocation::GamepadDisconnected { device_id: -7 },
            &mut NoopScriptHost,
        )
        .unwrap();
    assert_eq!(
        instance.get_input("disconnectedId").unwrap(),
        ScriptValue::Number(-7.0)
    );

    instance
        .call_listener_action(
            ScriptListenerActionMethod::PerformAction,
            &ScriptListenerInvocation::GamepadConnected {
                snapshot: ScriptGamepadSnapshot {
                    device_id: 1,
                    button_mask: u64::MAX,
                    button_values: vec![1.0],
                    axes: vec![1.0; 6],
                    mapping: ScriptGamepadMappingKind::Unknown,
                },
            },
            &mut NoopScriptHost,
        )
        .unwrap();
    assert_eq!(
        instance.get_input("standardButtonCount").unwrap(),
        ScriptValue::Number(0.0)
    );
    assert_eq!(
        instance.get_input("mapping").unwrap(),
        ScriptValue::String("unknown".to_owned())
    );
    assert_eq!(
        instance.get_input("gamepadMapping").unwrap(),
        ScriptValue::Number(1.0)
    );
    assert_eq!(
        instance.get_input("leftX").unwrap(),
        ScriptValue::Number(0.0)
    );
    assert_eq!(
        instance.get_input("rightTrigger").unwrap(),
        ScriptValue::Number(0.0)
    );
}

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
