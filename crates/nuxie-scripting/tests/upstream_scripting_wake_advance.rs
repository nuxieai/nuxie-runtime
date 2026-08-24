//! One-for-one expected-red ports of
//! `tests/unit_tests/runtime/scripting/scripting_wake_advance_test.cpp`.
//!
//! The Rust runtime does not yet expose the upstream `ScriptedDrawable` wake
//! owner. The complete fixture and both action/assertion sequences remain here
//! for the source-correspondence phase.

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

struct MissingWakeHarness;

impl MissingWakeHarness {
    fn new(script: &str) -> Self {
        assert_eq!(script, WAKE_SCRIPT);
        Self
    }

    fn implemented_methods(&mut self, _: u32) {
        missing_scripted_drawable_wake_owner()
    }

    fn set_script_asset(&mut self) {
        missing_scripted_drawable_wake_owner()
    }

    fn ensure_script_initialized(&mut self) -> bool {
        missing_scripted_drawable_wake_owner()
    }

    fn read_counter(&self, _: &str) -> i32 {
        missing_scripted_drawable_wake_owner()
    }

    fn advance_component(&mut self, _: f32, _: u32) {
        missing_scripted_drawable_wake_owner()
    }

    fn pointer_down(&mut self, _: f32, _: f32, _: bool, _: f32, _: u32) {
        missing_scripted_drawable_wake_owner()
    }

    fn key_a_down(&mut self, _: bool) {
        missing_scripted_drawable_wake_owner()
    }
}

fn missing_scripted_drawable_wake_owner() -> ! {
    panic!("Rust runtime has no primary ScriptedDrawable idle/wake owner")
}

fn park_advance_loop(harness: &mut MissingWakeHarness) {
    let before = harness.read_counter("getAdvanceCount");
    harness.advance_component(0.016, ADVANCE_FLAGS);
    assert_eq!(harness.read_counter("getAdvanceCount"), before + 1);
    harness.advance_component(0.016, ADVANCE_FLAGS);
    assert_eq!(harness.read_counter("getAdvanceCount"), before + 1);
}

#[test]
#[ignore = "expected red: source correspondence must supply the ScriptedDrawable wake owner"]
fn pointer_event_rearms_an_idle_scripted_drawables_advance_loop() {
    let mut drawable = MissingWakeHarness::new(WAKE_SCRIPT);
    drawable.implemented_methods((1 << 0) | (1 << 3));
    drawable.set_script_asset();
    assert!(drawable.ensure_script_initialized());

    park_advance_loop(&mut drawable);

    drawable.pointer_down(1.0, 1.0, true, 0.0, 0);
    assert_eq!(drawable.read_counter("getPointerDownCount"), 1);

    drawable.advance_component(0.016, ADVANCE_FLAGS);
    assert_eq!(drawable.read_counter("getAdvanceCount"), 2);
}

#[test]
#[ignore = "expected red: source correspondence must supply the ScriptedDrawable wake owner"]
fn keyboard_event_rearms_an_idle_scripted_drawables_advance_loop() {
    let mut drawable = MissingWakeHarness::new(WAKE_SCRIPT);
    drawable.implemented_methods((1 << 0) | (1 << 16));
    drawable.set_script_asset();
    assert!(drawable.ensure_script_initialized());

    park_advance_loop(&mut drawable);

    drawable.key_a_down(false);
    assert_eq!(drawable.read_counter("getKeyCount"), 1);

    drawable.advance_component(0.016, ADVANCE_FLAGS);
    assert_eq!(drawable.read_counter("getAdvanceCount"), 2);
}
