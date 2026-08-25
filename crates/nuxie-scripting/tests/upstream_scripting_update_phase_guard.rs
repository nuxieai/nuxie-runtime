//! One-for-one ports of
//! `tests/unit_tests/runtime/scripting/scripting_update_phase_guard_test.cpp`.
#![cfg(all(feature = "luau", feature = "compiler"))]

use nuxie_render_api::{PersistentFactory, RecordingFactory};
use nuxie_runtime::{ScriptHost, ScriptInstance, ScriptMethod};
use nuxie_scripting::vm::ScriptVm;

mod support;
use support::compile_source;

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

#[derive(Default)]
struct TestScriptedDrawable {
    dirt_count: usize,
    in_update_phase: bool,
}

impl TestScriptedDrawable {
    fn mark_needs_update(&mut self) {
        if self.in_update_phase {
            return;
        }
        self.dirt_count += 1;
    }

    fn script_update(&mut self, instance: &mut dyn ScriptInstance) {
        self.in_update_phase = true;
        instance
            .call_method(ScriptMethod::Update, &[], self)
            .expect("protected update callback");
        self.in_update_phase = false;
    }
}

impl ScriptHost for TestScriptedDrawable {
    fn mark_script_update(&mut self) {
        self.mark_needs_update();
    }
}

fn scripted_instance() -> Box<dyn ScriptInstance> {
    // The upstream test installs ScriptedContext into `_ctx` after invoking
    // the exact generator. Rust's safe adapter performs that attachment in a
    // wrapper because ScriptInstance intentionally does not expose its Lua
    // self table.
    let wrapped = format!(
        "local generator = (function()\n{UPDATE_SCRIPT}\nend)()\n\
         return function(context)\n\
             local instance = generator()\n\
             instance._ctx = context\n\
             return instance\n\
         end"
    );
    let bytecode = compile_source(&wrapped).expect("update-phase script compiles");
    let mut payload = Vec::with_capacity(bytecode.len() + 1);
    payload.push(0);
    payload.extend(bytecode);

    let vm = ScriptVm::new();
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let program = vm
        .register_protocol_script_with_factory("update-phase-guard", &payload, &mut factory)
        .expect("script registers");
    vm.instantiate_registered_script_with_context(&program, None, Vec::new())
        .expect("script initializes")
}

#[test]
fn mark_needs_update_is_ignored_during_script_update() {
    let mut instance = scripted_instance();
    assert!(instance.has_method(ScriptMethod::Update).unwrap());
    let mut obj = TestScriptedDrawable::default();

    assert!(!obj.in_update_phase);
    obj.script_update(instance.as_mut());
    assert!(!obj.in_update_phase);
    assert_eq!(obj.dirt_count, 0);

    obj.mark_needs_update();
    assert_eq!(obj.dirt_count, 1);
}

#[test]
fn in_update_phase_defaults_to_false() {
    let obj = TestScriptedDrawable::default();
    assert!(!obj.in_update_phase);
}

#[test]
fn mark_needs_update_works_outside_update_phase() {
    let mut obj = TestScriptedDrawable::default();
    assert_eq!(obj.dirt_count, 0);
    obj.mark_needs_update();
    assert_eq!(obj.dirt_count, 1);
    obj.mark_needs_update();
    assert_eq!(obj.dirt_count, 2);
}
