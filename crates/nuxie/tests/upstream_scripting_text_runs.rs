//! Direct port of pinned
//! `tests/unit_tests/runtime/scripting/scripting_text_runs.cpp`.
#![cfg(feature = "scripting")]

use std::path::PathBuf;

use nuxie::{
    FileImportLimits, PersistentFactory, RuntimeArtboardInstanceHandle,
    RuntimeStateMachineInstanceHandle, RuntimeViewModelInstanceHandle, ScriptExecutionLimits,
    ScriptedFile, ViewModelInstanceRuntime, import_unsigned_scripted,
};
use nuxie_render_api::SerializingFactory;
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
        .join(name);
    std::fs::read(&silver)
        .unwrap_or_else(|error| panic!("read pinned silver {}: {error}", silver.display()))
}

struct Fixture {
    _file: ScriptedFile,
    artboard: RuntimeArtboardInstanceHandle,
    machine: RuntimeStateMachineInstanceHandle,
    view_model: RuntimeViewModelInstanceHandle,
    silver: PersistentFactory<SerializingFactory>,
}

impl Fixture {
    fn new() -> Self {
        let mut silver = PersistentFactory::new(SerializingFactory::new());
        let file = import_unsigned_scripted(
            &pinned_fixture("script_create_text_runs.riv"),
            &mut silver,
            None,
            FileImportLimits::new(),
            ScriptExecutionLimits::new(),
        )
        .expect("script_create_text_runs.riv imports with trusted scripts");
        let artboard = file
            .native_file()
            .with_file(|file| file.artboard_named("main"))
            .expect("main artboard");
        let machine = artboard.state_machine_at(0).expect("state machine 0");
        let view_model = file
            .native_file()
            .with_file_mut(|file| {
                file.create_default_view_model_instance_for_artboard(artboard.core_handle())
                    .or_else(|| {
                        file.create_view_model_instance_for_artboard(artboard.core_handle())
                    })
            })
            .map(ViewModelInstanceRuntime::new)
            .map(ViewModelInstanceRuntime::into_handle)
            .expect("main view-model instance");
        machine
            .with_instance_mut(|machine| machine.bind_view_model_instance(view_model.instance()));
        artboard.bind_view_model_instance(Some(view_model.instance()));
        Self {
            _file: file,
            artboard,
            machine,
            view_model,
            silver,
        }
    }

    fn advance(&self) {
        self.machine.advance_and_apply(0.1);
    }

    fn draw(&mut self) {
        let mut renderer = self.silver.borrow().make_renderer();
        self.artboard.draw(&mut renderer);
    }

    fn fire_button(&self, button: &str, count: usize) {
        let trigger = format!("{button}/onClick");
        let trigger = self
            .view_model
            .property_trigger(&trigger)
            .unwrap_or_else(|| panic!("trigger {trigger}"));
        for _ in 0..count {
            trigger.trigger();
        }
    }

    fn list(&self) -> nuxie::runtime::viewmodel::runtime::viewmodel_instance_list_runtime::ViewModelInstanceListRuntime{
        self.view_model
            .property_list("settings/lis")
            .expect("scripted list remains addressable")
    }
}

#[test]
fn scripted_text_run_view_model_inputs_hydrate_before_user_init() {
    let fixture = Fixture::new();
    fixture.advance();
}

#[test]
fn scripted_text_run_new_button_pushes_one_list_item() {
    let fixture = Fixture::new();
    fixture.advance();

    assert_eq!(
        fixture.list().size(),
        1,
        "authored list starts with one item"
    );
    fixture.fire_button("newButton", 1);
    fixture.advance();
    let list = fixture.list();
    assert_eq!(
        list.size(),
        2,
        "ScriptedViewModel.new feeds ScriptedPropertyList.push"
    );
    let item = list.instance_at(1).expect("pushed list item");
    assert_eq!(
        item.property_string("textContent")
            .expect("textContent")
            .value(),
        "label for 2",
        "the pushed instance retains the authored TextValueRun schema and script write"
    );
    assert_eq!(
        item.property_string("textStyle")
            .expect("textStyle")
            .value(),
        "style2",
        "the pushed instance retains the authored style selection"
    );
}

#[test]
fn script_creates_view_models_that_map_to_text_runs() {
    let mut fixture = Fixture::new();
    let (width, height) = fixture
        .artboard
        .with_artboard(|artboard| (artboard.width(), artboard.height()));
    fixture
        .silver
        .borrow_mut()
        .frame_size(width as u32, height as u32);

    fixture.advance();
    fixture.draw();

    for (button, count) in [
        ("newButton", 1),
        ("newAtButton", 1),
        ("swapButton", 1),
        ("shiftButton", 1),
        ("popButton", 1),
        ("popButton", 4),
        ("newButton", 2),
    ] {
        fixture.silver.borrow_mut().add_frame();
        fixture.fire_button(button, count);
        fixture.advance();
        fixture.draw();
    }

    let actual = parse_sriv(&fixture.silver.borrow().bytes()).expect("valid Rust SRIV stream");
    let expected = parse_sriv(&pinned_silver("script_create_text_runs.sriv"))
        .expect("valid pinned SRIV stream");
    compare_sriv(&expected, &actual)
        .unwrap_or_else(|difference| panic!("script_create_text_runs differs: {difference}"));
}
