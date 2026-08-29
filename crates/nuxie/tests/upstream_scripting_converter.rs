//! Frozen-render ports from pinned
//! `tests/unit_tests/runtime/scripting/scripting_converter_test.cpp`.
#![cfg(feature = "scripting")]

use std::path::PathBuf;

use nuxie::{
    FileImportLimits, PersistentFactory, RuntimeArtboardInstanceHandle,
    RuntimeStateMachineInstanceHandle, RuntimeViewModelInstanceHandle, ScriptExecutionLimits,
    ScriptedFile, ViewModelInstanceRuntime, import_unsigned_scripted,
};
use nuxie_render_api::{SerializingFactory, SerializingRenderer};
use silver_corpus::{compare_sriv, parse_sriv};

fn pinned_fixture(name: &str) -> Vec<u8> {
    let root = std::env::var_os("RIVE_RUNTIME_DIR")
        .unwrap_or_else(|| "/Users/levi/dev/oss/rive-runtime".into());
    let fixture = PathBuf::from(root)
        .join("tests/unit_tests/assets")
        .join(name);
    std::fs::read(&fixture)
        .unwrap_or_else(|error| panic!("read pinned fixture {}: {error}", fixture.display()))
}

fn pinned_silver(name: &str) -> Vec<u8> {
    let root = std::env::var_os("RIVE_RUNTIME_DIR")
        .unwrap_or_else(|| "/Users/levi/dev/oss/rive-runtime".into());
    let silver = PathBuf::from(root)
        .join("tests/unit_tests/silvers")
        .join(format!("{name}.sriv"));
    std::fs::read(&silver)
        .unwrap_or_else(|error| panic!("read pinned silver {}: {error}", silver.display()))
}

fn compare_silver(name: &str, actual: &[u8]) {
    let actual = parse_sriv(actual).expect("valid Rust SRIV stream");
    let expected = parse_sriv(&pinned_silver(name)).expect("valid pinned SRIV stream");
    compare_sriv(&expected, &actual)
        .unwrap_or_else(|difference| panic!("{name} differs: {difference}"));
}

struct Fixture {
    _file: ScriptedFile,
    artboard: RuntimeArtboardInstanceHandle,
    machine: RuntimeStateMachineInstanceHandle,
    view_model: RuntimeViewModelInstanceHandle,
    silver: PersistentFactory<SerializingFactory>,
    renderer: Option<SerializingRenderer>,
}

impl Fixture {
    fn new(asset: &str, artboard_name: Option<&str>, use_default_instance: bool) -> Self {
        let mut silver = PersistentFactory::new(SerializingFactory::new());
        let file = import_unsigned_scripted(
            &pinned_fixture(asset),
            &mut silver,
            None,
            FileImportLimits::new(),
            ScriptExecutionLimits::new(),
        )
        .unwrap_or_else(|error| panic!("{asset} imports with trusted scripts: {error:#}"));
        let artboard = file
            .native_file()
            .with_file(|file| match artboard_name {
                Some(name) => file.artboard_named(name),
                None => file.artboard_default(),
            })
            .unwrap_or_else(|| panic!("{} artboard", artboard_name.unwrap_or("default")));
        let (width, height) =
            artboard.with_artboard(|artboard| (artboard.width(), artboard.height()));
        silver.borrow_mut().frame_size(width as u32, height as u32);
        let machine = artboard.state_machine_at(0).expect("state machine 0");
        let view_model_id = artboard.with_artboard(|artboard| artboard.base.view_model_id());
        let view_model = file
            .native_file()
            .with_file_mut(|file| {
                if use_default_instance {
                    file.create_default_view_model_instance_for_artboard(artboard.core_handle())
                } else if view_model_id == u32::MAX {
                    file.create_view_model_instance_for_artboard(artboard.core_handle())
                } else {
                    file.create_view_model_instance_at(view_model_id as usize, 0)
                }
            })
            .map(ViewModelInstanceRuntime::new)
            .map(ViewModelInstanceRuntime::into_handle)
            .expect("artboard view-model instance");
        machine.with_instance_mut(|machine| {
            machine.bind_view_model_instance(view_model.instance());
        });
        Self {
            _file: file,
            artboard,
            machine,
            view_model,
            silver,
            renderer: None,
        }
    }

    fn advance_draw(&mut self, seconds: f32) {
        self.machine.advance_and_apply(seconds);
        let renderer = self
            .renderer
            .get_or_insert_with(|| self.silver.borrow().make_renderer());
        self.artboard.draw(renderer);
    }

    fn add_frame(&mut self) {
        self.silver.borrow_mut().add_frame();
    }

    fn set_string(&self, name: &str, value: &str) {
        let property = self
            .view_model
            .property_string(name)
            .unwrap_or_else(|| panic!("string property {name}"));
        property.set_value(value);
        assert_eq!(property.value(), value);
    }

    fn matches(&self, name: &str) {
        compare_silver(name, &self.silver.borrow().bytes());
    }
}

#[test]
fn scripted_string_converter() {
    let mut fixture = Fixture::new("script_string_converter_test.riv", Some("Converter"), false);

    fixture.advance_draw(0.1);

    fixture.set_string("Field1", "H#e%l&l*o");
    fixture.add_frame();
    fixture.advance_draw(0.016);

    fixture.set_string("Field2", "____one two three___");
    fixture.add_frame();
    fixture.advance_draw(0.016);

    fixture.set_string("Field3", "  **This uses a string converter@@. ");
    fixture.add_frame();
    fixture.advance_draw(0.016);

    fixture.set_string("Field4", "It strips special characters like *&^%$#@!)()");
    fixture.add_frame();
    fixture.advance_draw(0.016);

    fixture.matches("script_string_converter");
}

#[test]
fn data_converter_with_bound_inputs_in_artboard_and_state_machine() {
    let mut fixture = Fixture::new("scripted_data_converter_bound_input.riv", None, true);

    fixture.advance_draw(0.1);
    fixture.add_frame();
    fixture.advance_draw(0.1);

    fixture.matches("scripted_data_converter_bound_input");
}
