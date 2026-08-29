//! Live-owner ports of the three pinned
//! `tests/unit_tests/runtime/scripting/scripting_update_phase_guard_test.cpp` cases.

use std::path::PathBuf;

use nuxie_render_api::{PersistentFactory, RecordingFactory};
use nuxie_runtime::source::scripted::scripted_object::{ScriptedObject, UPDATES_BIT};
use nuxie_runtime::{
    ComponentDirt, CoreHandle, File, RuntimeFactoryHandle, RuntimeScriptingVmHandle, ScriptMethod,
};
use nuxie_scripting::vm::ScriptVm;

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

fn compile_source(source: &str) -> Vec<u8> {
    use luaur_compiler::functions::luau_compile::luau_compile;

    luaur_common::set_all_flags(true);
    let mut output_size = 0;
    let output = luau_compile(
        source.as_ptr().cast(),
        source.len(),
        std::ptr::null_mut(),
        &mut output_size,
    );
    assert!(!output.is_null(), "pinned Luau compiler returned null");
    assert_ne!(output_size, 0, "pinned Luau compiler returned no bytecode");
    // SAFETY: luau_compile returns a malloc allocation of output_size bytes.
    let bytecode = unsafe { std::slice::from_raw_parts(output.cast::<u8>(), output_size) }.to_vec();
    unsafe extern "C" {
        fn free(pointer: *mut std::ffi::c_void);
    }
    // SAFETY: output is the allocation returned by luau_compile above.
    unsafe { free(output.cast()) };
    assert_ne!(
        bytecode.first(),
        Some(&0),
        "literal update script did not compile"
    );
    bytecode
}

fn pinned_fixture(name: &str) -> Vec<u8> {
    let root = std::env::var_os("RIVE_RUNTIME_DIR")
        .unwrap_or_else(|| "/Users/levi/dev/oss/rive-runtime".into());
    let path = PathBuf::from(root)
        .join("tests/unit_tests/assets")
        .join(name);
    std::fs::read(&path)
        .unwrap_or_else(|error| panic!("read pinned fixture {}: {error}", path.display()))
}

fn with_production_update_owner(test: impl FnOnce(&CoreHandle)) {
    // Retain a production occurrence from the pinned fixture so these preserved
    // tests observe actual Component dirt instead of the C++ test subclass's
    // counting stub. Install the same literal test program on that owner.
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let file = File::import(
        &pinned_fixture("viewmodel_access.riv"),
        RuntimeFactoryHandle::from_factory(&mut factory).expect("retained factory"),
        None,
        None,
        None,
    )
    .expect("viewmodel_access.riv imports");
    let artboard = file
        .with_file(|file| file.artboard_default())
        .expect("default artboard");
    let owner = artboard.with_artboard(|artboard| {
        artboard
            .objects()
            .iter()
            .flatten()
            .find(|owner| {
                owner
                    .with(|owner| owner.as_scripted_drawable().is_some())
                    .unwrap_or(false)
            })
            .cloned()
            .expect("fixture owns a concrete ScriptedDrawable")
    });

    let wrapped = format!(
        "local generator = (function()\n{UPDATE_SCRIPT}\nend)()\n\
         return function(context)\n\
             local instance = generator()\n\
             instance._ctx = context\n\
             return instance\n\
         end"
    );
    let bytecode = compile_source(&wrapped);
    let mut payload = Vec::with_capacity(bytecode.len() + 1);
    payload.push(0);
    payload.extend(bytecode);
    let vm = ScriptVm::new();
    let program = vm
        .register_protocol_script_with_factory("update-phase-guard", &payload, &mut factory)
        .expect("literal update program registers");
    let script = vm
        .instantiate_registered_script_with_context(&program, None, vec![None])
        .expect("literal update program initializes with a live Context");
    assert!(
        script
            .has_method(ScriptMethod::Update)
            .expect("method lookup")
    );
    owner.with_mut(|owner| {
        let scripted = owner.as_scripted_object_mut().expect("ScriptedObject base");
        scripted.install_script_instance(script, RuntimeScriptingVmHandle::new(Box::new(vm)));
        scripted.set_implemented_methods(scripted.implemented_methods() | UPDATES_BIT);
        assert!(scripted.updates());
    });
    test(&owner);
    // Keep the defining file, artboard occurrence and factory alive through
    // the real callback and teardown; there is no late runtime attachment.
}

fn in_update_phase(owner: &CoreHandle) -> bool {
    owner
        .with(|owner| owner.as_scripted_object().unwrap().in_update_phase())
        .unwrap()
}

fn has_update_dirt(owner: &CoreHandle) -> bool {
    owner
        .with(|owner| {
            owner
                .as_component()
                .unwrap()
                .dirt()
                .contains(ComponentDirt::SCRIPT_UPDATE)
        })
        .unwrap()
}

fn clear_dirt(owner: &CoreHandle) {
    owner.with_mut(|owner| {
        owner
            .as_component_mut()
            .unwrap()
            .set_dirt(ComponentDirt::NONE)
    });
}

fn mark_needs_update(owner: &CoreHandle) {
    owner.with_mut(|owner| {
        owner
            .as_scripted_drawable_mut()
            .unwrap()
            .mark_needs_update()
    });
}

#[test]
fn mark_needs_update_is_ignored_during_script_update() {
    with_production_update_owner(|owner| {
        assert!(
            !in_update_phase(owner),
            "the production owner starts outside its update phase"
        );
        mark_needs_update(owner);
        assert!(has_update_dirt(owner));
        clear_dirt(owner);
        assert!(!in_update_phase(owner));
        ScriptedObject::script_update_occurrence(owner);
        assert!(!in_update_phase(owner));
        assert!(
            !has_update_dirt(owner),
            "Context.markNeedsUpdate is suppressed by the production owner's update phase",
        );

        mark_needs_update(owner);
        assert!(
            has_update_dirt(owner),
            "outside scriptUpdate the same live owner accepts ScriptUpdate dirt",
        );
    });
}

#[test]
fn in_update_phase_defaults_to_false() {
    with_production_update_owner(|owner| {
        assert!(!in_update_phase(owner));
        mark_needs_update(owner);
        assert!(
            has_update_dirt(owner),
            "the production owner's default phase accepts ScriptUpdate dirt",
        );
    });
}

#[test]
fn mark_needs_update_works_outside_update_phase() {
    with_production_update_owner(|owner| {
        clear_dirt(owner);
        assert!(!has_update_dirt(owner));
        mark_needs_update(owner);
        assert!(has_update_dirt(owner));
        mark_needs_update(owner);
        assert!(
            has_update_dirt(owner),
            "the second outside-phase request still reaches the live production owner",
        );
    });
}
