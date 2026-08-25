//! Executable expected-red action-stream ports from pinned
//! `tests/unit_tests/runtime/data_binding_artboards_test.cpp`.

use std::collections::BTreeMap;
use std::path::PathBuf;

use nuxie::{File, StateMachineInstance, ViewModelInstance};
use nuxie_render_api::SerializingFactory;
use nuxie_runtime::RuntimeBindableArtboard;

#[derive(Clone, Copy, Debug)]
enum Action {
    Import(&'static str),
    SelectArtboard(&'static str),
    FrameSize,
    CreateViewModel,
    BindViewModel,
    Advance(f32),
    Draw,
    Frame,
    SetArtboard(&'static str, Option<(&'static str, &'static str)>),
    SetArtboardIndex(&'static str, u64),
    SetBoundViewModel(&'static str, &'static str, &'static str),
    ReplaceViewModel(&'static str, &'static str),
    SetString(&'static str, &'static str),
    SetBool(&'static str, bool),
    Fire(&'static str, usize),
    PointerDown(f32, f32),
    PointerUp(f32, f32),
    ExpectNested(&'static str, bool),
    Compare(&'static str),
}

fn pinned_fixture(name: &str) -> PathBuf {
    let root = std::env::var_os("RIVE_RUNTIME_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/Users/levi/dev/oss/rive-runtime"));
    root.join("tests/unit_tests/assets").join(name)
}

struct LiveStream {
    files: BTreeMap<&'static str, &'static File>,
    primary_file: Option<&'static str>,
    artboard: Option<nuxie::ArtboardInstance<'static>>,
    machine: Option<StateMachineInstance>,
    view_model: Option<ViewModelInstance>,
    silver: SerializingFactory,
}

impl LiveStream {
    fn new() -> Self {
        Self {
            files: BTreeMap::new(),
            primary_file: None,
            artboard: None,
            machine: None,
            view_model: None,
            silver: SerializingFactory::new(),
        }
    }

    fn current_file(&self) -> &'static File {
        self.files[self.primary_file.expect("primary imported File")]
    }

    fn artboard_mut(&mut self) -> &mut nuxie::ArtboardInstance<'static> {
        self.artboard.as_mut().expect("selected ArtboardInstance")
    }

    fn view_model_mut(&mut self) -> &mut ViewModelInstance {
        self.view_model.as_mut().expect("created ViewModelInstance")
    }

    fn advance(&mut self, seconds: f32) {
        let mut artboard = self.artboard.take().expect("selected ArtboardInstance");
        let mut machine = self.machine.take();
        let mut view_model = self.view_model.take();
        match (&mut machine, &mut view_model) {
            (Some(machine), Some(view_model)) => {
                artboard.advance_with_state_machines_and_view_model(
                    std::slice::from_mut(machine),
                    seconds,
                    view_model,
                );
            }
            (Some(machine), None) => {
                artboard.advance_with_state_machine(machine, seconds);
            }
            (None, _) => {
                artboard.advance(seconds);
            }
        }
        self.artboard = Some(artboard);
        self.machine = machine;
        self.view_model = view_model;
    }

    fn draw(&mut self) {
        let mut artboard = self.artboard.take().expect("selected ArtboardInstance");
        let mut renderer = self.silver.make_renderer();
        artboard
            .draw(&mut self.silver, &mut renderer)
            .expect("translated artboard draw");
        self.artboard = Some(artboard);
    }

    fn set_artboard(&mut self, property: &str, source: Option<(&str, &str)>) {
        let bindable = source.map(|(file_name, artboard_name)| {
            let source_file = self.files[file_name];
            let source = source_file
                .artboard_named(artboard_name)
                .or_else(|| {
                    (artboard_name == "default")
                        .then(|| source_file.default_artboard())
                        .flatten()
                })
                .unwrap_or_else(|| panic!("{file_name} has artboard {artboard_name}"))
                .instantiate()
                .unwrap_or_else(|error| panic!("instantiate {file_name}/{artboard_name}: {error:#}"));
            RuntimeBindableArtboard::new_with_artboard_instance(artboard_name, source.raw())
        });
        assert!(
            self.view_model_mut()
                .raw_mut()
                .set_runtime_artboard_by_property_name(property, bindable),
            "translated SetArtboard must reach the live ViewModelInstanceArtboard owner"
        );
    }
}

fn execute_until_concrete_parity_boundary(actions: &[Action]) {
    let mut live = LiveStream::new();
    for action in actions {
        match *action {
            Action::Import(name) => {
                let fixture = pinned_fixture(name);
                let bytes = std::fs::read(&fixture).unwrap_or_else(|error| {
                    panic!("missing fixture {}: {error}", fixture.display())
                });
                let file = Box::leak(Box::new(
                    File::import(&bytes)
                        .unwrap_or_else(|error| panic!("fixture {name} imports: {error}")),
                ));
                live.primary_file.get_or_insert(name);
                live.files.insert(name, file);
            }
            Action::SelectArtboard(name) => {
                let artboard = live
                    .current_file()
                    .artboard_named(name)
                    .or_else(|| (name == "default").then(|| live.current_file().default_artboard()).flatten())
                    .unwrap_or_else(|| panic!("primary fixture has artboard {name}"))
                    .instantiate()
                    .unwrap_or_else(|error| panic!("instantiate artboard {name}: {error:#}"));
                live.artboard = Some(artboard);
                live.machine = live.artboard_mut().default_state_machine_instance();
            }
            Action::FrameSize => {
                let (width, height) = live.artboard_mut().artboard_dimensions();
                assert!(width.is_finite() && height.is_finite());
                live.silver.frame_size(width as u32, height as u32);
            }
            Action::CreateViewModel => {
                live.view_model = live
                    .artboard_mut()
                    .instantiate_default_view_model_instance()
                    .or_else(|| live.artboard_mut().instantiate_view_model());
                assert!(live.view_model.is_some(), "fixture has a live ViewModelInstance");
            }
            Action::BindViewModel => {
                let view_model = live.view_model.as_ref().expect("created ViewModelInstance").clone();
                let _ = live.artboard_mut().bind_view_model(&view_model);
                if let Some(machine) = live.machine.as_mut() {
                    let _ = machine.bind_owned_view_model_handle(view_model.handle());
                }
            }
            Action::Draw => live.draw(),
            Action::Frame => live.silver.add_frame(),
            Action::Advance(seconds) => live.advance(seconds),
            Action::SetArtboard(property, source) => live.set_artboard(property, source),
            Action::SetArtboardIndex(property, value) => {
                assert!(live.view_model_mut().set_artboard(property, value));
            }
            Action::SetBoundViewModel(property, model, instance) => {
                let bound = live
                    .files
                    .values()
                    .find_map(|file| {
                        file.view_model_named(model)
                            .and_then(|view_model| view_model.instantiate_instance_named(instance))
                    })
                    .unwrap_or_else(|| panic!("translated bound ViewModel {model}/{instance}"));
                assert!(
                    live.view_model_mut()
                        .raw()
                        .runtime_artboard_by_property_name(property)
                        .is_some(),
                    "SetBoundViewModel follows a live SetArtboard"
                );
                let _ = bound;
                panic!(
                    "expected-red: RuntimeOwnedViewModelInstance has no public setter for ViewModelInstanceArtboard::boundViewModelInstance"
                );
            }
            Action::ReplaceViewModel(property, source) => {
                let replacement = live
                    .files
                    .values()
                    .find_map(|file| {
                        file.view_models().find_map(|view_model| {
                            view_model.instantiate_instance_named(source)
                        })
                    })
                    .unwrap_or_else(|| panic!("translated replacement ViewModel {source}"));
                live.view_model_mut()
                    .handle()
                    .link_view_model_by_property_name_path(property, replacement.handle())
                    .expect("translated ReplaceViewModel reaches live graph owner");
            }
            Action::SetString(property, value) => {
                assert!(live.view_model_mut().set_string(property, value));
            }
            Action::SetBool(property, value) => {
                assert!(live.view_model_mut().set_bool(property, value));
            }
            Action::Fire(property, count) => {
                for _ in 0..count {
                    assert!(live.view_model_mut().fire_trigger(property));
                }
            }
            Action::PointerDown(x, y) => {
                let mut artboard = live.artboard.take().expect("selected ArtboardInstance");
                live.machine
                    .as_mut()
                    .expect("StateMachineInstance")
                    .pointer_down(artboard.raw_mut(), x, y, 0);
                live.artboard = Some(artboard);
            }
            Action::PointerUp(x, y) => {
                let mut artboard = live.artboard.take().expect("selected ArtboardInstance");
                live.machine
                    .as_mut()
                    .expect("StateMachineInstance")
                    .pointer_up(artboard.raw_mut(), x, y, 0);
                live.artboard = Some(artboard);
            }
            Action::ExpectNested(name, expected) => {
                assert!(
                    live.artboard_mut()
                        .artboard()
                        .graph()
                        .components
                        .iter()
                        .any(|component| component.name.as_deref() == Some(name)),
                    "translated nested host exists"
                );
                let _ = expected;
                panic!(
                    "expected-red: the facade has no callable nested-Artboard occurrence-presence query"
                );
            }
            Action::Compare(name) => {
                assert!(live.silver.bytes().len() > 16, "live draw stream is non-empty");
                panic!(
                    "expected-red: live translated stream reached the missing pinned SRIV comparator for {name}"
                );
            }
        }
    }
}

#[test]
#[ignore = "expected-red: live initial draw reaches missing nested artboard graph for global 7"]
fn data_binding_artboards_from_same_and_different_sources() {
    execute_until_concrete_parity_boundary(&[
        Action::Import("data_binding_artboards_test.riv"),
        Action::Import("data_binding_artboards_source_test.riv"),
        Action::SelectArtboard("default"),
        Action::FrameSize,
        Action::CreateViewModel,
        Action::BindViewModel,
        Action::Advance(0.1),
        Action::Draw,
        Action::Frame,
        Action::Advance(0.1),
        Action::Draw,
        Action::SetArtboard("ab", Some(("data_binding_artboards_test.riv", "ch1"))),
        Action::Frame,
        Action::Advance(0.1),
        Action::Draw,
        Action::SetArtboard("ab", Some(("data_binding_artboards_test.riv", "ch2"))),
        Action::Frame,
        Action::Advance(0.1),
        Action::Draw,
        Action::SetArtboard(
            "ab",
            Some(("data_binding_artboards_source_test.riv", "source_1")),
        ),
        Action::Frame,
        Action::Advance(0.1),
        Action::Draw,
        Action::Fire("ch/tr", 1),
        Action::Frame,
        Action::Advance(0.1),
        Action::Draw,
        Action::Fire("ch/tr", 1),
        Action::Frame,
        Action::Advance(0.1),
        Action::Draw,
        Action::SetArtboard(
            "ab",
            Some(("data_binding_artboards_source_test.riv", "source_2")),
        ),
        Action::Frame,
        Action::Advance(0.1),
        Action::Draw,
        Action::SetArtboard(
            "ab",
            Some(("data_binding_artboards_source_test.riv", "source_1")),
        ),
        Action::Frame,
        Action::Advance(0.1),
        Action::Draw,
        Action::SetArtboard("ab", Some(("data_binding_artboards_test.riv", "ch2"))),
        Action::Frame,
        Action::Advance(0.1),
        Action::Draw,
        Action::SetArtboard("ab", None),
        Action::Frame,
        Action::Advance(0.1),
        Action::Draw,
        Action::SetArtboard("ab", Some(("data_binding_artboards_test.riv", "ch2"))),
        Action::Frame,
        Action::Advance(0.1),
        Action::Draw,
        Action::Compare("data_binding_artboards_test"),
    ]);
}

#[test]
#[ignore = "expected-red: complete live stream reaches the unavailable pinned SRIV comparator"]
fn recursive_data_binding_artboards_are_skipped() {
    execute_until_concrete_parity_boundary(&[
        Action::Import("data_binding_artboards_test.riv"),
        Action::SelectArtboard("recursive-grand-parent"),
        Action::FrameSize,
        Action::CreateViewModel,
        Action::BindViewModel,
        Action::Advance(0.1),
        Action::Draw,
        Action::Frame,
        Action::Advance(0.1),
        Action::Draw,
        Action::SetArtboard(
            "ab",
            Some(("data_binding_artboards_test.riv", "recursive-grand-child-1")),
        ),
        Action::Frame,
        Action::Advance(0.1),
        Action::Draw,
        Action::SetArtboard(
            "ab",
            Some(("data_binding_artboards_test.riv", "recursive-parent")),
        ),
        Action::Frame,
        Action::Advance(0.1),
        Action::Draw,
        Action::SetArtboard(
            "ab",
            Some(("data_binding_artboards_test.riv", "recursive-grand-parent")),
        ),
        Action::Frame,
        Action::Advance(0.1),
        Action::Draw,
        Action::SetArtboard(
            "ab",
            Some(("data_binding_artboards_test.riv", "recursive-grand-child-2")),
        ),
        Action::Frame,
        Action::Advance(0.1),
        Action::Draw,
        Action::Compare("data_binding_artboards_test_recursive"),
    ]);
}

#[test]
#[ignore = "expected-red: live cross-File draw reaches missing nested artboard graph for global 7"]
fn default_data_binding_artboard_from_different_source() {
    execute_until_concrete_parity_boundary(&[
        Action::Import("data_binding_artboards_test.riv"),
        Action::Import("data_binding_artboards_source_test.riv"),
        Action::SelectArtboard("default"),
        Action::FrameSize,
        Action::CreateViewModel,
        Action::BindViewModel,
        Action::Advance(0.1),
        Action::Draw,
        Action::Frame,
        Action::Advance(0.1),
        Action::Draw,
        Action::SetArtboard("ab", Some(("data_binding_artboards_test.riv", "ch1"))),
        Action::Frame,
        Action::Advance(0.1),
        Action::Draw,
        Action::SetArtboard("ab", Some(("data_binding_artboards_test.riv", "ch2"))),
        Action::Frame,
        Action::Advance(0.1),
        Action::Draw,
        Action::SetArtboard(
            "ab",
            Some(("data_binding_artboards_source_test.riv", "default")),
        ),
        Action::Frame,
        Action::Advance(0.1),
        Action::Draw,
        Action::Compare("data_binding_artboards_default_test"),
    ]);
}

#[test]
#[ignore = "expected-red: complete live scripted Artboard stream reaches the unavailable pinned SRIV comparator"]
fn scripted_artboard_input_bound_to_internal_and_external_artboards() {
    execute_until_concrete_parity_boundary(&[
        Action::Import("data_bind_artboard_input.riv"),
        Action::Import("data_binding_artboards_source_test.riv"),
        Action::SelectArtboard("default"),
        Action::FrameSize,
        Action::CreateViewModel,
        Action::BindViewModel,
        Action::Draw,
        Action::Advance(0.1),
        Action::Draw,
        Action::SetArtboard(
            "artboardProperty",
            Some(("data_bind_artboard_input.riv", "child2")),
        ),
        Action::Frame,
        Action::Advance(0.1),
        Action::Draw,
        Action::SetArtboard(
            "artboardProperty",
            Some(("data_bind_artboard_input.riv", "child1")),
        ),
        Action::Frame,
        Action::Advance(0.1),
        Action::Draw,
        Action::SetArtboard(
            "artboardProperty",
            Some(("data_binding_artboards_source_test.riv", "default")),
        ),
        Action::Frame,
        Action::Advance(0.1),
        Action::Draw,
        Action::SetArtboardIndex("artboardProperty", 1),
        Action::Frame,
        Action::Advance(0.1),
        Action::Draw,
        Action::SetArtboardIndex("artboardProperty", 10),
        Action::Frame,
        Action::Advance(0.1),
        Action::Draw,
        Action::Compare("data_bind_artboard_input"),
    ]);
}

#[test]
#[ignore = "expected-red: live cross-File set reaches the missing bound ViewModelInstanceArtboard setter"]
fn external_artboard_with_no_initial_source() {
    execute_until_concrete_parity_boundary(&[
        Action::Import("databind_external_artboard_main.riv"),
        Action::Import("databind_external_artboard_child.riv"),
        Action::SelectArtboard("default"),
        Action::FrameSize,
        Action::Advance(0.0),
        Action::Draw,
        Action::Frame,
        Action::CreateViewModel,
        Action::BindViewModel,
        Action::SetArtboard(
            "ab",
            Some(("databind_external_artboard_child.riv", "ExternalChild")),
        ),
        Action::SetBoundViewModel("ab", "Child", "Instance"),
        Action::ReplaceViewModel("child", "Child/Instance"),
        Action::SetString("child/label", "updated label"),
        Action::Advance(0.016),
        Action::Draw,
        Action::Compare("databind_external_artboard_main"),
    ]);
}

#[test]
#[ignore = "expected-red: imported bound ViewModel cannot yet be injected into ViewModelInstanceArtboard"]
fn bound_artboard_with_view_model_instance_resets_properties() {
    execute_until_concrete_parity_boundary(&[
        Action::Import("bindable_artboard_nesty.riv"),
        Action::Import("bindable_artboard_child.riv"),
        Action::SelectArtboard("default"),
        Action::FrameSize,
        Action::CreateViewModel,
        Action::SetArtboard(
            "someArtboard",
            Some(("bindable_artboard_child.riv", "Artboard")),
        ),
        Action::SetBoundViewModel("someArtboard", "ViewModelRenamed", "new"),
        Action::BindViewModel,
        Action::Advance(0.016),
        Action::Draw,
        Action::Frame,
        Action::PointerDown(250.0, 250.0),
        Action::PointerUp(250.0, 250.0),
        Action::Advance(0.016),
        Action::Draw,
        Action::Frame,
        Action::Advance(0.1),
        Action::Draw,
        Action::Frame,
        Action::Advance(0.1),
        Action::Draw,
        Action::Frame,
        Action::Advance(0.1),
        Action::Draw,
        Action::Frame,
        Action::Advance(0.1),
        Action::Draw,
        Action::Frame,
        Action::Advance(0.1),
        Action::Draw,
        Action::Frame,
        Action::PointerDown(250.0, 250.0),
        Action::PointerUp(250.0, 250.0),
        Action::Advance(0.016),
        Action::Draw,
        Action::Compare("bindable_artboard_nesty"),
    ]);
}

#[test]
#[ignore = "expected-red: live initial draw reaches missing nested artboard graph for global 794"]
fn multiple_targets_bound_to_same_artboard_property_bidirectionally() {
    let mut actions = vec![
        Action::Import("bidirectional_binding_source.riv"),
        Action::Import("bidirectional_binding_target_1.riv"),
        Action::Import("bidirectional_binding_target_2.riv"),
        Action::SelectArtboard("default"),
        Action::FrameSize,
        Action::CreateViewModel,
        Action::BindViewModel,
        Action::Advance(0.0),
        Action::SetBool("costume_db_bool", true),
        Action::Draw,
        Action::Frame,
        Action::Advance(0.016),
        Action::Draw,
    ];
    for source in [
        "bidirectional_binding_target_1.riv",
        "bidirectional_binding_target_2.riv",
        "bidirectional_binding_target_1.riv",
        "bidirectional_binding_target_2.riv",
    ] {
        actions.extend([
            Action::Frame,
            Action::SetArtboard("costume_db_artboard", Some((source, "costume_artboard"))),
            Action::Advance(0.016),
            Action::Draw,
            Action::Frame,
            Action::Advance(0.016),
            Action::Draw,
        ]);
    }
    actions.push(Action::Compare("bidirectional_binding_source"));
    execute_until_concrete_parity_boundary(&actions);
}

#[test]
#[ignore = "expected-red: facade has no callable nested-Artboard occurrence-presence query"]
fn null_bound_artboard_swap_survives_pending_layout_sync() {
    execute_until_concrete_parity_boundary(&[
        Action::Import("databind_null_artboard_swap.riv"),
        Action::SelectArtboard("default"),
        Action::CreateViewModel,
        Action::BindViewModel,
        Action::ExpectNested("swap host", true),
        Action::Advance(0.0),
        Action::Advance(0.0),
        Action::ExpectNested("swap host", false),
        Action::Compare("no-crash-and-null-instance"),
    ]);
}
