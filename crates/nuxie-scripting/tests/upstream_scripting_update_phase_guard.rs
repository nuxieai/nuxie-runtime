//! One-for-one expected-red ports of
//! `tests/unit_tests/runtime/scripting/scripting_update_phase_guard_test.cpp`.
//!
//! Rust exposes neither the upstream per-object `inUpdatePhase` state nor the
//! `ScriptedDrawable::markNeedsUpdate` dirt owner. The complete script and all
//! three action/assertion sequences remain here for source correspondence.

const UPDATE_SCRIPT: &str = r#"
type MyObj = {
  _ctx: Context?,
}

function update(self: MyObj)
  if self._ctx then
    self._ctx:markNeedsUpdate()
  end
end

return function(): Node<MyObj>
  return {
    _ctx = nil,
    update = update,
  }
end
"#;

struct MissingUpdatePhaseDrawable;

impl MissingUpdatePhaseDrawable {
    fn new() -> Self {
        Self
    }

    fn from_script(script: &str) -> Self {
        assert_eq!(script, UPDATE_SCRIPT);
        Self
    }

    fn ensure_script_initialized(&mut self) -> bool {
        missing_scripted_drawable_update_phase_owner()
    }

    fn attach_context_to_self(&mut self) {
        missing_scripted_drawable_update_phase_owner()
    }

    fn implemented_methods(&self) -> u32 {
        missing_scripted_drawable_update_phase_owner()
    }

    fn set_implemented_methods(&mut self, _: u32) {
        missing_scripted_drawable_update_phase_owner()
    }

    fn updates(&self) -> bool {
        missing_scripted_drawable_update_phase_owner()
    }

    fn reset_dirt_count(&mut self) {
        missing_scripted_drawable_update_phase_owner()
    }

    fn in_update_phase(&self) -> bool {
        missing_scripted_drawable_update_phase_owner()
    }

    fn script_update(&mut self) {
        missing_scripted_drawable_update_phase_owner()
    }

    fn dirt_count(&self) -> i32 {
        missing_scripted_drawable_update_phase_owner()
    }

    fn mark_needs_update(&mut self) {
        missing_scripted_drawable_update_phase_owner()
    }

    fn clear_scripted_object_from_context(&mut self) {
        missing_scripted_drawable_update_phase_owner()
    }
}

fn missing_scripted_drawable_update_phase_owner() -> ! {
    panic!("Rust runtime has no primary ScriptedDrawable update-phase and dirt owner")
}

#[test]
#[ignore = "expected red: source correspondence must supply the ScriptedDrawable update-phase owner"]
fn mark_needs_update_is_ignored_during_script_update() {
    let mut obj = MissingUpdatePhaseDrawable::from_script(UPDATE_SCRIPT);

    assert!(obj.ensure_script_initialized());

    obj.attach_context_to_self();

    obj.set_implemented_methods(obj.implemented_methods() | (1 << 1));
    assert!(obj.updates());

    obj.reset_dirt_count();
    assert!(!obj.in_update_phase());
    obj.script_update();
    assert!(!obj.in_update_phase());
    assert_eq!(obj.dirt_count(), 0);

    obj.reset_dirt_count();
    obj.mark_needs_update();
    assert_eq!(obj.dirt_count(), 1);

    obj.clear_scripted_object_from_context();
}

#[test]
#[ignore = "expected red: source correspondence must supply the ScriptedDrawable update-phase owner"]
fn in_update_phase_defaults_to_false() {
    let obj = MissingUpdatePhaseDrawable::new();
    assert!(!obj.in_update_phase());
}

#[test]
#[ignore = "expected red: source correspondence must supply the ScriptedDrawable dirt owner"]
fn mark_needs_update_works_outside_update_phase() {
    let mut obj = MissingUpdatePhaseDrawable::new();
    assert_eq!(obj.dirt_count(), 0);
    obj.mark_needs_update();
    assert_eq!(obj.dirt_count(), 1);
    obj.mark_needs_update();
    assert_eq!(obj.dirt_count(), 2);
}
