use nuxie_render_api::{PersistentFactory, SerializingFactory};
use nuxie_runtime::source::lua::scripting_vm::RuntimeScriptingVmHandle;
use nuxie_runtime::source::viewmodel::runtime::viewmodel_instance_runtime::ViewModelInstanceRuntime;
use nuxie_runtime::{
    File, RuntimeArtboardInstanceHandle, RuntimeBlobAsset, RuntimeFactoryHandle, RuntimeFileHandle,
    RuntimeStateMachineInstanceHandle,
};
use nuxie_scripting::vm::{ScriptExecutionLimits, ScriptVm};
use silver_corpus::{Execution, compare_sriv, parse_sriv, read_manifest, resolve_expected};
use std::path::{Path, PathBuf};
use std::sync::Arc;

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("silver-corpus workspace root")
        .to_path_buf()
}

fn runtime_root(test: &str) -> Option<PathBuf> {
    let root = std::env::var_os("RIVE_RUNTIME_DIR").map(PathBuf::from);
    if root.is_none() {
        eprintln!(
            "skipping {test}; RIVE_RUNTIME_DIR is unset; point it at the pinned rive-runtime checkout"
        );
    }
    root
}

fn compare_case(id: &str) -> anyhow::Result<()> {
    let Some(runtime) = runtime_root(id) else {
        return Ok(());
    };
    let manifest = read_manifest(&workspace_root().join("silver-corpus.toml"))?;
    let case = manifest
        .cases
        .iter()
        .find(|case| case.id == id)
        .ok_or_else(|| anyhow::anyhow!("missing silver case {id}"))?;
    let actual = parse_sriv(Execution::run(case, &runtime)?.bytes())?;
    let expected = parse_sriv(&std::fs::read(resolve_expected(&runtime, case))?)?;
    compare_sriv(&expected, &actual).map_err(|difference| anyhow::anyhow!("{id}: {difference}"))
}

struct NativeSilver {
    file: RuntimeFileHandle,
    artboard: RuntimeArtboardInstanceHandle,
    machine: RuntimeStateMachineInstanceHandle,
    factory: PersistentFactory<SerializingFactory>,
}

impl NativeSilver {
    fn new(asset: &str) -> Self {
        Self::new_named(asset, None)
    }

    fn new_named(asset: &str, artboard_name: Option<&str>) -> Self {
        let runtime = runtime_root(asset).expect("RIVE_RUNTIME_DIR");
        let path = runtime.join("tests/unit_tests/assets").join(asset);
        let bytes = std::fs::read(&path)
            .unwrap_or_else(|error| panic!("read pinned fixture {}: {error}", path.display()));
        let mut factory = PersistentFactory::new(SerializingFactory::new());
        let scripting_vm = RuntimeScriptingVmHandle::new(Box::new(
            ScriptVm::new_with_execution_limits(ScriptExecutionLimits::default())
                .expect("native script VM"),
        ));
        let file = File::import(
            &bytes,
            RuntimeFactoryHandle::from_factory(&mut factory).expect("retained factory"),
            None,
            None,
            Some(scripting_vm),
        )
        .unwrap_or_else(|| panic!("import pinned fixture {}", path.display()));
        let artboard = file
            .with_file(|file| match artboard_name {
                Some(name) => file.artboard_named(name),
                None => file.artboard_default(),
            })
            .expect("selected artboard instance");
        let (width, height) =
            artboard.with_artboard(|artboard| (artboard.width(), artboard.height()));
        factory.borrow_mut().frame_size(width as u32, height as u32);
        let machine = artboard.state_machine_at(0).expect("state machine 0");
        Self {
            file,
            artboard,
            machine,
            factory,
        }
    }

    fn bind_default_view_model(&self) -> ViewModelInstanceRuntime {
        let owner = self
            .file
            .with_file_mut(|file| {
                file.create_default_view_model_instance_for_artboard(self.artboard.core_handle())
            })
            .expect("default view model instance");
        self.machine
            .with_instance_mut(|machine| machine.bind_view_model_instance(owner.clone()));
        ViewModelInstanceRuntime::new(owner)
    }

    fn draw(&mut self) {
        let mut renderer = self.factory.borrow().make_renderer();
        self.artboard.draw(&mut renderer);
    }

    fn add_frame(&mut self) {
        self.factory.borrow_mut().add_frame();
    }

    fn run_frames(&mut self, frames: usize, seconds: f32) {
        for _ in 0..frames {
            self.add_frame();
            self.machine.advance_and_apply(seconds);
            self.draw();
        }
    }

    fn compare(self, id: &str) {
        let runtime = runtime_root(id).expect("RIVE_RUNTIME_DIR");
        let expected = parse_sriv(
            &std::fs::read(
                runtime
                    .join("tests/unit_tests/silvers")
                    .join(format!("{id}.sriv")),
            )
            .expect("pinned SRIV"),
        )
        .expect("parse pinned SRIV");
        let actual = parse_sriv(&self.factory.borrow().bytes()).expect("parse native SRIV");
        compare_sriv(&expected, &actual).unwrap_or_else(|difference| panic!("{id}: {difference}"));
    }
}

fn catch_approx(actual: f32, expected: f32, margin: f32) -> bool {
    let actual = f64::from(actual);
    let expected = f64::from(expected);
    let margin = f64::from(margin);
    let relative = f64::from(f32::EPSILON) * 100.0 * expected.abs();
    (actual - expected).abs() <= margin.max(relative)
}

#[test]
fn clip_apply_outside_hierarchy() {
    compare_case("clipping_and_draw_order").unwrap();
}

#[test]
fn clip_animated_nodes() {
    compare_case("animated_clipping-nodes").unwrap();
}

#[test]
fn clip_animated_layouts() {
    compare_case("animated_clipping-layout").unwrap();
}

#[test]
fn component_list_virtualized_scroll_manual() {
    compare_case("component_list_virtualized_scroll_manual").unwrap();
}

#[test]
fn component_list_override_horizontal() {
    compare_case("artboard_list_overrides_horizontal").unwrap();
}

#[test]
fn component_list_override_vertical() {
    compare_case("artboard_list_overrides_vertical").unwrap();
}

#[test]
fn component_list_reset_triggers() {
    compare_case("reset_phase_multi_main").unwrap();
}

#[test]
fn component_list_non_layout_position() {
    compare_case("component_list_grouped").unwrap();
}

#[test]
fn component_list_follow_path() {
    compare_case("component_list_follow_path").unwrap();
}

#[test]
fn component_list_follow_path_distance() {
    compare_case("component_list_follow_path_distance").unwrap();
}

#[test]
fn component_list_hit_order() {
    compare_case("component_list_hit_order").unwrap();
}

#[test]
fn component_list_virtualized_nested_data_binding() {
    compare_case("virtualized_artboard_databound_children").unwrap();
}

#[test]
fn component_list_map_rules() {
    compare_case("artboard_list_map_rules").unwrap();
}

#[test]
fn component_list_stateful_component() {
    compare_case("component_stateful").unwrap();
}

#[test]
fn component_list_child_origin() {
    compare_case("component_list_child_origin").unwrap();
}

#[test]
fn component_list_draw_index_order() {
    compare_case("draw_index_list").unwrap();
}

#[test]
fn component_origin_animated_clicks() {
    compare_case("nested_artboard_origin_override_test").unwrap();
}

#[test]
fn component_stateful_view_model() {
    compare_case("component_stateful_vm_instance").unwrap();
}

#[test]
fn component_stateful_view_model_multi() {
    compare_case("component_stateful_vm_instance_2").unwrap();
}

#[test]
fn component_stateful_multi_property() {
    compare_case("stateful_multi_property").unwrap();
}

#[test]
fn component_stateful_nested() {
    compare_case("stateful_nested").unwrap();
}

#[test]
fn component_stateful_list_cleanup() {
    let mut silver = NativeSilver::new_named("stateful_list_props.riv", Some("Main"));
    let view_model_id = silver
        .artboard
        .with_artboard(|artboard| artboard.view_model_id());
    let owner = silver
        .file
        .with_file_mut(|file| {
            if view_model_id == u32::MAX {
                file.create_view_model_instance_for_artboard(silver.artboard.core_handle())
            } else {
                file.create_view_model_instance_at(view_model_id as usize, 0)
            }
        })
        .expect("stateful list view model");
    let view_model = ViewModelInstanceRuntime::new(owner.clone());
    silver
        .machine
        .with_instance_mut(|machine| machine.bind_view_model_instance(owner));
    silver.machine.advance_and_apply(0.0);
    silver.draw();
    let buttons = view_model
        .property_list("buttons")
        .expect("buttons list property");
    let make_button = |label: &str, tint: i32| {
        let owner = silver
            .file
            .with_file_mut(|file| {
                let model = file.view_model_named("ButtonVM")?;
                file.create_view_model_instance(model)
            })
            .expect("fresh ButtonVM");
        let runtime = ViewModelInstanceRuntime::new(owner).into_handle();
        runtime
            .property_string("label")
            .expect("ButtonVM.label")
            .set_value(label);
        runtime
            .property_color("tint")
            .expect("ButtonVM.tint")
            .set_value(tint);
        buttons.add_instance(runtime.clone());
        runtime
    };
    let _button_a = make_button("Alpha", 0xffff_3344u32 as i32);
    let button_b = make_button("Beta", 0xff33_aaffu32 as i32);
    let button_c = make_button("Gamma", 0xff44_cc55u32 as i32);
    silver.run_frames(3, 0.016);
    assert_eq!(buttons.size(), 3);
    buttons.remove_instance_at(1);
    silver.run_frames(5, 0.016);
    assert_eq!(buttons.size(), 2);
    let clicked_c = button_c.property_boolean("clicked").expect("Gamma.clicked");
    let clicked_b = button_b.property_boolean("clicked").expect("Beta.clicked");
    assert!(!clicked_c.value());
    silver.machine.with_instance_mut(|machine| {
        machine.pointer_down(
            nuxie_runtime::source::math::vec2d::Vec2D::new(50.0, 73.0),
            0,
        );
        machine.pointer_up(
            nuxie_runtime::source::math::vec2d::Vec2D::new(50.0, 73.0),
            0,
        );
    });
    silver.machine.advance_and_apply(0.016);
    assert!(clicked_c.value());
    assert!(!clicked_b.value());
    silver.run_frames(3, 0.016);
    buttons.add_instance(button_b.clone());
    silver.run_frames(5, 0.016);
    assert_eq!(buttons.size(), 3);
    silver.machine.with_instance_mut(|machine| {
        machine.pointer_down(
            nuxie_runtime::source::math::vec2d::Vec2D::new(50.0, 118.0),
            0,
        );
        machine.pointer_up(
            nuxie_runtime::source::math::vec2d::Vec2D::new(50.0, 118.0),
            0,
        );
    });
    silver.machine.advance_and_apply(0.016);
    assert!(clicked_b.value());
    silver.run_frames(3, 0.016);
    while buttons.size() > 0 {
        buttons.remove_instance_at(0);
    }
    silver.run_frames(5, 0.016);
    assert_eq!(buttons.size(), 0);
    silver.compare("stateful_list_props_lifecycle");
}

#[test]
fn component_stateful_keyed_triggers() {
    compare_case("stateful_keyed_trigger").unwrap();
}

#[test]
fn data_bind_lists_reset_triggers() {
    compare_case("viewmodel_list_trigger").unwrap();
}

#[test]
fn data_bind_lists_number_to_list_children() {
    compare_case("number_to_list_nested_children").unwrap();
}

#[test]
fn data_bind_lists_add_remove_item() {
    compare_case("list_items").unwrap();
}

#[test]
fn data_bind_lists_clear() {
    compare_case("clear_viewmodel_list").unwrap();
}

#[test]
fn data_binding_blobs_internal_external() {
    let mut silver = NativeSilver::new("data_bind_blob_test.riv");
    let model_id = silver
        .artboard
        .with_artboard(|artboard| artboard.view_model_id());
    assert_ne!(model_id, u32::MAX);
    let owner = silver
        .file
        .with_file(|file| file.create_view_model_instance_at(model_id as usize, 0))
        .expect("authored view model instance 0");
    let view_model = ViewModelInstanceRuntime::new(owner.clone());
    let blob = view_model.property_blob("xml").expect("blob property xml");
    silver
        .machine
        .with_instance_mut(|machine| machine.bind_view_model_instance(owner));
    silver.machine.advance_and_apply(0.1);
    silver.draw();
    silver.add_frame();
    silver.machine.advance_and_apply(0.1);
    silver.draw();
    for _ in 0..(2.0 / 0.5) as usize {
        silver.add_frame();
        silver.machine.advance_and_apply(0.5);
        silver.draw();
    }
    let runtime = runtime_root("data_bind_blob_test").expect("RIVE_RUNTIME_DIR");
    let bytes = std::fs::read(
        runtime
            .join("tests/unit_tests/assets")
            .join("data_enum_roundtrip.rml"),
    )
    .expect("external blob fixture");
    let external = Arc::new(RuntimeBlobAsset::new(
        "data_enum_roundtrip.rml",
        Arc::from(bytes.clone().into_boxed_slice()),
    ));
    blob.set_value(Some(external.clone()));
    let retained = blob.testing_value().expect("live external blob");
    assert!(Arc::ptr_eq(&retained, &external));
    assert_eq!(retained.bytes().len(), bytes.len());
    silver.add_frame();
    silver.machine.advance_and_apply(0.5);
    silver.draw();
    silver.compare("data_bind_blob_test");
}

#[test]
fn data_binding_computed_root_values() {
    let mut silver = NativeSilver::new("computed_values_test.riv");
    silver.bind_default_view_model();
    silver.machine.advance_and_apply(0.0);
    silver.machine.advance_and_apply(0.016);
    silver.draw();
    for _ in 0..(2.0 / 0.032) as usize {
        silver.add_frame();
        silver.machine.advance_and_apply(0.032);
        silver.draw();
    }
    silver.compare("computed_values_test");
}

#[test]
fn data_binding_computed_image_resize() {
    let mut silver = NativeSilver::new("image_computed_transform_bind.riv");
    let view_model = silver.bind_default_view_model();
    let number = |name: &str| {
        view_model
            .property_number(name)
            .unwrap_or_else(|| panic!("number property {name}"))
            .value()
    };
    silver.machine.advance_and_apply(0.0);
    silver.machine.advance_and_apply(0.016);
    silver.draw();
    for name in ["img1Width", "img1Height", "img2Width", "img2Height"] {
        let actual = number(name);
        assert!(catch_approx(actual, 150.0, 5.0), "{name}={actual}");
    }
    for _ in 0..(2.0 / 0.032) as usize {
        silver.add_frame();
        silver.machine.advance_and_apply(0.032);
        silver.draw();
    }
    for name in ["img1Width", "img1Height"] {
        let actual = number(name);
        assert!(catch_approx(actual, 200.0, 0.01), "{name}={actual}");
    }
    for name in ["img2Width", "img2Height"] {
        let actual = number(name);
        assert!(catch_approx(actual, 250.0, 0.01), "{name}={actual}");
    }
    silver.compare("image_computed_transform_bind");
}
