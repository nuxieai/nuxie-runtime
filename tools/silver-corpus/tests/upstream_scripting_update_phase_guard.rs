//! Exact live-owner port of pinned
//! `tests/unit_tests/runtime/scripting/scripting_update_phase_guard_test.cpp#1`.

use std::path::PathBuf;

use nuxie::{File, PersistentFactory};
use nuxie_render_api::RecordingFactory;
use nuxie_runtime::{ComponentDirt, ScriptMethod};
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

fn with_production_update_owner(test: impl FnOnce(&mut nuxie::ArtboardInstance<'_>, usize, u32)) {
    // Rust does not expose an abstract ScriptedDrawable constructor. Retain a
    // concrete production occurrence from a pinned scripted fixture and attach
    // the exact literal test program to that owner.
    let file = File::import(&pinned_fixture("viewmodel_access.riv"))
        .expect("viewmodel_access.riv imports");
    let artboard = file.default_artboard().expect("default artboard");
    let component = artboard
        .graph()
        .components
        .iter()
        .find(|component| component.type_name == "ScriptedDrawable")
        .expect("fixture owns a concrete ScriptedDrawable");
    let local_id = component.local_id;
    let global_id = component.global_id;
    let mut artboard = artboard
        .instantiate()
        .expect("default artboard instantiates");

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
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let program = vm
        .register_protocol_script_with_factory("update-phase-guard", &payload, &mut factory)
        .expect("literal update program registers");
    let script = vm
        .instantiate_registered_script_with_context(&program, None, vec![None])
        .expect("literal update program initializes with a live Context");
    assert!(script
        .has_method(ScriptMethod::Update)
        .expect("method lookup"));
    artboard
        .raw_mut()
        .set_script_instance_for_global(global_id, script);
    test(&mut artboard, local_id, global_id);
}

#[test]
fn mark_needs_update_is_ignored_during_script_update() {
    with_production_update_owner(|artboard, local_id, global_id| {
        assert!(
            artboard.raw_mut().mark_script_update_for_global(global_id),
            "the production owner starts outside its update phase",
        );
        artboard.raw_mut().clear_component_dirt(local_id);
        assert!(artboard
            .raw_mut()
            .update_script_instances()
            .expect("scriptUpdate"));
        assert!(
            !artboard
                .raw()
                .debug_component_dirt(local_id)
                .is_some_and(|dirt| dirt.contains(ComponentDirt::SCRIPT_UPDATE)),
            "Context.markNeedsUpdate is suppressed by the production owner's update phase",
        );

        assert!(artboard.raw_mut().mark_script_update_for_global(global_id));
        assert!(
            artboard
                .raw()
                .debug_component_dirt(local_id)
                .is_some_and(|dirt| dirt.contains(ComponentDirt::SCRIPT_UPDATE)),
            "outside scriptUpdate the same live owner accepts ScriptUpdate dirt",
        );
    });
}

#[test]
fn in_update_phase_defaults_to_false() {
    with_production_update_owner(|artboard, _local_id, global_id| {
        assert!(
            artboard.raw_mut().mark_script_update_for_global(global_id),
            "the production owner's default phase accepts ScriptUpdate dirt",
        );
    });
}

#[test]
fn mark_needs_update_works_outside_update_phase() {
    with_production_update_owner(|artboard, local_id, global_id| {
        artboard.raw_mut().clear_component_dirt(local_id);
        assert!(!artboard
            .raw()
            .debug_component_dirt(local_id)
            .is_some_and(|dirt| dirt.contains(ComponentDirt::SCRIPT_UPDATE)),);
        assert!(artboard.raw_mut().mark_script_update_for_global(global_id));
        assert!(artboard
            .raw()
            .debug_component_dirt(local_id)
            .is_some_and(|dirt| dirt.contains(ComponentDirt::SCRIPT_UPDATE)),);
        assert!(artboard.raw_mut().mark_script_update_for_global(global_id));
        assert!(
            artboard
                .raw()
                .debug_component_dirt(local_id)
                .is_some_and(|dirt| dirt.contains(ComponentDirt::SCRIPT_UPDATE)),
            "the second outside-phase request still reaches the live production owner",
        );
    });
}
