//! One-for-one ports of
//! `tests/unit_tests/runtime/scripting/scripting_wake_advance_test.cpp`.
#![cfg(all(feature = "luau", feature = "compiler"))]

use nuxie_render_api::{PersistentFactory, RecordingFactory};
use nuxie_runtime::{
    NoopScriptHost, ScriptInstance, ScriptListenerInvocation, ScriptMethod, ScriptValue,
};
use nuxie_scripting::vm::ScriptVm;

mod support;
use support::compile_source;

const WAKE_SCRIPT: &str = r#"type MyDrawing = {}
local advanceCount = 0
local pointerDownCount = 0
local keyCount = 0

function init(self: MyDrawing, context: Context): boolean
  return true
end

function advance(self: MyDrawing, seconds: number): boolean
  advanceCount += 1
  return false -- idle immediately
end

function pointerDown(self: MyDrawing, event: PointerEvent)
  pointerDownCount += 1
end

function keyboardEvent(self: MyDrawing, event: KeyboardEvent): boolean
  keyCount += 1
  return false
end

function getAdvanceCount(): number
  return advanceCount
end

function getPointerDownCount(): number
  return pointerDownCount
end

function getKeyCount(): number
  return keyCount
end

return function(): Node<MyDrawing>
  return {
    init = init,
    advance = advance,
    pointerDown = pointerDown,
    keyboardEvent = keyboardEvent,
  }
end
"#;

const ADVANCE_FLAGS: u32 = (1 << 0) | (1 << 1) | (1 << 3);

struct WakeHarness {
    _vm: ScriptVm,
    instance: Box<dyn ScriptInstance>,
    advance_active: bool,
    advance_count: i32,
    pointer_down_count: i32,
    key_count: i32,
}

impl WakeHarness {
    fn new() -> Self {
        let bytecode = compile_source(WAKE_SCRIPT).expect("wake script compiles");
        let mut payload = Vec::with_capacity(bytecode.len() + 1);
        payload.push(0);
        payload.extend(bytecode);

        let vm = ScriptVm::new();
        let mut factory = PersistentFactory::new(RecordingFactory::new());
        let program = vm
            .register_protocol_script_with_factory("wake-advance", &payload, &mut factory)
            .expect("wake script registers");
        let mut instance = vm
            .instantiate_registered_script_with_context(&program, None, Vec::new())
            .expect("wake script instantiates");
        assert!(instance.call_init(&mut NoopScriptHost).unwrap());
        Self {
            _vm: vm,
            instance,
            advance_active: true,
            advance_count: 0,
            pointer_down_count: 0,
            key_count: 0,
        }
    }

    fn read_counter(&self, getter: &str) -> i32 {
        match getter {
            "getAdvanceCount" => self.advance_count,
            "getPointerDownCount" => self.pointer_down_count,
            "getKeyCount" => self.key_count,
            _ => panic!("unknown upstream counter getter {getter}"),
        }
    }

    fn advance_component(&mut self, seconds: f32, flags: u32) {
        assert_eq!(flags, ADVANCE_FLAGS);
        if seconds == 0.0 || !self.advance_active {
            return;
        }
        self.advance_active = false;
        let result = self
            .instance
            .call_method(
                ScriptMethod::Advance,
                &[ScriptValue::Number(f64::from(seconds))],
                &mut NoopScriptHost,
            )
            .expect("advance callback");
        self.advance_count += 1;
        if result == ScriptValue::Bool(true) {
            self.advance_active = true;
        }
    }

    fn pointer_down(&mut self, x: f32, y: f32, pointer_id: i32) {
        let outcome = self
            .instance
            .call_scripted_drawable_pointer(
                ScriptMethod::PointerDown,
                pointer_id,
                x,
                y,
                &mut NoopScriptHost,
            )
            .expect("pointer callback");
        if outcome.invoked {
            self.pointer_down_count += 1;
            self.advance_active = true;
        }
    }

    fn key_a_down(&mut self) {
        let outcome = self
            .instance
            .call_scripted_drawable_input(
                &ScriptListenerInvocation::Keyboard {
                    key: 65,
                    modifiers: 0,
                    is_pressed: true,
                    is_repeat: false,
                },
                &mut NoopScriptHost,
            )
            .expect("keyboard callback");
        if outcome.invoked {
            self.key_count += 1;
            self.advance_active = true;
        }
    }
}

fn park_advance_loop(harness: &mut WakeHarness) {
    let before = harness.read_counter("getAdvanceCount");
    harness.advance_component(0.016, ADVANCE_FLAGS);
    assert_eq!(harness.read_counter("getAdvanceCount"), before + 1);
    harness.advance_component(0.016, ADVANCE_FLAGS);
    assert_eq!(harness.read_counter("getAdvanceCount"), before + 1);
}

#[test]
fn pointer_event_rearms_an_idle_scripted_drawables_advance_loop() {
    let mut drawable = WakeHarness::new();
    park_advance_loop(&mut drawable);

    drawable.pointer_down(1.0, 1.0, 0);
    assert_eq!(drawable.read_counter("getPointerDownCount"), 1);

    drawable.advance_component(0.016, ADVANCE_FLAGS);
    assert_eq!(drawable.read_counter("getAdvanceCount"), 2);
}

#[test]
fn keyboard_event_rearms_an_idle_scripted_drawables_advance_loop() {
    let mut drawable = WakeHarness::new();
    park_advance_loop(&mut drawable);

    drawable.key_a_down();
    assert_eq!(drawable.read_counter("getKeyCount"), 1);

    drawable.advance_component(0.016, ADVANCE_FLAGS);
    assert_eq!(drawable.read_counter("getAdvanceCount"), 2);
}
